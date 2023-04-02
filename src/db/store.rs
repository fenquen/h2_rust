use anyhow::Result;
use std::sync::Arc;
use atomic_refcell::AtomicRefCell;
use crate::engine::database::Database;
use crate::h2_rust_common::Nullable;
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::mv_store::MVStoreBuilder;

#[derive(Default)]
pub struct Store {}


impl Store {
    pub fn new(database: Arc<AtomicRefCell<Nullable<Database>>>, encryption_key: Arc<Nullable<Vec<u8>>>) -> Result<()> {
        let this = Arc::new(AtomicRefCell::new(NotNull(Store::default())));
        Self::init(this, database, encryption_key)?;
        Ok(())
    }

    pub fn init(this: Arc<AtomicRefCell<Nullable<Store>>>,
                database: Arc<AtomicRefCell<Nullable<Database>>>,
                encryption_key: Arc<Nullable<Vec<u8>>>) -> Result<()> {
        let atomic_ref_mut = (&*database).borrow();
        let database = atomic_ref_mut.unwrap();

        let database_path = database.get_database_path()?;
        let mv_store_builder = MVStoreBuilder::new();
        let encrypted = false;

        if !database_path.is_empty() {

        }

        Ok(())
    }
}