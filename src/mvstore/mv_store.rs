use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicPtr, Ordering};
use std::thread;
use std::time::{Instant, SystemTime};
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use crate::h2_rust_common::{Byte, h2_rust_constant, h2_rust_utils, Integer, Long, MyMutex, Optional};
use crate::mvstore::cache::cache_long_key_lirs::{CacheLongKeyLIRS, CacheLongKeyLIRSConfig};
use crate::mvstore::{chunk, data_utils, mv_map, page};
use crate::mvstore::file_store::{FileStore};
use crate::mvstore::mv_map::{MVMap};
use crate::mvstore::page::{Page, PageTrait};
use crate::mvstore::r#type::string_data_type;
use crate::{
    atomic_ref_cell,
    atomic_ref_cell_mut,
    get_ref_mut,
    build_option_arc_h2RustCell,
    get_ref,
    throw,
    build_arc_h2RustCell};
use crate::api::error_code;
use crate::db::store;
use crate::h2_rust_common::h2_rust_cell::{H2RustCell, SharedPtr, WeakPtr};
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::message::db_error;
use crate::message::db_error::DbError;
use crate::mvstore::chunk::{Chunk};
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

#[derive(Default)]
pub struct MVStore {
    recoveryMode: bool,
    compressionLevel: Integer,
    fileStoreShallBeClosed: bool,
    fileStore: SharedPtr<FileStore>,

    pageCache: Option<CacheLongKeyLIRS<SharedPtr<dyn PageTrait>, WeakPtr<dyn PageTrait>>>,
    chunkCache: Option<CacheLongKeyLIRS<SharedPtr<Vec<Long>>, WeakPtr<Vec<Long>>>>,

    pageSplitSize: Integer,

    pub keysPerPage: Integer,

    /// The layout map, Contains chunks metadata and root locations for all maps
    /// This is relatively fast changing part of metadata
    layout: SharedPtr<MVMap>,

    /// The metadata map. Holds name -> id and id -> name and id -> metadata
    /// mapping for all maps. This is relatively slow changing part of metadata
    meta: SharedPtr<MVMap>,

    pub currentVersion: AtomicI64,

    retentionTime: Integer,

    autoCommitMemory: Integer,
    autoCompactFillRate: Integer,

    /// Lock which governs access to major store operations: store(), close(), ...
    /// It serves as a replacement for synchronized(this), except it allows for non-blocking lock attempt
    storeLock: MyMutex<()>,
    serializationLock: MyMutex<()>,
    saveChunkLock: MyMutex<()>,

    creationTime: Long,

    storeHeader: HashMap<String, Box<dyn Any + Send + Sync>>,
    lastChunk: AtomicCell<SharedPtr<Chunk>>,
    chunkId_chunk: DashMap<Integer, SharedPtr<Chunk>>,
    lastChunkId: Integer,
    lastMapId: AtomicI32,

    state: AtomicI32,
    oldestVersionToKeep: AtomicI64,
    versionsToKeep: Integer,
    metaChanged: AtomicBool,
}

impl MVStore {
    pub fn new(config: &mut HashMap<String, Box<dyn Any>>) -> Result<SharedPtr<MVStore>> {
        let mut mvStore = MVStore::default();
        mvStore.versionsToKeep = 5;
        let mv_store_ref = build_option_arc_h2RustCell!(mvStore);
        Self::init(mv_store_ref.clone(), config)?;

        Ok(mv_store_ref)
    }

