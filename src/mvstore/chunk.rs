use std::sync::Arc;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::{Integer, Long};

pub type ChunkRef = Option<Arc<H2RustCell<Chunk>>>;

pub struct Chunk {
    pub id: Integer,
    pub version: Long,
    pub layout_root_pos: Long,
    pub map_id: Integer,
}

impl Chunk {
    pub fn show(&mut self) {}
}