use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{h2_rust_utils, Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::mvstore::cache::cache_long_key_lirs::{CacheLongKeyLIRS, CacheLongKeyLIRSConfig};
use crate::mvstore::data_utils;
use crate::mvstore::file_store::{FileStore, FileStoreRef};
use crate::mvstore::mv_map::MVMap;
use crate::mvstore::page::Page;

#[derive(Default)]
pub struct MVStore {
    recovery_mode: bool,
    compression_level: Integer,
    file_store_shall_be_closed: bool,
    file_store: FileStoreRef,
    page_cache: Nullable<CacheLongKeyLIRS<Page<Nullable<Arc<dyn Any + Sync + Send>>, Nullable<Arc<dyn Any + Sync + Send>>>>>,
    chunk_cache: Nullable<CacheLongKeyLIRS<Vec<Long>>>,

    pg_split_size: Integer,
    keys_per_page: Integer,
}

pub type MVStoreRef = Nullable<Arc<AtomicRefCell<MVStore>>>;

impl MVStore {
    pub fn new(config: &HashMap<String, Box<dyn Any>>) -> Result<MVStoreRef> {
        let this = NotNull(Arc::new(AtomicRefCell::new(MVStore::default())));
        Self::init(this.clone(), config)?;

        Ok(this)
    }

    fn init(mv_store_ref: MVStoreRef, config: &HashMap<String, Box<dyn Any>>) -> Result<()> {
        let mut this_atomic_ref_mut = mv_store_ref.unwrap().borrow_mut();
        let this = this_atomic_ref_mut.deref_mut();

        this.recovery_mode = config.contains_key("recoveryMode");
        this.compression_level = data_utils::get_config_int_param(&config, "compress", 0);
        let file_name = h2_rust_utils::get_from_map::<String>(config, "fileName");

        let mut file_store_shall_be_open = false;
        if !file_name.is_null() {
            this.file_store = FileStore::new()?;
            file_store_shall_be_open = true;
        }
        this.file_store_shall_be_closed = true;

        // cache体系
        let mut pg_split_size = 48; // for "mem:" case it is # of keys
        let mut page_cache_config: Nullable<CacheLongKeyLIRSConfig> = Null;
        let mut chunk_cache_config: Nullable<CacheLongKeyLIRSConfig> = Null;
        if !this.file_store.is_null() {
            let cache_size = data_utils::get_config_int_param(config, "cacheSize", 16);
            if cache_size > 0 {
                page_cache_config = NotNull(CacheLongKeyLIRSConfig::new());
                page_cache_config.unwrap_mut().max_memory = cache_size as Long * 1024 * 1024;
                let o = h2_rust_utils::get_from_map::<Integer>(config, "cacheConcurrency");
                if !o.is_null() {
                    page_cache_config.unwrap_mut().segment_count = *o.unwrap();
                }
            }
            chunk_cache_config = NotNull(CacheLongKeyLIRSConfig::new());
            chunk_cache_config.unwrap_mut().max_memory = 1024 * 1024;
            pg_split_size = 16 * 1024;
        }
        if page_cache_config.is_not_null() {
            this.page_cache = NotNull(CacheLongKeyLIRS::new(&page_cache_config.unwrap()));
        }
        if chunk_cache_config.is_not_null() {
            this.chunk_cache = NotNull(CacheLongKeyLIRS::new(&chunk_cache_config.unwrap()));
        }

        pg_split_size = data_utils::get_config_int_param(config, "pageSplitSize", pg_split_size);
        if this.page_cache.is_not_null() {
            let max_item_size = this.page_cache.unwrap().get_max_item_size() as Integer;
            if pg_split_size > max_item_size {
                pg_split_size = max_item_size;
            }
        }
        this.pg_split_size = pg_split_size;
        this.keys_per_page = data_utils::get_config_int_param(config, "keysPerPage", 48);
        //backgroundExceptionHandler = (UncaughtExceptionHandler) config.get("backgroundExceptionHandler");

        //layout = new MVMap<>(this, 0, StringDataType.INSTANCE, StringDataType.INSTANCE);


        Ok(())
    }

    pub fn read_page<K, V>(&self, mv_map: MVMap<K, V>, pos: Long) {
        //  pageCache.put(page.getPos(), page, page.get_memory());
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
