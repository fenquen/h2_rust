use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicPtr};
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::MVStoreRef;
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
    root_reference: AtomicPtr<RootReferenceRef<K, V>>,
    avg_key_size: Option<AtomicI64>,
    avg_val_size: Option<AtomicI64>,

    keys_per_page: Integer,
}

impl<K: Default, V: Default> MVMap<K, V> {
    pub fn new(mv_store: MVStoreRef,
               id: Integer,
               key_type: Arc<dyn DataType<K> + Send + Sync>,
               value_type: Arc<dyn DataType<V> + Send + Sync>) -> Result<MVMapRef<K, V>> {
        let keys_per_page = mv_store.as_ref().unwrap().borrow().keys_per_page;
        let current_version = mv_store.as_ref().unwrap().borrow().get_current_version();
        let this = Self::new1(mv_store,
                              key_type,
                              value_type,
                              id,
                              0,
                              AtomicPtr::default(),
                              keys_per_page,
                              false)?;

        Ok(this)
    }

    fn new1(mv_store: MVStoreRef,
            key_type: Arc<dyn DataType<K> + Send + Sync>,
            value_type: Arc<dyn DataType<V> + Send + Sync>,
            id: Integer,
            create_version: Long,
            root_reference: AtomicPtr<RootReferenceRef<K, V>>,
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


    fn create_empty_leaf(&self) {}
}