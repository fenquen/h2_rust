use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicPtr, Ordering};
use std::thread;
use std::time::{Instant, SystemTime};
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use crate::h2_rust_common::{Byte, h2_rust_utils, Integer, Long, MyMutex, Nullable};
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::mvstore::cache::cache_long_key_lirs::{CacheLongKeyLIRS, CacheLongKeyLIRSConfig};
use crate::mvstore::{chunk, data_utils, page};
use crate::mvstore::file_store::{FileStore, FileStoreRef};
use crate::mvstore::mv_map::{MVMap, MVMapSharedPtr};
use crate::mvstore::page::{Page, PageTraitSharedPtr};
use crate::mvstore::r#type::string_data_type;
use crate::{h2_rust_cell_call,
            atomic_ref_cell,
            atomic_ref_cell_mut,
            h2_rust_cell_mut_call,
            get_ref_mut,
            build_option_arc_h2RustCell,
            get_ref,
            throw,
            build_arc_h2RustCell};
use crate::api::error_code;
use crate::db::store;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::message::db_error;
use crate::message::db_error::DbError;
use crate::mvstore::chunk::{Chunk, ChunkSharedPtr};
use crate::util::utils;

/// The following are attribute names (keys) in store header map
const HDR_H: &str = "H";
const HDR_BLOCK_SIZE: &str = "blockSize";
const HDR_FORMAT: &str = "format";
const HDR_CREATED: &str = "created";
const HDR_FORMAT_READ: &str = "formatRead";
const HDR_CHUNK: &str = "chunk";
const HDR_BLOCK: &str = "block";
const HDR_VERSION: &str = "version";
const HDR_CLEAN: &str = "clean";
const HDR_FLETCHER: &str = "fletcher";

pub const BLOCK_SIZE: Integer = 4 * 1024;
const FORMAT_WRITE_MIN: Integer = 2;
const FORMAT_WRITE_MAX: Integer = 2;
const FORMAT_READ_MIN: Integer = 2;
const FORMAT_READ_MAX: Integer = 2;

/// This designates the "last stored" version for a store which was just open for the first time.
const INITIAL_VERSION: Long = -1;


/// Store is about to close now, but is still operational.<br>
/// Outstanding store operation by background writer or other thread may be in progress.<br>
/// New updates must not be initiated, unless they are part of a closing procedure itself.
const STATE_STOPPING: Integer = 1;

pub type MVStoreRef = Option<Arc<H2RustCell<MVStore>>>;
pub type MVStoreWeakPtr = Weak<H2RustCell<MVStore>>;

#[derive(Default)]
pub struct MVStore {
    recovery_mode: bool,
    compression_level: Integer,
    file_store_shall_be_closed: bool,
    file_store: FileStoreRef,

    page_cache: Option<CacheLongKeyLIRS<PageTraitSharedPtr>>,
    chunk_cache: Option<CacheLongKeyLIRS<Option<Arc<Vec<Long>>>>>,

    pg_split_size: Integer,

    pub keys_per_page: Integer,

    /// The layout map, Contains chunks metadata and root locations for all maps
    /// This is relatively fast changing part of metadata
    layout: MVMapSharedPtr,

    current_version: AtomicI64,

    retention_time: Integer,

    auto_commit_memory: Integer,
    auto_compact_fill_rate: Integer,

    /// Lock which governs access to major store operations: store(), close(), ...
    /// It serves as a replacement for synchronized(this), except it allows for non-blocking lock attempt
    store_lock: MyMutex<()>,
    serialization_lock: MyMutex<()>,
    save_chunk_lock: MyMutex<()>,

    creation_time: Long,

    store_header: HashMap<String, Box<dyn Any + Send + Sync>>,
    last_chunk: AtomicCell<ChunkSharedPtr>,
    chunkId_chunk: DashMap<Integer, ChunkSharedPtr>,
    last_chunk_id: Integer,
    last_map_id: AtomicI32,

    state: AtomicI32,
}

impl MVStore {
    pub fn new(config: &mut HashMap<String, Box<dyn Any>>) -> Result<MVStoreRef> {
        let mv_store_ref = build_option_arc_h2RustCell!(MVStore::default());
        Self::init(mv_store_ref.clone(), config)?;

        Ok(mv_store_ref)
    }

