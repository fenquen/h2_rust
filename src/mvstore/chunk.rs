use std::sync::Arc;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;

pub type ChunkRef = Option<Arc<H2RustCell<Chunk>>>;

pub struct Chunk {

}

impl Chunk{
    pub fn show (&mut self){

    }
}