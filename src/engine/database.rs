use std::cell::RefCell;
use std::fmt::format;
use std::ops::Add;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::db_settings::DbSettings;
use anyhow::Result;
use crate::api::error_code;
use crate::engine::{constant, database};
use crate::engine::mode::Mode;
use crate::h2_rust_common::{h2_rust_constant, Integer, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::message::db_error::DbError;
use crate::mode::default_null_ordering::DefaultNullOrdering;
use crate::store::{file_lock, file_lock_method};
use crate::store::file_lock_method::FileLockMethod;
use crate::store::fs::encrypt::file_encrypt;
use crate::store::fs::file_utils;
use crate::throw;
use crate::util::string_utils;

#[derive(Default)]
pub struct Database {
    db_settings: DbSettings,
    persistent: bool,
    file_password_hash: Arc<Nullable<Vec<u8>>>,
    database_path: String,
    max_length_inplace_lob: Integer,
    cipher: String,
    auto_server_mode: bool,
    auto_server_port: Integer,
    page_size: Integer,
    database_short_name: String,
    read_only: bool,
    file_lock_method: FileLockMethod,
    database_url: String,
    mode: Nullable<&'static Mode>,
    default_null_ordering: Nullable<&'static DefaultNullOrdering>,
    cache_type: String,
    ignore_catalogs: bool,
    lock_mode: Integer,
    starting: AtomicBool,
}

impl Database {
    pub fn new(connection_info: &mut ConnectionInfo, cipher: &String) {
        let database: Database = Default::default();
        let this = Arc::new(RefCell::new(NotNull(database)));
        Self::init(this, connection_info, cipher);
    }

    fn init(this: Arc<RefCell<Nullable<Database>>>, connection_info: &mut ConnectionInfo, cipher: &String) -> Result<()> {
        let mut binding = this.borrow_mut();
        let database = binding.unwrap_mut();

        database.db_settings = connection_info.get_db_settings()?;
        database.persistent = connection_info.persistent;

        database.file_password_hash = connection_info.file_password_hash.clone();
        database.database_path = connection_info.get_database_path()?;
        database.max_length_inplace_lob = constant::DEFAULT_MAX_LENGTH_INPLACE_LOB;
        database.cipher = cipher.clone();
        database.auto_server_mode = connection_info.get_property_bool("AUTO_SERVER", false)?;
        database.auto_server_port = connection_info.get_property_int("AUTO_SERVER_PORT", 0)?;
        database.page_size = connection_info.get_property_int("PAGE_SIZE", constant::DEFAULT_PAGE_SIZE)?;

        database.database_short_name = Self::parse_database_short_name(this.clone());

        if !database.cipher.is_empty() && database.page_size % file_encrypt::BLOCK_SIZE != 0 {
            throw!( DbError::get_unsupported_exception(&format!("CIPHER && PAGE_SIZE={}",  database.page_size)));
        }

        let access_mode_data = string_utils::to_lower_english(&connection_info.get_property_string("ACCESS_MODE_DATA", "rw"));
        if "r".eq(&access_mode_data) {
            database.read_only = true;
        }

        let lock_method_name = connection_info.get_property_string("FILE_LOCK", h2_rust_constant::EMPTY_STR);
        database.file_lock_method =
            if !lock_method_name.is_empty() {
                file_lock::get_file_lock_method(&lock_method_name)?
            } else {
                if database.auto_server_mode {
                    file_lock_method::FILE
                } else {
                    file_lock_method::FS
                }
            };

        database.database_url = connection_info.url.clone();

        database.mode = Nullable::from(Mode::get_regular());
        let s = connection_info.remove_property_str("MODE", h2_rust_constant::EMPTY_STR);
        if !s.is_empty() {
            database.mode = Nullable::from(Mode::get_instance(&s));
            if database.mode.is_null() {
                throw!(DbError::get(error_code::UNKNOWN_MODE_1, vec![&s]));
            }
        }

        let s = connection_info.remove_property_str("DEFAULT_NULL_ORDERING", h2_rust_constant::EMPTY_STR);
        if !s.is_empty() {
            let default_null_ordering = DefaultNullOrdering::value_of(&string_utils::to_upper_english(&s));
            match default_null_ordering {
                Some(d) => {
                    database.default_null_ordering = NotNull(d);
                }
                None => {
                    throw!(DbError::get_invalid_value_exception("DEFAULT_NULL_ORDERING", &s));
                }
            }
        }

        let allow_builtin_alias_override = connection_info.get_property_bool("BUILTIN_ALIAS_OVERRIDE", false)?;

        let close_at_vm_shutdown = database.db_settings.db_close_on_exit;
        if database.auto_server_mode && !close_at_vm_shutdown {
            throw!(DbError::get_unsupported_exception("AUTO_SERVER=TRUE && DB_CLOSE_ON_EXIT=FALSE"));
        }

        database.cache_type = string_utils::to_upper_english(&connection_info.remove_property_str("CACHE_TYPE", constant::CACHE_TYPE_DEFAULT));
        database.ignore_catalogs = connection_info.get_property_bool("IGNORE_CATALOGS", database.db_settings.ignore_catalogs)?;
        database.lock_mode = connection_info.get_property_int("LOCK_MODE", constant::DEFAULT_LOCK_MODE)?;

        {
            if database.auto_server_mode &&
                (database.read_only || !database.persistent || database.file_lock_method == file_lock_method::NO || database.file_lock_method == file_lock_method::FS) {
                throw!(DbError::get_unsupported_exception("AUTO_SERVER=TRUE && (readOnly || inMemory || FILE_LOCK=NO || FILE_LOCK=FS)"));
            }

            if database.persistent {
                let lock_file_name = database.database_path.clone().add(constant::SUFFIX_LOCK_FILE);
                if database.read_only {
                    if file_utils::exist(&lock_file_name) {
                        throw!( DbError::get(error_code::DATABASE_ALREADY_OPEN_1, vec![&format!("Lock file exists: {}" , lock_file_name)]));
                    }
                } else if database.file_lock_method != file_lock_method::NO && database.file_lock_method != file_lock_method::FS {
                    todo!()
                    /*lock = new FileLock(traceSystem, lock_file_name, Constants.LOCK_SLEEP);
                    lock.lock(fileLockMethod);
                    if autoServerMode {
                        startServer(lock.getUniqueId());
                    }*/
                }

                Self::delete_old_temp_files(this.clone())?;
            }

            database.starting = AtomicBool::new(true);

            if !database.db_settings.mv_store {
                throw!( DbError::get(error_code::GENERAL_ERROR_1,vec!["mv store not enabled"]));
            }

            //store = new Store(this, connectionInfo.fileEncryptionKey);
        }

        Ok(())
    }

    fn parse_database_short_name(this: Arc<RefCell<Nullable<Database>>>) -> String {
        let mut binding = this.borrow_mut();
        let database = binding.unwrap_mut();

        let database_path = &database.database_path;
        let len = database_path.len();

        let a =
            if let Some(i) = database_path.rfind(|c| c == '/' || c == '\\' || c == ':') {
                i as Integer
            } else {
                -1
            };

        let a = (a + 1) as usize;
        let database_path =
            if a == len {
                "UNNAMED"
            } else {
                &database_path[a..]
            };

        let database_path =
            if database.db_settings.database_to_upper {
                string_utils::to_upper_english(database_path)
            } else {
                if database.db_settings.database_to_lower {
                    string_utils::to_lower_english(database_path)
                } else {
                    database_path.to_string()
                }
            };

        string_utils::truncate_string(&database_path, constant::MAX_IDENTIFIER_LENGTH as usize)
    }

    fn delete_old_temp_files(this: Arc<RefCell<Nullable<Database>>>) -> Result<()> {
        let mut binding = this.borrow_mut();
        let database = binding.unwrap_mut();

        let parent_dir_path = file_utils::get_parent(&database.database_path)?;

        for path in file_utils::new_directory_stream(parent_dir_path)? {
            if path.ends_with(constant::SUFFIX_TEMP_FILE) && path.starts_with(&database.database_path) {
                file_utils::try_delete(&path);
            }
        }

        Ok(())
    }
}