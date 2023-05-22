use std::sync::Arc;
use crate::build_option_arc_h2RustCell;
use crate::h2_rust_common::{Byte, Long};
use crate::h2_rust_common::h2_rust_cell::{H2RustCell, SharedPtr};
use crate::mvstore::page::{PageTrait};

pub type RootReferenceRef = Option<Arc<H2RustCell<RootReference>>>;

pub struct RootReference {
    /// The root page.
    pub root: SharedPtr<dyn PageTrait>,

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
    previous: RootReferenceRef,

    /// Counter for successful root updates.
    update_counter: Long,

    /// Counter for attempted root updates.
    update_attempt_counter: Long,

    /// Size of the occupied part of the append buffer.
    append_counter: Byte,
}

impl RootReference {
    /// This one is used to set root initially and for r/o snapshots
    pub fn new(root: SharedPtr<dyn PageTrait>, version: Long) -> RootReferenceRef {
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

        build_option_arc_h2RustCell!(root_reference)
    }
}