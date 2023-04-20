use std::ops::{Add, Deref, DerefMut};
use anyhow::Result;
use std::sync::Arc;
use crate::engine::constant;
use crate::engine::database::{Database, DatabaseRef};
use crate::{build_h2_rust_cell, get_ref, get_ref_mut};
use crate::h2_rust_common::{Nullable, VecRef};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::{MVStoreBuilder, MVStoreRef};
use crate::mvstore::mv_store_tool;
use crate::store::fs::file_utils;

#[derive(Default)]
pub struct Store {
    mv_file_path: String,
    encrypted: bool,
    mv_store: MVStoreRef,
}

pub type StoreRef = Option<Arc<H2RustCell<Store>>>;

impl Store {
    pub fn new(database_ref: DatabaseRef, encryption_key: VecRef<u8>) -> Result<StoreRef> {
        let store_ref = build_h2_rust_cell!(Store::default());
        Self::init(store_ref.clone(), database_ref.clone(), encryption_key)?;
        Ok(store_ref)
    }

    pub fn init(store_ref: StoreRef,
                database_ref: DatabaseRef,
                encryption_key: VecRef<u8>) -> Result<()> {
        let this = get_ref_mut!(store_ref);

        let database = get_ref!(database_ref);

        let database_path = database.get_database_path()?;
        let mut mv_store_builder = MVStoreBuilder::new();
        let mut encrypted = false;

        if !database_path.is_empty() {
            this.mv_file_path = database_path.add(constant::SUFFIX_MV_FILE);
            let mv_file_path = &this.mv_file_path;

            mv_store_tool::compact_clean_up(mv_file_path)?;
            mv_store_builder.file_name(mv_file_path);
            mv_store_builder.page_split_size(database.page_size);

            if database.read_only {
                mv_store_builder.read_only();
            } else {
                let exist = file_utils::exist(mv_file_path);

                if exist && file_utils::can_write(mv_file_path) {
                    // read only
                } else {
                    file_utils::create_directories(file_utils::get_parent(mv_file_path)?)?;
                }

                let auto_compact_fill_rate = database.db_settings.auto_compact_fill_rate;
                if auto_compact_fill_rate <= 100 {
                    mv_store_builder.auto_compact_fill_rate(auto_compact_fill_rate);
                }
            }

            if encryption_key.is_some() {
                encrypted = true;
                // mvStoreBuilder.encryptionKey(decodePassword(encryptionKey));
            }

            if database.db_settings.compress_data {
                mv_store_builder.compress();
                // use a larger page split size to improve the compression ratio
                mv_store_builder.page_split_size(64 * 1024);
            }

            mv_store_builder.auto_commit_disabled();
        }

        this.encrypted = encrypted;

        this.mv_store = mv_store_builder.open()?;

        Ok(())
    }
}