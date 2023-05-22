use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use anyhow::Result;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicI64, AtomicPtr};
use usync::RwLock;
use crate::{build_option_arc_h2RustCell, get_ref, get_ref_mut, h2_rust_cell_equals};
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::h2_rust_cell::{H2RustCell, SharedPtr};
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::{MVStore};
use crate::mvstore::page;
use crate::mvstore::page::{Page, PageTrait};
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::root_reference::{RootReference, RootReferenceRef};

#[derive(Default)]
pub struct MVMap {
    /// 隶属的上头
    mv_store: Weak<H2RustCell<MVStore>>,
    id: Integer,
    createVersion: Long,
    keys_buffer: Option<Vec<H2RustType>>,
    values_buffer: Option<Vec<H2RustType>>,
    pub keyType: Option<Arc<dyn DataType>>,
    pub valueType: Option<Arc<dyn DataType>>,
    single_writer: bool,

    /// volatile 通过set函数中的mutex模拟
    rootReference: RootReferenceRef,
    rootReferenceSetMutex: RwLock<()>,

    avgKeySize: Option<AtomicI64>,
    avgValSize: Option<AtomicI64>,
    keysPerPage: Integer,
    pub isVolatile: bool,
}

impl MVMap where {
    pub fn new(mvStoreWeakPtr: Weak<H2RustCell<MVStore>>,
               id: Integer,
               key_type: Arc<dyn DataType>,
               value_type: Arc<dyn DataType>) -> Result<SharedPtr<MVMap>> {
        let arc_h2RustCell_mvStore = mvStoreWeakPtr.upgrade().unwrap();
        let mvStoreRef = arc_h2RustCell_mvStore.get_ref();

        let keys_per_page = mvStoreRef.keysPerPage;
        let current_version = mvStoreRef.get_current_version();
        let mv_map_ref = Self::new1(mvStoreWeakPtr.clone(),
                                    key_type,
                                    value_type,
                                    id,
                                    0,
                                    None,
                                    keys_per_page,
                                    false)?;
        let mvMapMutRef = get_ref_mut!(mv_map_ref);

        mvMapMutRef.set_initial_root(mvMapMutRef.createEmptyLeaf(mv_map_ref.clone()), mvStoreRef.get_current_version());

        Ok(mv_map_ref.clone())
    }

    fn new1(mvStoreWeakPtr: Weak<H2RustCell<MVStore>>,
            key_type: Arc<dyn DataType>,
            value_type: Arc<dyn DataType>,
            id: Integer,
            create_version: Long,
            root_reference: RootReferenceRef,
            keys_per_page: Integer,
            single_writer: bool) -> Result<SharedPtr<MVMap>> {
        let mut mv_map = MVMap::default();

        mv_map.mv_store = mvStoreWeakPtr;
        mv_map.id = id;
        mv_map.createVersion = create_version;
        mv_map.keyType = Some(key_type);
        mv_map.valueType = Some(value_type);
        mv_map.rootReference = root_reference;
        mv_map.keysPerPage = keys_per_page;
        mv_map.single_writer = single_writer;

        if single_writer {
            mv_map.keys_buffer = Some(mv_map.keyType.as_ref().unwrap().create_storage(keys_per_page));
            mv_map.values_buffer = Some(mv_map.valueType.as_ref().unwrap().create_storage(keys_per_page));
        }

        if mv_map.keyType.as_ref().unwrap().is_memory_estimation_allowed() {
            mv_map.avgKeySize = Some(AtomicI64::new(0));
        }

        if mv_map.valueType.as_ref().unwrap().is_memory_estimation_allowed() {
            mv_map.avgValSize = Some(AtomicI64::new(0));
        }

        Ok(build_option_arc_h2RustCell!(mv_map))
    }

    fn createEmptyLeaf(&self, this: SharedPtr<MVMap>) -> SharedPtr<dyn PageTrait> {
        Page::createEmptyLeaf(this)
    }

    pub fn is_persistent(&self) -> bool {
        return self.mv_store.upgrade().is_some() && !self.isVolatile;
    }

    fn set_initial_root(&mut self, root_page: SharedPtr<dyn PageTrait>, version: Long) {
        self.set_root_reference(RootReference::new(root_page, version));
    }

    fn set_root_reference(&mut self, root_reference: RootReferenceRef) {
        let write_guard = self.rootReferenceSetMutex.write();
        self.rootReference = root_reference;
    }

    /// set the position of the root page.
    pub fn setRootPosition(&self, rootPosition: Long, version: Long, this: SharedPtr<MVMap>) {
        let mut root: SharedPtr<dyn PageTrait> = self.readOrCreateRootPage(rootPosition, this.clone());

        let mvMap = get_ref!(root).getMvMap();
        if h2_rust_cell_equals!(mvMap,this) {
            // this can only happen on concurrent opening of existing map,
            // when second thread picks up some cached page already owned by
            // the first map's instantiation (both maps share the same id)
            assert_eq!(self.id, get_ref!(mvMap).id);

            root = get_ref!(root).copy(this, false, root.clone());
        }
    }

    fn readOrCreateRootPage(&self, rootPosition: Long, this: SharedPtr<MVMap>) -> SharedPtr<dyn PageTrait> {
        if rootPosition == 0 {
            self.createEmptyLeaf(this)
        } else {
            self.readPage(this, rootPosition).unwrap()
        }
    }

    pub fn readPage(&self, this: SharedPtr<MVMap>, position: Long) -> Result<SharedPtr<dyn PageTrait>> {
        self.mv_store.upgrade().unwrap().get_ref_mut().readPage(this, position)
    }

    pub fn get(&self, key: &H2RustType) -> H2RustType {
        self.get2(self.getRootPage(), key)
    }

    pub fn get2(&self, page_trait_ref: SharedPtr<dyn PageTrait>, key: &H2RustType) -> H2RustType {
        page::get(page_trait_ref, key)
    }

    pub fn getRootPage(&self) -> SharedPtr<dyn PageTrait> {
        let root_reference_ref = self.flushAndGetRootReference();
        get_ref!(root_reference_ref).root.clone()
    }

    pub fn flushAndGetRootReference(&self) -> RootReferenceRef {
        let r = self.get_root_reference();
        // todo 因为通常singleWriter是false 且 flushAppendBuffer()很难 暂时的略过
        //if (singleWriter && rootReference.getAppendCounter() > 0) {
        //return flushAppendBuffer(rootReference, true);
        //}
        r
    }

    pub fn get_root_reference(&self) -> RootReferenceRef {
        let read_guard = self.rootReferenceSetMutex.read();
        self.rootReference.clone()
    }

    pub fn getKeyType(&self) -> Arc<dyn DataType> {
        self.keyType.as_ref().unwrap().clone()
    }

    pub fn getValueType(&self) -> Arc<dyn DataType> {
        self.valueType.as_ref().unwrap().clone()
    }

    pub fn getId(&self) -> Integer {
        self.id
    }
}