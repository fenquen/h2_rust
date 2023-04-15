use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Byte, Long};
use crate::mvstore::page::PageTraitRef;

pub type RootReferenceRef<K, V> = Option<Arc<AtomicRefCell<RootReference<K, V>>>>;

pub struct RootReference<K, V> {
    /// The root page.
    pub root: PageTraitRef<K, V>,

    /// The version used for writing.
    pub version: Long,

    /// Counter of reentrant locks.
    hold_count: Byte,

    /// Lock owner thread id.
    owner_id: Long,

    /// volatile
    /// Reference to the previous root in the chain.
    /// That is the last root of the previous version, which had any data changes.
    /// Versions without any data changes are dropped from the chain, as it built.
    previous: RootReferenceRef<K, V>,

    /// Counter for successful root updates.
    update_counter: Long,

    /// Counter for attempted root updates.
    update_attempt_counter: Long,

    /// Size of the occupied part of the append buffer.
    append_counter: Byte,
}

impl<K, V> RootReference<K, V> {
    /// This one is used to set root initially and for r/o snapshots
    pub fn new(root: PageTraitRef<K, V>, version: Long) -> RootReferenceRef<K, V> {
        let root_reference = RootReference {
            root,
            version,
            hold_count: 0,
            owner_id: 0,
            previous: None,
            update_counter: 1,
            update_attempt_counter: 1,
            append_counter: 0,
        };

        Some(Arc::new(AtomicRefCell::new(root_reference)))
    }
}