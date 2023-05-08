use anyhow::Result;
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use bit_set::BitSet;
use crate::api::error_code;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::{Byte, byte_buffer, h2_rust_constant, Integer, Long, UInteger};
use crate::message::db_error::DbError;
use crate::mvstore::{chunk, data_utils, mv_store};
use crate::{build_option_arc_h2RustCell, get_ref, get_ref_mut, throw};
use crate::h2_rust_common::byte_buffer::ByteBuffer;
use crate::mvstore::file_store::{FileStore, FileStoreRef};
use crate::util::string_utils;

/// The maximum chunk id.
pub const MAX_ID: Integer = (1 << 26) - 1;

/// The maximum length of a chunk header, in bytes.
static MAX_HEADER_LENGTH: Integer = 1024;

/// The length of the chunk footer. The longest footer is:
///
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

#[derive(Default)]
pub struct Chunk {
    pub id: Integer,
    pub version: Long,
    pub layoutRootPos: Long,
    pub map_id: Integer,
    pub block: AtomicI64,
    /// 占有了多少个block
    pub blockCount: Integer,
    pub pageCount: Integer,
    pub pageCountLive: Integer,
    pub mapId: Integer,
    pub maxLen: Long,
    pub maxLenLive: Long,
    pub time: Long,
    pub unused: Long,
    pub unusedAtVersion: Long,
    pub next: Long,
    pub pinCount: Integer,
    pub tocPos: Integer,
    pub occupancy: BitSet,
}

impl Chunk {
    pub fn new(s: &String) -> Result<ChunkSharedPtr> {
        Self::new2(data_utils::parseMap(s)?, true)
    }

    fn new2(map: HashMap<String, String>, full: bool) -> Result<ChunkSharedPtr> {
        let chunk_id: Integer = data_utils::readHexIntOrLong(&map, ATTR_CHUNK, 0)?;
        if 0 >= chunk_id {
            throw!(DbError::get(error_code::FILE_CORRUPTED_1,vec![&format!("invalid chunk id {}",chunk_id)]));
        }

        let mut chunk = Chunk::default();

        chunk.id = chunk_id;

        chunk.block.store(data_utils::readHexIntOrLong(&map, ATTR_BLOCK, 0)?, Ordering::Release);
        chunk.version = data_utils::readHexIntOrLong(&map, ATTR_VERSION, chunk_id as Long)?;

        if full {
            chunk.blockCount = data_utils::readHexIntOrLong(&map, ATTR_LEN, 0)?;
            chunk.pageCount = data_utils::readHexIntOrLong(&map, ATTR_PAGES, 0)?;
            chunk.pageCountLive = data_utils::readHexIntOrLong(&map, ATTR_LIVE_PAGES, chunk.pageCount)?;
            chunk.mapId = data_utils::readHexIntOrLong(&map, ATTR_MAP, 0)?;
            chunk.maxLen = data_utils::readHexIntOrLong(&map, ATTR_MAX, 0)?;
            chunk.maxLenLive = data_utils::readHexIntOrLong(&map, ATTR_LIVE_MAX, chunk.maxLen)?;
            chunk.layoutRootPos = data_utils::readHexIntOrLong(&map, ATTR_ROOT, 0)?;
            chunk.time = data_utils::readHexIntOrLong(&map, ATTR_TIME, 0)?;
            chunk.unused = data_utils::readHexIntOrLong(&map, ATTR_UNUSED, 0)?;
            chunk.unusedAtVersion = data_utils::readHexIntOrLong(&map, ATTR_UNUSED_AT_VERSION, 0)?;
            chunk.next = data_utils::readHexIntOrLong(&map, ATTR_NEXT, 0)?;
            chunk.pinCount = data_utils::readHexIntOrLong(&map, ATTR_PIN_COUNT, 0)?;
            chunk.tocPos = data_utils::readHexIntOrLong(&map, ATTR_TOC, 0)?;

            let v = map.get(ATTR_OCCUPANCY);
            if v.is_some() {
                let byteVec = string_utils::convertHexString2ByteArr(v.unwrap())?;
                chunk.occupancy = BitSet::from_bytes(&byteVec);
                let cardinality = chunk.occupancy.iter().count() as Integer;
                if chunk.pageCount - chunk.pageCountLive != cardinality {
                    throw!( DbError::get(error_code::FILE_CORRUPTED_1,
                                 vec![&format!("inconsistent occupancy info {} - {} != {}", chunk.pageCount, chunk.pageCountLive, cardinality)]));
                }
            }
        }

        Ok(build_option_arc_h2RustCell!(chunk))
    }
}

impl Chunk {
    pub fn isSaved(&self) -> bool {
        self.block.load(Ordering::Acquire) != Long::MAX
    }

    pub fn readBufferForPage(&self, fileStore: FileStoreRef, offset: Integer, position: Long) -> Result<ByteBuffer> {
        loop {
            let originalBlock = self.block.load(Ordering::Acquire);

            let mut positionInFile = originalBlock * mv_store::BLOCK_SIZE as Long;
            let maxPos = positionInFile + (self.blockCount * mv_store::BLOCK_SIZE) as Long;

            positionInFile = positionInFile + offset as Long;
            if positionInFile < 0 {
                throw!(DbError::get(error_code::FILE_CORRUPTED_1,
                    vec![&format!("negative positionInFile:{}; position:{}", positionInFile, position)]));
            }

            let mut length = data_utils::getPageMaxLength(position);
            if length == data_utils::PAGE_LARGE {
                let mut byteBuffer = get_ref_mut!(fileStore).readFully(positionInFile as usize, 128)?;
                length = byteBuffer.getI32();

                length += 4;
            }

            length = Long::min(maxPos - positionInFile, length as Long) as Integer;
            if length < 0 {
                throw!(DbError::get(error_code::FILE_CORRUPTED_1,
                                    vec![&format!("illegal page length {} reading at {}; max pos {} ", length, positionInFile, maxPos)]));
            }

            let byteBuffer = get_ref_mut!(fileStore).readFully(positionInFile as usize, length as usize)?;

            if originalBlock == self.block.load(Ordering::Acquire) {
                return Ok(byteBuffer);
            }
        }
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