    fn init(mvStoreSharedPtr: MVStoreRef, config: &mut HashMap<String, Box<dyn Any>>) -> Result<()> {
        // 为什么要区分this 和 this_mut的原因是 this.store_lock.lock() 然后调用 this_mut.set_last_chunk(None) 会报错在可变引用的时候进行不可变引用
        let mvStoreRef = get_ref!(mvStoreSharedPtr);
        let mvStoreMutRef = get_ref_mut!(mvStoreSharedPtr);

        mvStoreMutRef.recovery_mode = config.contains_key("recoveryMode");
        mvStoreMutRef.compression_level = data_utils::get_config_int_param(&config, "compress", 0);
        let file_name = h2_rust_utils::get_from_map::<String>(config, "fileName");

        let mut file_store_shall_be_open = false;
        if file_name.is_some() {
            mvStoreMutRef.file_store = FileStore::new()?;
            file_store_shall_be_open = true;
        }
        mvStoreMutRef.file_store_shall_be_closed = true;

        // cache体系
        let mut pgSplitSize = 48; // for "mem:" case it is # of keys
        let mut page_cache_config: Option<CacheLongKeyLIRSConfig> = None;
        let mut chunk_cache_config: Option<CacheLongKeyLIRSConfig> = None;
        if mvStoreMutRef.file_store.is_some() {
            let cache_size = data_utils::get_config_int_param(config, "cacheSize", 16);
            if cache_size > 0 {
                page_cache_config = Some(CacheLongKeyLIRSConfig::new());
                page_cache_config.as_mut().unwrap().max_memory = cache_size as Long * 1024 * 1024;
                let o = h2_rust_utils::get_from_map::<Integer>(config, "cacheConcurrency");
                if o.is_some() {
                    page_cache_config.as_mut().unwrap().segment_count = *o.as_ref().unwrap();
                }
            }
            chunk_cache_config = Some(CacheLongKeyLIRSConfig::new());
            chunk_cache_config.as_mut().unwrap().max_memory = 1024 * 1024;
            pgSplitSize = 16 * 1024;
        }
        if page_cache_config.is_some() {
            mvStoreMutRef.page_cache = Some(CacheLongKeyLIRS::new(&page_cache_config.unwrap()));
        }
        if chunk_cache_config.is_some() {
            mvStoreMutRef.chunk_cache = Some(CacheLongKeyLIRS::new(&chunk_cache_config.unwrap()));
        }

        pgSplitSize = data_utils::get_config_int_param(config, "pageSplitSize", pgSplitSize);
        if mvStoreMutRef.page_cache.is_some() {
            let max_item_size = mvStoreMutRef.page_cache.as_ref().unwrap().get_max_item_size() as Integer;
            if pgSplitSize > max_item_size {
                pgSplitSize = max_item_size;
            }
        }
        mvStoreMutRef.pg_split_size = pgSplitSize;
        mvStoreMutRef.keys_per_page = data_utils::get_config_int_param(config, "keysPerPage", 48);
        //backgroundExceptionHandler = (UncaughtExceptionHandler) config.get("backgroundExceptionHandler");

        mvStoreMutRef.layout = MVMap::new(Arc::downgrade(mvStoreSharedPtr.as_ref().unwrap()),
                                          0,
                                          string_data_type::INSTANCE.clone(),
                                          string_data_type::INSTANCE.clone())?;

        if mvStoreMutRef.file_store.is_some() {
            mvStoreMutRef.retention_time = h2_rust_cell_call!(mvStoreMutRef.file_store, get_default_retention_time);

            // 19 KB memory is about 1 KB storage
            let mut kb = Integer::max(1, Integer::min(19, utils::scale_for_available_memory(64))) * 1024;
            kb = data_utils::get_config_int_param(config, "autoCommitBufferSize", kb);
            mvStoreMutRef.auto_commit_memory = kb * 1024;

            mvStoreMutRef.auto_compact_fill_rate = data_utils::get_config_int_param(config, "autoCompactFillRate", 90);
            let encryption_key = config.remove("encryptionKey");

            // there is no need to lock store here, since it is not opened (or even created) yet,
            // just to make some assertions happy, when they ensure single-threaded access
            let store_lock_guard = mvStoreRef.store_lock.lock();

            {
                let save_chunk_guard = mvStoreRef.save_chunk_lock.lock();

                if file_store_shall_be_open {
                    let read_only = config.contains_key("readOnly");

                    let file_name = file_name.unwrap();
                    let encryption_key = h2_rust_utils::cast::<Vec<Byte>>(encryption_key);
                    h2_rust_cell_mut_call!(mvStoreMutRef.file_store, open, &file_name, read_only, encryption_key)?;
                }

                if h2_rust_cell_call!(mvStoreMutRef.file_store, size) == 0 {
                    mvStoreMutRef.creation_time = h2_rust_utils::get_timestamp();

                    mvStoreMutRef.store_header.insert(HDR_H.to_string(), Box::new(2));
                    mvStoreMutRef.store_header.insert(HDR_BLOCK_SIZE.to_string(), Box::new(BLOCK_SIZE));
                    mvStoreMutRef.store_header.insert(HDR_FORMAT.to_string(), Box::new(FORMAT_WRITE_MAX));
                    mvStoreMutRef.store_header.insert(HDR_CREATED.to_string(), Box::new(mvStoreMutRef.creation_time));

                    mvStoreMutRef.set_last_chunk(None);
                }
            }
        }

        Ok(())
    }

    pub fn get_current_version(&self) -> Long {
        self.current_version.load(Ordering::Acquire)
    }

