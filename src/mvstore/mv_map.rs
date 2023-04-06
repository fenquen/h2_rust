use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::MVStoreRef;
use crate::mvstore::r#type::data_type::DataType;

pub type MVMapRef<K, V> = Option<Arc<AtomicRefCell<MVMap<K, V>>>>;

#[derive(Default)]
pub struct MVMap<K, V> {
    keys_buffer: Vec<K>,
    values_buffer: Vec<V>,
    key_type: Option<Arc<dyn DataType<K> + Send + Sync>>,
    value_type: Option<Arc<dyn DataType<V> + Send + Sync>>,
}

impl<K: Default, V: Default> MVMap<K, V> {
    pub fn new(mv_store: MVStoreRef,
               id: Integer,
               key_type: Arc<dyn DataType<K> + Send + Sync>,
               value_type: Arc<dyn DataType<V> + Send + Sync>) {
        let mut mv_map = MVMap::<K, V>::default();
        mv_map.key_type = Some(key_type);
        mv_map.value_type = Some(value_type);
    }
}