    fn init(mvStoreSharedPtr: SharedPtr<MVStore>, config: &mut HashMap<String, Box<dyn Any>>) -> Result<()> {
        // 为什么要区分this 和 this_mut的原因是 this.store_lock.lock() 然后调用 this_mut.set_last_chunk(None) 会报错在可变引用的时候进行不可变引用
        let mvStoreRef = get_ref!(mvStoreSharedPtr);
        let mvStoreMutRef = get_ref_mut!(mvStoreSharedPtr);

        mvStoreMutRef.recoveryMode = config.contains_key("recoveryMode");
        mvStoreMutRef.compressionLevel = data_utils::get_config_int_param(&config, "compress", 0);
        let file_name = h2_rust_utils::get_from_map::<String>(config, "fileName");

        let mut file_store_shall_be_open = false;
        if file_name.is_some() {
            mvStoreMutRef.fileStore = FileStore::new()?;
            file_store_shall_be_open = true;
        }
        mvStoreMutRef.fileStoreShallBeClosed = true;

        // cache体系
        let mut pgSplitSize = 48; // for "mem:" case it is # of keys
        let mut pageCacheConfig: Option<CacheLongKeyLIRSConfig> = None;
        let mut chunkCacheConfig: Option<CacheLongKeyLIRSConfig> = None;
        if mvStoreMutRef.fileStore.is_some() {
            let cache_size = data_utils::get_config_int_param(config, "cacheSize", 16);
            if cache_size > 0 {
                pageCacheConfig = Some(CacheLongKeyLIRSConfig::new());
                pageCacheConfig.as_mut().unwrap().max_memory = cache_size as Long * 1024 * 1024;
                let o = h2_rust_utils::get_from_map::<Integer>(config, "cacheConcurrency");
                if o.is_some() {
                    pageCacheConfig.as_mut().unwrap().segment_count = *o.as_ref().unwrap();
                }
            }
            chunkCacheConfig = Some(CacheLongKeyLIRSConfig::new());
            chunkCacheConfig.as_mut().unwrap().max_memory = 1024 * 1024;
            pgSplitSize = 16 * 1024;
        }
        if pageCacheConfig.is_some() {
            mvStoreMutRef.pageCache = Some(CacheLongKeyLIRS::new(&pageCacheConfig.unwrap()));
        }
        if chunkCacheConfig.is_some() {
            mvStoreMutRef.chunkCache = Some(CacheLongKeyLIRS::new(&chunkCacheConfig.unwrap()));
        }

        pgSplitSize = data_utils::get_config_int_param(config, "pageSplitSize", pgSplitSize);
        if mvStoreMutRef.pageCache.is_some() {
            let max_item_size = mvStoreMutRef.pageCache.as_ref().unwrap().get_max_item_size() as Integer;
            if pgSplitSize > max_item_size {
                pgSplitSize = max_item_size;
            }
        }
        mvStoreMutRef.pageSplitSize = pgSplitSize;
        mvStoreMutRef.keysPerPage = data_utils::get_config_int_param(config, "keysPerPage", 48);
        //backgroundExceptionHandler = (UncaughtExceptionHandler) config.get("backgroundExceptionHandler");

        mvStoreMutRef.layout = MVMap::new(Some(Arc::downgrade(mvStoreSharedPtr.as_ref().unwrap())),
                                          0,
                                          string_data_type::INSTANCE.clone(),
                                          string_data_type::INSTANCE.clone())?;

        if mvStoreMutRef.fileStore.is_some() {
            mvStoreMutRef.retentionTime = get_ref!(mvStoreMutRef.fileStore).get_default_retention_time();

            // 19 KB memory is about 1 KB storage
            let mut kb = Integer::max(1, Integer::min(19, utils::scaleForAvailableMemory(64))) * 1024;
            kb = data_utils::get_config_int_param(config, "autoCommitBufferSize", kb);
            mvStoreMutRef.autoCommitMemory = kb * 1024;

            mvStoreMutRef.autoCompactFillRate = data_utils::get_config_int_param(config, "autoCompactFillRate", 90);
            let encryption_key = config.remove("encryptionKey");

            // there is no need to lock store here, since it is not opened (or even created) yet,
            // just to make some assertions happy, when they ensure single-threaded access
            let storeLockGuard = mvStoreRef.storeLock.lock();

            {
                let save_chunk_guard = mvStoreRef.saveChunkLock.lock();

                if file_store_shall_be_open {
                    let read_only = config.contains_key("readOnly");

                    let file_name = file_name.unwrap();
                    let encryption_key = h2_rust_utils::cast::<Vec<Byte>>(encryption_key);
                    get_ref_mut!(mvStoreMutRef.fileStore).open(&file_name, read_only, encryption_key)?;
                }

                if get_ref!(mvStoreMutRef.fileStore).size() == 0 {
                    mvStoreMutRef.creationTime = h2_rust_utils::getTimestamp();

                    mvStoreMutRef.storeHeader.insert(HDR_H.to_string(), Box::new(2));
                    mvStoreMutRef.storeHeader.insert(HDR_BLOCK_SIZE.to_string(), Box::new(BLOCK_SIZE));
                    mvStoreMutRef.storeHeader.insert(HDR_FORMAT.to_string(), Box::new(FORMAT_WRITE_MAX));
                    mvStoreMutRef.storeHeader.insert(HDR_CREATED.to_string(), Box::new(mvStoreMutRef.creationTime));

                    mvStoreMutRef.set_last_chunk(None);
                }
            }
        }

        Ok(())
    }

    pub fn get_current_version(&self) -> Long {
        self.currentVersion.load(Ordering::Acquire)
    }

    fn set_last_chunk(&mut self, last_chunk: SharedPtr<Chunk>) {
        self.lastChunk.store(last_chunk.clone());
        self.chunkId_chunk.clear();
        self.lastChunkId = 0;
        self.currentVersion.store(self.last_chunk_version(), Ordering::Release);

        let mut layout_root_pos: Long = 0;
        let mut map_id: Integer = 0;

        if last_chunk.is_some() { // there is a valid chunk
            self.lastChunkId = get_ref!(last_chunk).id;
            self.currentVersion.store(get_ref!(last_chunk).version, Ordering::Release);
            layout_root_pos = get_ref!(last_chunk).layoutRootPos;
            map_id = get_ref!(last_chunk).map_id;
            self.chunkId_chunk.insert(get_ref!(last_chunk).id, last_chunk);
        }

        self.lastMapId.store(map_id, Ordering::Release);
        get_ref_mut!(self.layout).setRootPosition(layout_root_pos,
                                                  self.currentVersion.load(Ordering::Acquire) - 1,
                                                  self.layout.clone());
    }

