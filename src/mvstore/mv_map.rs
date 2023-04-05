use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::Nullable;
use crate::mvstore::mv_store::MVStoreRef;

pub type MVMapRef<K, V> = Nullable<Arc<AtomicRefCell<MVMap<K, V>>>>;

pub struct MVMap<K, V> {
    keys_buffer: Vec<K>,
    values_buffer: Vec<V>,
}