use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use atomic_refcell::AtomicRefCell;
use crate::h2_rust_common::{Byte, Integer, Long, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use std::{fs, i64};
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;
use crate::api::error_code;
use crate::h2_rust_common::file_lock::FileLock;
use crate::message::db_error::DbError;
use crate::store::fs::file_utils;
use crate::throw;

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
    file: Option<File>,

    file_lock: Option<FileLock>,
}

pub type FileStoreRef = Option<Arc<AtomicRefCell<FileStore>>>;

impl FileStore {
    pub fn open(&mut self, file_name: &str, mut read_only: bool, encryption_key: Option<Box<Vec<Byte>>>) -> Result<()> {
        if self.file.is_some() {
            return Ok(());
        }

        self.file_name = file_name.to_string();

        let file_path = Path::new(file_name);
        let parent_dir_path = file_path.parent();
        if parent_dir_path.is_some() && !parent_dir_path.unwrap().exists() {
            throw!(DbError::get_internal_error("parent dir not exist"));
        }

        if file_path.exists() && !file_utils::can_write(file_path) {
            read_only = true;
        }

        let mut open_options = OpenOptions::new();
        open_options.read(true);

        if !read_only {
            open_options.create(true);
            open_options.write(true);
            open_options.append(true);
        }

        match open_options.open::<&Path>(file_name.as_ref()) {
            Ok(file) => {
                self.file = Some(file)
            }
            Err(e) => {
                self.close();
                throw!( DbError::get_internal_error("ERROR_READING_FAILED,Could not open file"));
            }
        }

        let file = self.file.as_ref().unwrap();
        let fd = file.as_raw_fd();
        let file_lock_result = if read_only {
            FileLock::try_lock(fd, 0, i64::MAX, true)
        } else {
            FileLock::try_lock(fd, 0, i64::MAX, false)
        };
        match file_lock_result {
            Ok(file_lock) => {
                self.file_lock = Some(file_lock);
            }
            Err(e) => {
                self.close();

                let message = format!("try lock file failed,cause: {}", e);
                throw!(DbError::get_internal_error(&message));
            }
        }

        self.file_size = file_utils::get_size(&self.file_name)?;

        Ok(())
    }


    pub fn new() -> Result<FileStoreRef> {
        Ok(Some(Arc::new(AtomicRefCell::new(FileStore::default()))))
    }

    pub fn close(&mut self) {
        if self.file.is_some() {
            if self.file_lock.is_some() {
                self.file_lock.as_ref().unwrap().release();
                self.file_lock = None;
            }

            self.file = None;
        }
    }

    pub fn get_default_retention_time(&self) -> Integer {
        45000
    }
}