    fn last_chunk_version(&self) -> Long {
        let chunk_ref = unsafe { &*self.lastChunk.as_ptr() };
        if chunk_ref.is_none() {
            INITIAL_VERSION + 1
        } else {
            get_ref!(chunk_ref).version
        }
    }

    pub fn readPage(&mut self, mvMap: SharedPtr<MVMap>, position: Long) -> Result<SharedPtr<dyn PageTrait>> {
        if !data_utils::is_page_saved(position) { // position不能是0
            throw!(DbError::get_internal_error("ERROR_FILE_CORRUPT,Position 0"))
        }

        let mut pageTrait = self.read_page_from_cache(position);
        if pageTrait.is_none() {
            let chunkSharedPtr = self.get_chunk(position)?;
            let pageOffset = data_utils::getPageOffset(position);

            let mut byteBuffer = get_ref!(chunkSharedPtr).readBufferForPage(self.fileStore.clone(), pageOffset, position)?;
            pageTrait = page::readFromByteBuffer(&mut byteBuffer, position, mvMap)?;

            self.cachePage(pageTrait.clone())?;
        }

        Ok(pageTrait)
    }

    fn read_page_from_cache(&mut self, position: Long) -> SharedPtr<dyn PageTrait> {
        if self.pageCache.is_none() {
            None
        } else {
            self.pageCache.as_mut().unwrap().get(position)
        }
    }

    fn get_chunk(&mut self, position: Long) -> Result<SharedPtr<Chunk>> {
        let chunk_id = data_utils::getPageChunkId(position);

        let pair = self.chunkId_chunk.get(&chunk_id);

        if pair.is_none() || pair.as_ref().unwrap().value().is_none() {
            self.check_open()?;

            let s = get_ref!(self.layout).get(&H2RustType::String(build_arc_h2RustCell!(chunk::get_meta_key(chunk_id))));
            if s.isNull() {
                let error_code = store::dataUtilsErrorCode2ErrorCode(data_utils::ERROR_CHUNK_NOT_FOUND);
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
            let error_code = store::dataUtilsErrorCode2ErrorCode(data_utils::ERROR_CLOSED);
            throw!(DbError::get(error_code,vec![]));
        }

        Ok(())
    }

    fn is_open_or_stopping(&self) -> bool {
        self.state.load(Ordering::Acquire) <= STATE_STOPPING
    }

    fn cachePage(&mut self, pageTrait: SharedPtr<dyn PageTrait>) -> Result<()> {
        if self.pageCache.is_some() {
            let position = get_ref!(pageTrait).getPosition();
            let memory = get_ref!(pageTrait).getMemory();

            self.pageCache.as_mut().unwrap().put(position, pageTrait, memory)?;
        }

        Ok(())
    }

    pub fn getOldestVersionToKeep(&self) -> Long {
        let mut v = self.oldestVersionToKeep.load(Ordering::Acquire);
        v = Long::max(v - self.versionsToKeep as Long, INITIAL_VERSION);
        if self.fileStore.is_some() {
            let storeVersion = self.lastChunkVersion() - 1;
            if storeVersion != INITIAL_VERSION && storeVersion < v {
                v = storeVersion;
            }
        }
        v
    }

    fn lastChunkVersion(&self) -> Long {
        let chunk = unsafe { &*self.lastChunk.as_ptr() };
        if chunk.is_none() {
            INITIAL_VERSION + 1
        } else {
            get_ref!(chunk).version
        }
    }

    pub fn deregisterMapRoot(&mut self, mapId: Integer) {
        if get_ref!(self.layout).remove(H2RustType::String(build_arc_h2RustCell!(mv_map::getMapRootKey(mapId)))) != null {
            self.markMetaChanged();
        }
    }

    /// changes in the metadata alone are usually not detected, as the meta map is changed after storing
    fn markMetaChanged(&mut self) {
        self.metaChanged.store(true, Ordering::Release);
    }

    fn getMapName(&self, id: Integer) -> Result<Option<String>> {
        // 元信息使用1个string表达
        let h2RustType = get_ref!(self.meta).get(&H2RustType::String(build_arc_h2RustCell!(mv_map::getMapKey(id))));
        if h2RustType.isNull() {
            Ok(None)
        } else {
            data_utils::getMapName(&h2RustType.toString().unwrap())
        }
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

    pub fn fileName(&mut self, file_name: &str) {
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

    pub fn open(&mut self) -> Result<SharedPtr<MVStore>> {
        MVStore::new(&mut self.config)
    }
}
