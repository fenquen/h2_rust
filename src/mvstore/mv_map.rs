use std::ops::{Deref, DerefMut};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, AtomicPtr};
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::MVStoreRef;
use crate::mvstore::page::{Page, PageTrait, PageTraitRef};
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::root_reference::{RootReference, RootReferenceRef};

pub type MVMapRef<K, V> = Option<Arc<AtomicRefCell<MVMap<K, V>>>>;

#[derive(Default)]
pub struct MVMap<K, V> {
    mv_store: MVStoreRef,
    id: Integer,
    create_version: Long,
    keys_buffer: Option<Vec<K>>,
    values_buffer: Option<Vec<V>>,
    pub key_type: Option<Arc<dyn DataType<K> + Send + Sync>>,
    pub value_type: Option<Arc<dyn DataType<V> + Send + Sync>>,
    single_writer: bool,
    /// volatile 通过set函数中的mutex模拟
    root_reference: RootReferenceRef<K, V>,
    root_reference_set_mutex: Mutex<()>,
    avg_key_size: Option<AtomicI64>,
    avg_val_size: Option<AtomicI64>,
    keys_per_page: Integer,
    pub is_volatile: bool,
}

impl<K, V> MVMap<K, V> where K: Default + Send + Sync + 'static,
                             V: Default + Send + Sync + 'static {
    pub fn new(mv_store_ref: MVStoreRef,
               id: Integer,
               key_type: Arc<dyn DataType<K> + Send + Sync>,
               value_type: Arc<dyn DataType<V> + Send + Sync>) -> Result<MVMapRef<K, V>> {
        let keys_per_page = mv_store_ref.as_ref().unwrap().borrow().keys_per_page;
        let current_version = mv_store_ref.as_ref().unwrap().borrow().get_current_version();
        let mv_map_ref = Self::new1(mv_store_ref.clone(),
                                    key_type,
                                    value_type,
                                    id,
                                    0,
                                    None,
                                    keys_per_page,
                                    false)?;

        let mut mv_map = mv_map_ref.as_ref().unwrap().borrow_mut();
        let mv_map = mv_map.deref_mut();

        let mv_store = mv_store_ref.as_ref().unwrap().borrow();
        let mv_store = mv_store.deref();

        mv_map.set_initial_root(mv_map.create_empty_leaf(mv_map_ref.clone()), mv_store.get_current_version());

        Ok(mv_map_ref.clone())
    }

    fn new1(mv_store: MVStoreRef,
            key_type: Arc<dyn DataType<K> + Send + Sync>,
            value_type: Arc<dyn DataType<V> + Send + Sync>,
            id: Integer,
            create_version: Long,
            root_reference: RootReferenceRef<K, V>,
            keys_per_page: Integer,
            single_writer: bool) -> Result<MVMapRef<K, V>> {
        let mut mv_map = MVMap::<K, V>::default();

        mv_map.mv_store = mv_store;
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

        Ok(Some(Arc::new(AtomicRefCell::new(mv_map))))
    }


    fn create_empty_leaf(&self, mv_map_ref: MVMapRef<K, V>) -> PageTraitRef<K, V> {
        Page::<K, V>::create_empty_leaf(mv_map_ref)
    }

    pub fn is_persistent(&self) -> bool {
        return self.mv_store.is_some() && !self.is_volatile;
    }

    fn set_initial_root(&mut self, root_page: PageTraitRef<K, V>, version: Long) {
        self.set_root_reference(RootReference::new(root_page, version));
    }

    fn set_root_reference(&mut self, root_reference: RootReferenceRef<K, V>) {
        self.root_reference_set_mutex.lock().unwrap();
        self.root_reference = root_reference;
    }
}