use std::sync::Arc;
use std::thread;
use crate::{build_option_arc_h2RustCell, get_ref, h2_rust_cell_equals};
use crate::h2_rust_common::{Byte, Long};
use crate::h2_rust_common::h2_rust_cell::{H2RustCell, SharedPtr};
use crate::mvstore::page::{PageTrait};

pub struct RootReference {
    /// The root page.
    pub root: SharedPtr<dyn PageTrait>,

    /// The version used for writing.
    pub version: Long,

    /// Counter of reentrant locks.
    holdCount: Byte,

    /// lock owner thread id.
    ownerId: Long,

    /// volatile
    /// Reference to the previous root in the chain.
    /// That is the last root of the previous version, which had any data changes.
    /// Versions without any data changes are dropped from the chain, as it built.
    previous: SharedPtr<RootReference>,

    /// counter for successful root updates.
    updateCounter: Long,

    /// counter for attempted root updates.
    updateAttemptCounter: Long,

    /// size of the occupied part of the append buffer.
    appendCounter: Byte,
}

impl RootReference {
    /// This one is used to set root initially and for r/o snapshots
    pub fn new(root: SharedPtr<dyn PageTrait>, version: Long) -> SharedPtr<RootReference> {
        let root_reference = RootReference {
            root,
            version,
            holdCount: 0,
            ownerId: 0,
            previous: None,
            updateCounter: 1,
            updateAttemptCounter: 1,
            appendCounter: 0,
        };

        build_option_arc_h2RustCell!(root_reference)
    }

    pub fn getVersion(&self) -> Long {
        let prev = self.previous.clone();
        if prev.is_none() || h2_rust_cell_equals!(get_ref!(prev).root,self.root) || get_ref!(prev).appendCounter != self.appendCounter {
            self.version
        } else {
            get_ref!(prev).getVersion()
        }
    }

    pub fn isLockedByCurrentThread(&self) -> bool {
        self.holdCount != 0 && self.ownerId == thread::current().id().as_u64().get() as Long
    }
}