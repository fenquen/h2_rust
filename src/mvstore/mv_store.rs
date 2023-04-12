use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{h2_rust_utils, Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::mvstore::cache::cache_long_key_lirs::{CacheLongKeyLIRS, CacheLongKeyLIRSConfig};
use crate::mvstore::data_utils;
use crate::mvstore::file_store::{FileStore, FileStoreRef};
use crate::mvstore::mv_map::{MVMap, MVMapRef};
use crate::mvstore::page::Page;
use crate::mvstore::r#type::string_data_type;

#[derive(Default)]
pub struct MVStore {
    recovery_mode: bool,
    compression_level: Integer,
    file_store_shall_be_closed: bool,
    file_store: FileStoreRef,
    page_cache: Option<CacheLongKeyLIRS<Option<Arc<Page<Arc<dyn Any + Sync + Send>, Arc<dyn Any + Sync + Send>>>>>>,
    chunk_cache: Option<CacheLongKeyLIRS<Option<Arc<Vec<Long>>>>>,

    pg_split_size: Integer,
    pub keys_per_page: Integer,

    layout: MVMapRef<String, String>,

    current_version: AtomicI64,
}

pub type MVStoreRef = Option<Arc<AtomicRefCell<MVStore>>>;

impl MVStore {
    pub fn new(config: &HashMap<String, Box<dyn Any>>) -> Result<MVStoreRef> {
        let this = Some(Arc::new(AtomicRefCell::new(MVStore::default())));
        Self::init(this.clone(), config)?;

        Ok(this)
    }

    fn init(mv_store_ref: MVStoreRef, config: &HashMap<String, Box<dyn Any>>) -> Result<()> {
        let mut this_atomic_ref_mut = mv_store_ref.as_ref().unwrap().borrow_mut();
        let this = this_atomic_ref_mut.deref_mut();

        this.recovery_mode = config.contains_key("recoveryMode");
        this.compression_level = data_utils::get_config_int_param(&config, "compress", 0);
        let file_name = h2_rust_utils::get_from_map::<String>(config, "fileName");

        let mut file_store_shall_be_open = false;
        if file_name.is_some() {
            this.file_store = FileStore::new()?;
            file_store_shall_be_open = true;
        }
        this.file_store_shall_be_closed = true;

        // cache体系
        let mut pg_split_size = 48; // for "mem:" case it is # of keys
        let mut page_cache_config: Option<CacheLongKeyLIRSConfig> = None;
        let mut chunk_cache_config: Option<CacheLongKeyLIRSConfig> = None;
        if this.file_store.is_some() {
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
            pg_split_size = 16 * 1024;
        }
        if page_cache_config.is_some() {
            this.page_cache = Some(CacheLongKeyLIRS::new(&page_cache_config.unwrap()));
        }
        if chunk_cache_config.is_some() {
            this.chunk_cache = Some(CacheLongKeyLIRS::new(&chunk_cache_config.unwrap()));
        }

        pg_split_size = data_utils::get_config_int_param(config, "pageSplitSize", pg_split_size);
        if this.page_cache.is_some() {
            let max_item_size = this.page_cache.as_ref().unwrap().get_max_item_size() as Integer;
            if pg_split_size > max_item_size {
                pg_split_size = max_item_size;
            }
        }
        this.pg_split_size = pg_split_size;
        this.keys_per_page = data_utils::get_config_int_param(config, "keysPerPage", 48);
        //backgroundExceptionHandler = (UncaughtExceptionHandler) config.get("backgroundExceptionHandler");

        this.layout = MVMap::new(mv_store_ref.clone(),
                                 0,
                                 string_data_type::INSTANCE.clone(),
                                 string_data_type::INSTANCE.clone())?;

        Ok(())
    }

    pub fn read_page<K, V>(&self, mv_map: MVMap<K, V>, pos: Long) {
        //  pageCache.put(page.getPos(), page, page.get_memory());
    }

    pub fn get_current_version(&self) -> Long {
        self.current_version.load(Ordering::Acquire)
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

    pub fn open(&self) -> Result<MVStoreRef> {
        MVStore::new(&self.config)
    }
}
