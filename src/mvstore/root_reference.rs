use std::sync::Arc;
use atomic_refcell::AtomicRefCell;

pub type RootReferenceRef<K, V> = Option<Arc<AtomicRefCell<RootReference<K, V>>>>;

pub struct RootReference<K, V> {
    a:K,

    d:V
}