    fn set_last_chunk(&mut self, last_chunk: ChunkSharedPtr) {
        self.last_chunk.store(last_chunk.clone());
        self.chunkId_chunk.clear();
        self.last_chunk_id = 0;
        self.current_version.store(self.last_chunk_version(), Ordering::Release);

        let mut layout_root_pos: Long = 0;
        let mut map_id: Integer = 0;

        if last_chunk.is_some() { // there is a valid chunk
            self.last_chunk_id = get_ref!(last_chunk).id;
            self.current_version.store(get_ref!(last_chunk).version, Ordering::Release);
            layout_root_pos = get_ref!(last_chunk).layoutRootPos;
            map_id = get_ref!(last_chunk).map_id;
            self.chunkId_chunk.insert(get_ref!(last_chunk).id, last_chunk);
        }

        self.last_map_id.store(map_id, Ordering::Release);
        get_ref!(self.layout).set_root_pos(layout_root_pos,
                                           self.current_version.load(Ordering::Acquire) - 1,
                                           self.layout.clone());
    }

    fn last_chunk_version(&self) -> Long {
        let chunk_ref = unsafe { &*self.last_chunk.as_ptr() };
        if chunk_ref.is_none() {
            INITIAL_VERSION + 1
        } else {
            get_ref!(chunk_ref).version
        }
    }

    pub fn read_page(&mut self, mvMap: MVMapSharedPtr, position: Long) -> Result<PageTraitSharedPtr> {
        if !data_utils::is_page_saved(position) { // position不能是0
            throw!(DbError::get_internal_error("ERROR_FILE_CORRUPT,Position 0"))
        }

        let mut page_ref = self.read_page_from_cache(position);
        if page_ref.is_none() {
            let chunkSharedPtr = self.get_chunk(position)?;
            let pageOffset = data_utils::getPageOffset(position);

            let mut byteBuffer = get_ref!(chunkSharedPtr).readBufferForPage(self.file_store.clone(), pageOffset, position)?;

            page_ref = page::readFromByteBuffer(&mut byteBuffer, position, mvMap);
        }

        todo!()
    }

    fn read_page_from_cache(&mut self, position: Long) -> PageTraitSharedPtr {
        if self.page_cache.is_none() {
            None
        } else {
            self.page_cache.as_mut().unwrap().get(position)
        }
    }

    fn get_chunk(&mut self, position: Long) -> Result<ChunkSharedPtr> {
        let chunk_id = data_utils::getPageChunkId(position);

        let pair = self.chunkId_chunk.get(&chunk_id);

        if pair.is_none() || pair.as_ref().unwrap().value().is_none() {
            self.check_open()?;

            let s = get_ref!(self.layout).get(&H2RustType::String(build_arc_h2RustCell!(chunk::get_meta_key(chunk_id))));
            if s.isNull() {
                let error_code = store::data_utils_error_code_2_error_code(data_utils::ERROR_CHUNK_NOT_FOUND);
                throw!(DbError::get(error_code,vec![&format!("Chunk {} not found",chunk_id)]));
            }

            let chunk = chunk::fromString(s.castAsStringRef())?;
            if !get_ref!(chunk).isSaved() {
                throw!( DbError::get(error_code::FILE_CORRUPTED_1, vec![&format!("chunk {} is invalid",chunk_id)]));
            }

            self.chunkId_chunk.insert(get_ref!(chunk).id, chunk.clone());
            Ok(chunk)
        } else {
            Ok(pair.unwrap().value().clone())
        }
    }

    fn check_open(&self) -> Result<()> {
        if !self.is_open_or_stopping() {
            let error_code = store::data_utils_error_code_2_error_code(data_utils::ERROR_CLOSED);
            throw!(DbError::get(error_code,vec![]));
        }

        Ok(())
    }

    fn is_open_or_stopping(&self) -> bool {
        self.state.load(Ordering::Acquire) <= STATE_STOPPING
    }
}

#[derive(Default)]
pub struct MVStoreBuilder {
    pub config: HashMap<String, Box<dyn Any>>,
}

impl MVStoreBuilder {
    pub fn new() -> Self {
        MVStoreBuilder::default()
    }

    pub fn file_name(&mut self, file_name: &str) {
        self.config.insert("fileName".to_string(), Box::new(file_name.to_string()));
    }

    pub fn page_split_size(&mut self, page_split_size: Integer) {
        self.config.insert("pageSplitSize".to_string(), Box::new(page_split_size));
    }

    pub fn read_only(&mut self) {
        self.config.insert("readOnly".to_string(), Box::new(1));
    }

    pub fn auto_commit_disabled(&mut self) {
        self.config.insert("autoCommitDelay".to_string(), Box::new(0));
    }

    pub fn auto_compact_fill_rate(&mut self, value: Integer) {
        self.config.insert("autoCompactFillRate".to_string(), Box::new(value));
    }

    pub fn compress(&mut self) {
        self.config.insert("compress".to_string(), Box::new(1));
    }

    pub fn open(&mut self) -> Result<MVStoreRef> {
        MVStore::new(&mut self.config)
    }
}
