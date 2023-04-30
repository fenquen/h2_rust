use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use anyhow::Result;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicI64, AtomicPtr};
use usync::RwLock;
use crate::{build_option_arc_h2RustCell, get_ref, get_ref_mut};
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::{MVStore, MVStoreRef, MVStoreWeakPtr};
use crate::mvstore::page;
use crate::mvstore::page::{Page, PageTrait, PageTraitSharedPtr};
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::root_reference::{RootReference, RootReferenceRef};

pub type MVMapSharedPtr = Option<Arc<H2RustCell<MVMap>>>;

#[derive(Default)]
pub struct MVMap {
    /// 隶属的上头
    mv_store: MVStoreWeakPtr,
    id: Integer,
    create_version: Long,
    keys_buffer: Option<Vec<H2RustType>>,
    values_buffer: Option<Vec<H2RustType>>,
    pub key_type: Option<Arc<dyn DataType>>,
    pub value_type: Option<Arc<dyn DataType>>,
    single_writer: bool,

    /// volatile 通过set函数中的mutex模拟
    root_reference: RootReferenceRef,
    root_reference_set_mutex: RwLock<()>,

    avg_key_size: Option<AtomicI64>,
    avg_val_size: Option<AtomicI64>,
    keys_per_page: Integer,
    pub is_volatile: bool,

}

impl MVMap where {
    pub fn new(mvStoreWeakPtr: MVStoreWeakPtr,
               id: Integer,
               key_type: Arc<dyn DataType>,
               value_type: Arc<dyn DataType>) -> Result<MVMapSharedPtr> {
        let arc_h2RustCell_mvStore = mvStoreWeakPtr.upgrade().unwrap();
        let mvStoreRef = arc_h2RustCell_mvStore.get_ref();

        let keys_per_page = mvStoreRef.keys_per_page;
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

        mvMapMutRef.set_initial_root(mvMapMutRef.create_empty_leaf(mv_map_ref.clone()), mvStoreRef.get_current_version());

        Ok(mv_map_ref.clone())
    }

    fn new1(mvStoreWeakPtr: MVStoreWeakPtr,
            key_type: Arc<dyn DataType>,
            value_type: Arc<dyn DataType>,
            id: Integer,
            create_version: Long,
            root_reference: RootReferenceRef,
            keys_per_page: Integer,
            single_writer: bool) -> Result<MVMapSharedPtr> {
        let mut mv_map = MVMap::default();

        mv_map.mv_store = mvStoreWeakPtr;
        mv_map.id = id;
        mv_map.create_version = create_version;
        mv_map.key_type = Some(key_type);
        mv_map.value_type = Some(value_type);
        mv_map.root_reference = root_reference;
        mv_map.keys_per_page = keys_per_page;
        mv_map.single_writer = single_writer;

        if single_writer {
            mv_map.keys_buffer = Some(mv_map.key_type.as_ref().unwrap().create_storage(keys_per_page));
            mv_map.values_buffer = Some(mv_map.value_type.as_ref().unwrap().create_storage(keys_per_page));
        }

        if mv_map.key_type.as_ref().unwrap().is_memory_estimation_allowed() {
            mv_map.avg_key_size = Some(AtomicI64::new(0));
        }

        if mv_map.value_type.as_ref().unwrap().is_memory_estimation_allowed() {
            mv_map.avg_val_size = Some(AtomicI64::new(0));
        }

        Ok(build_option_arc_h2RustCell!(mv_map))
    }

    fn create_empty_leaf(&self, this: MVMapSharedPtr) -> PageTraitSharedPtr {
        Page::create_empty_leaf(this)
    }

    pub fn is_persistent(&self) -> bool {
        return self.mv_store.upgrade().is_some() && !self.is_volatile;
    }

    fn set_initial_root(&mut self, root_page: PageTraitSharedPtr, version: Long) {
        self.set_root_reference(RootReference::new(root_page, version));
    }

    fn set_root_reference(&mut self, root_reference: RootReferenceRef) {
        let write_guard = self.root_reference_set_mutex.write();
        self.root_reference = root_reference;
    }

    /// set the position of the root page.
    pub fn set_root_pos(&self, root_pos: Long, version: Long, this: MVMapSharedPtr) {
        let root: PageTraitSharedPtr = self.read_or_create_root_page(root_pos, this);
    }

    fn read_or_create_root_page(&self, root_pos: Long, this: MVMapSharedPtr) -> PageTraitSharedPtr {
        if root_pos == 0 {
            self.create_empty_leaf(this)
        } else {
            self.read_page(this, root_pos).unwrap()
        }
    }

    pub fn read_page(&self, this: MVMapSharedPtr, position: Long) -> Result<PageTraitSharedPtr> {
        self.mv_store.upgrade().unwrap().get_ref_mut().read_page(this, position)
    }

    pub fn get(&self, key: &H2RustType) -> H2RustType {
        self.get2(self.get_root_page(), key)
    }

    pub fn get2(&self, page_trait_ref: PageTraitSharedPtr, key: &H2RustType) -> H2RustType {
        page::get(page_trait_ref, key)
    }

    pub fn get_root_page(&self) -> PageTraitSharedPtr {
        let root_reference_ref = self.flush_and_get_root_reference();
        get_ref!(root_reference_ref).root.clone()
    }

    pub fn flush_and_get_root_reference(&self) -> RootReferenceRef {
        let r = self.get_root_reference();
        // todo 应为通常singleWriter是false且flushAppendBuffer()很难暂时的略过
        //if (singleWriter && rootReference.getAppendCounter() > 0) {
        //return flushAppendBuffer(rootReference, true);
        //}
        r
    }

    pub fn get_root_reference(&self) -> RootReferenceRef {
        let read_guard = self.root_reference_set_mutex.read();
        self.root_reference.clone()
    }

    pub fn get_key_type(&self) -> Arc<dyn DataType> {
        self.key_type.as_ref().unwrap().clone()
    }
}