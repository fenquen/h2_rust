use anyhow::Result;
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::{h2_rust_constant, Integer, Long};
use crate::mvstore::data_utils;

/// The maximum chunk id.
pub const MAX_ID: Integer = (1 << 26) - 1;

/// The maximum length of a chunk header, in bytes.
static MAX_HEADER_LENGTH: Integer = 1024;

/// The length of the chunk footer. The longest footer is:
/// chunk:ffffffff,block:ffffffffffffffff,
/// version:ffffffffffffffff,fletcher:ffffffff
static FOOTER_LENGTH: Integer = 128;

const ATTR_CHUNK: &str = "chunk";
const ATTR_BLOCK: &str = "block";
const ATTR_LEN: &str = "len";
const ATTR_MAP: &str = "map";
const ATTR_MAX: &str = "max";
const ATTR_NEXT: &str = "next";
const ATTR_PAGES: &str = "pages";
const ATTR_ROOT: &str = "root";
const ATTR_TIME: &str = "time";
const ATTR_VERSION: &str = "version";
const ATTR_LIVE_MAX: &str = "liveMax";
const ATTR_LIVE_PAGES: &str = "livePages";
const ATTR_UNUSED: &str = "unused";
const ATTR_UNUSED_AT_VERSION: &str = "unusedAtVersion";
const ATTR_PIN_COUNT: &str = "pinCount";
const ATTR_TOC: &str = "toc";
const ATTR_OCCUPANCY: &str = "occupancy";
const ATTR_FLETCHER: &str = "fletcher";

pub type ChunkSharedPtr = Option<Arc<H2RustCell<Chunk>>>;

pub struct Chunk {
    pub id: Integer,
    pub version: Long,
    pub layout_root_pos: Long,
    pub map_id: Integer,
}

impl Chunk {
    pub fn new(s: &String) -> Result<ChunkSharedPtr> {
        Self::new2(data_utils::parseMap(s)?, true);
        todo!()
    }

    fn new2(map: HashMap<String, String>, full: bool) -> ChunkSharedPtr {
        todo!()
    }
}

pub fn get_meta_key(chunk_id: Integer) -> String {
    let chunk_id_hex_string = format!("{:x}", chunk_id);
    ATTR_CHUNK.to_string().add(h2_rust_constant::DOT).add(&chunk_id_hex_string)
}

/// Build a block from the given string.
pub fn fromString(s: &String) -> Result<ChunkSharedPtr> {
    Chunk::new(s)
}