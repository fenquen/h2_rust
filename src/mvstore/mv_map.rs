use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use anyhow::Result;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicPtr, Ordering};
use usync::RwLock;
use crate::{build_option_arc_h2RustCell, get_ref, get_ref_mut, h2_rust_cell_equals, suffix_plus_plus, throw, weak_get_ref, weak_get_ref_mut};
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_utils, Integer, Long, Nullable};
use crate::h2_rust_common::h2_rust_cell::{H2RustCell, SharedPtr, WeakPtr};
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::message::db_error::DbError;
use crate::mvstore::mv_store::{MVStore};
use crate::mvstore::{data_utils, page};
use crate::mvstore::page::{Page, PageTrait};
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::root_reference::{RootReference};

#[derive(Default)]
pub struct MVMap {
    /// 隶属的上头
    mvStore: WeakPtr<MVStore>,
    id: Integer,
    createVersion: Long,
    keys_buffer: Option<Vec<H2RustType>>,
    values_buffer: Option<Vec<H2RustType>>,
    pub keyType: Option<Arc<dyn DataType>>,
    pub valueType: Option<Arc<dyn DataType>>,
    pub single_writer: bool,

    /// volatile 通过set函数中的mutex模拟
    rootReference: SharedPtr<RootReference>,
    rootReferenceSetMutex: RwLock<()>,

    avgKeySize: Option<AtomicI64>,
    avgValSize: Option<AtomicI64>,
    keysPerPage: Integer,
    pub isVolatile: bool,
    readOnly: bool,
    closed: AtomicBool,
}

impl MVMap where {
    pub fn new(mvStoreWeakPtr: WeakPtr<MVStore>,
               id: Integer,
               key_type: Arc<dyn DataType>,
               value_type: Arc<dyn DataType>) -> Result<SharedPtr<MVMap>> {
        let arc_h2RustCell_mvStore = mvStoreWeakPtr.as_ref().unwrap().upgrade().unwrap();
        let mvStoreRef = arc_h2RustCell_mvStore.get_ref();

        let keys_per_page = mvStoreRef.keysPerPage;
        let current_version = mvStoreRef.getCurrentVersion();
        let mv_map_ref = Self::new1(mvStoreWeakPtr.clone(),
                                    key_type,
                                    value_type,
                                    id,
                                    0,
                                    None,
                                    keys_per_page,
                                    false)?;
        let mvMapMutRef = get_ref_mut!(mv_map_ref);

        mvMapMutRef.setInitialRoot(mvMapMutRef.createEmptyLeaf(mv_map_ref.clone()), mvStoreRef.getCurrentVersion());

        Ok(mv_map_ref.clone())
    }

    fn new1(mvStoreWeakPtr: WeakPtr<MVStore>,
            key_type: Arc<dyn DataType>,
            value_type: Arc<dyn DataType>,
            id: Integer,
            create_version: Long,
            root_reference: SharedPtr<RootReference>,
            keys_per_page: Integer,
            single_writer: bool) -> Result<SharedPtr<MVMap>> {
        let mut mv_map = MVMap::default();

        mv_map.mvStore = mvStoreWeakPtr;
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
        return self.mvStore.as_ref().unwrap().upgrade().is_some() && !self.isVolatile;
    }

    fn setInitialRoot(&mut self, root_page: SharedPtr<dyn PageTrait>, version: Long) {
        self.setRootReference(RootReference::new(root_page, version));
    }

    fn setRootReference(&mut self, root_reference: SharedPtr<RootReference>) {
        let write_guard = self.rootReferenceSetMutex.write();
        self.rootReference = root_reference;
    }

