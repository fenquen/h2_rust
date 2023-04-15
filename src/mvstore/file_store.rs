use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Byte, Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use std::fs;
use std::fs::{File, OpenOptions};

#[derive(Default)]
pub struct FileStore {
    /// The number of read operations.
    read_count: AtomicU64,
    /// The number of read bytes.
    read_byte_count: AtomicU64,
    /// The number of write operations.
    write_count: AtomicU64,
    /// The number of written bytes.
    write_byte_count: AtomicU64,
    file_name: String,
    read_only: bool,
    file_size: Long,
}

pub type FileStoreRef = Option<Arc<AtomicRefCell<FileStore>>>;

impl FileStore {
    pub fn open(&self, file_name: &str, read_only: bool, encryption_key: Option<Vec<Byte>>) {

    }

    pub fn new() -> Result<FileStoreRef> {
        Ok(Some(Arc::new(AtomicRefCell::new(FileStore::default()))))
    }

    pub fn get_default_retention_time(&self) -> Integer {
        45000
    }
}

