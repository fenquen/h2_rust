use crate::h2_rust_common::h2_rust_cell::SharedPtr;
use crate::mvstore::page::PageTrait;

pub struct CursorPos {
    /// the page at the current level.
    pub page :SharedPtr<dyn PageTrait>,

}

// a