    /// set the position of the root page.
    pub fn setRootPosition(&mut self, rootPosition: Long, version: Long, this: SharedPtr<MVMap>) {
        let mut root: SharedPtr<dyn PageTrait> = self.readOrCreateRootPage(rootPosition, this.clone());

        let mvMap = get_ref!(root).getMvMap();
        if h2_rust_cell_equals!(mvMap,this) {
            // this can only happen on concurrent opening of existing map,
            // when second thread picks up some cached page already owned by the first map's instantiation (both maps share the same id)
            assert_eq!(self.id, get_ref!(mvMap).id);

            root = get_ref!(root).copy(this, false, root.clone());

            self.setInitialRoot(root, version);

            self.setWriteVersion(weak_get_ref!(self.mvStore).currentVersion.load(Ordering::Acquire));
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
        weak_get_ref_mut!(self.mvStore).readPage(this, position)
    }

    pub fn get(&self, key: &H2RustType) -> H2RustType {
        self.get2(self.getRootPage(), key)
    }

    pub fn get2(&self, pageTraitSharedPtr: SharedPtr<dyn PageTrait>, key: &H2RustType) -> H2RustType {
        page::get(pageTraitSharedPtr, key)
    }

    pub fn getRootPage(&self) -> SharedPtr<dyn PageTrait> {
        let root_reference_ref = self.flushAndGetRootReference();
        get_ref!(root_reference_ref).root.clone()
    }

    pub fn flushAndGetRootReference(&self) -> SharedPtr<RootReference> {
        let r = self.getRootReference();
        // todo 因为通常singleWriter是false 且 flushAppendBuffer()很难 暂时的略过
        //if (singleWriter && rootReference.getAppendCounter() > 0) {
        //return flushAppendBuffer(rootReference, true);
        //}
        r
    }

    pub fn getRootReference(&self) -> SharedPtr<RootReference> {
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

    pub fn setWriteVersion(&mut self, writeVersion: Long) -> SharedPtr<RootReference> {
        let mut attempt = 0;
        loop {
            let mut rootReference = self.flushAndGetRootReference();
            if get_ref!(rootReference).version >= writeVersion {
                return rootReference;
            }

            if self.closed.load(Ordering::Acquire) {
                // map was closed a while back and can not possibly be in use by now
                // it's time to remove it completely from the store (it was anonymous already)
                if get_ref!(rootReference).getVersion() + 1 < weak_get_ref!(self.mvStore).getOldestVersionToKeep() {
                    //     mvStore.deregisterMapRoot(id);
                    //     return Option::default();
                }
            }
        }
    }

    pub fn remove(&mut self, key: H2RustType) {
        self.operate(key, H2RustType::Null, DecisionMaker.REMOVE);
    }

    pub fn operate(&mut self,
                   key: H2RustType,
                   value: H2RustType,
                   decisionMaker: impl DecisionMaker) {
        let attempt = 0;

        loop {
            let rootReference = self.flushAndGetRootReference();
            let locked = get_ref!(rootReference).isLockedByCurrentThread();

            if !locked {
                if suffix_plus_plus!(attempt) == 0 {
                    self.beforeWrite();
                }
            }
        }
    }

    pub fn beforeWrite(&self) -> Result<()> {
        let mvStore = weak_get_ref_mut!(self.mvStore);

        if self.closed.load(Ordering::Acquire) {
            let mapName = mvStore.getMapName(self.id);
            throw!(DbError::get(error_code::DATABASE_IS_CLOSED,vec![]));
        }

        if self.readOnly {
            throw!(DbError::get(error_code::GENERAL_ERROR_1,vec!["this map is read only"]));
        }


        mvStore.beforeWrite(self);
        todo!()
    }
}

pub fn getMapRootKey(mapId: Integer) -> String {
    format!("{}{}", data_utils::META_ROOT, format!("{:x}", mapId))
}

pub fn getMapKey(mapId: Integer) -> String {
    format!("{}{}", data_utils::META_MAP, h2_rust_utils::int2HexString(mapId))
}

pub enum Decision {
    ABORT,
    REMOVE,
    PUT,
    REPEAT,
}

pub trait DecisionMaker {}