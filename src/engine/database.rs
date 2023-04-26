use std::cell::RefCell;
use std::fmt::format;
use std::ops::{Add, DerefMut};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::db_settings::DbSettings;
use anyhow::Result;
use crate::api::error_code;
use crate::db::store::{Store, StoreRef};
use crate::engine::{constant, database};
use crate::engine::mode::Mode;
use crate::h2_rust_common::{Byte, h2_rust_constant, Integer, Nullable, VecRef};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
use crate::h2_rust_common::Nullable::NotNull;
use crate::message::db_error::DbError;
use crate::mode::default_null_ordering::DefaultNullOrdering;
use crate::store::{file_lock, file_lock_method};
use crate::store::file_lock_method::FileLockMethod;
use crate::store::fs::encrypt::file_encrypt;
use crate::store::fs::file_utils;
use crate::{build_h2_rust_cell, get_ref_mut, throw};
use crate::util::string_utils;

#[derive(Default)]
pub struct Database {
    pub db_settings: DbSettings,
    persistent: bool,
    file_password_hash: VecRef<u8>,
    database_path: String,
    max_length_inplace_lob: Integer,
    cipher: String,
    auto_server_mode: bool,
    auto_server_port: Integer,
    pub page_size: Integer,
    database_short_name: String,
    pub read_only: bool,
    file_lock_method: FileLockMethod,
    database_url: String,
    mode: Option<&'static Mode>,
    default_null_ordering: Option<&'static DefaultNullOrdering>,
    cache_type: String,
    ignore_catalogs: bool,
    lock_mode: Integer,
    starting: AtomicBool,
    store: StoreRef,
}

pub type DatabaseRef = Option<Arc<H2RustCell<Database>>>;

impl Database {
    pub fn new(connection_info: &mut ConnectionInfo, cipher: &String) -> Result<DatabaseRef> {
        let database_ref = build_h2_rust_cell!(Default::default());
        Self::init(database_ref.clone(), connection_info, cipher)?;

        Ok(database_ref)
    }

    fn init(database_ref: DatabaseRef,
            connectionInfoMutRef: &mut ConnectionInfo,
            cipher: &String) -> Result<()> {
        let databaseMutRef = get_ref_mut!(database_ref);

        databaseMutRef.db_settings = connectionInfoMutRef.get_db_settings()?;
        databaseMutRef.persistent = connectionInfoMutRef.persistent;

        databaseMutRef.file_password_hash = connectionInfoMutRef.file_password_hash.clone();
        databaseMutRef.database_path = connectionInfoMutRef.get_database_path()?;
        databaseMutRef.max_length_inplace_lob = constant::DEFAULT_MAX_LENGTH_INPLACE_LOB;
        databaseMutRef.cipher = cipher.clone();
        databaseMutRef.auto_server_mode = connectionInfoMutRef.get_property_bool("AUTO_SERVER", false)?;
        databaseMutRef.auto_server_port = connectionInfoMutRef.get_property_int("AUTO_SERVER_PORT", 0)?;
        databaseMutRef.page_size = connectionInfoMutRef.get_property_int("PAGE_SIZE", constant::DEFAULT_PAGE_SIZE)?;

        // database.database_short_name = Self::parse_database_short_name(this.clone());
        databaseMutRef.database_short_name = databaseMutRef.parse_database_short_name();

        if !databaseMutRef.cipher.is_empty() && databaseMutRef.page_size % file_encrypt::BLOCK_SIZE != 0 {
            throw!( DbError::get_unsupported_exception(&format!("CIPHER && PAGE_SIZE={}",  databaseMutRef.page_size)));
        }

        let access_mode_data = string_utils::to_lower_english(&connectionInfoMutRef.get_property_string("ACCESS_MODE_DATA", "rw"));
        if "r".eq(&access_mode_data) {
            databaseMutRef.read_only = true;
        }

        let lock_method_name = connectionInfoMutRef.get_property_string("FILE_LOCK", h2_rust_constant::EMPTY_STR);
        databaseMutRef.file_lock_method =
            if !lock_method_name.is_empty() {
                file_lock::get_file_lock_method(&lock_method_name)?
            } else {
                if databaseMutRef.auto_server_mode {
                    file_lock_method::FILE
                } else {
                    file_lock_method::FS
                }
            };

        databaseMutRef.database_url = connectionInfoMutRef.url.clone();

        databaseMutRef.mode = Mode::get_regular();
        let s = connectionInfoMutRef.remove_property_str("MODE", h2_rust_constant::EMPTY_STR);
        if !s.is_empty() {
            databaseMutRef.mode = Mode::get_instance(&s);
            if databaseMutRef.mode.is_none() {
                throw!(DbError::get(error_code::UNKNOWN_MODE_1, vec![&s]));
            }
        }

        let s = connectionInfoMutRef.remove_property_str("DEFAULT_NULL_ORDERING", h2_rust_constant::EMPTY_STR);
        if !s.is_empty() {
            let default_null_ordering = DefaultNullOrdering::value_of(&string_utils::to_upper_english(&s));
            match default_null_ordering {
                Some(d) => {
                    databaseMutRef.default_null_ordering = Some(d);
                }
                None => {
                    throw!(DbError::get_invalid_value_exception("DEFAULT_NULL_ORDERING", &s));
                }
            }
        }

        let allow_builtin_alias_override = connectionInfoMutRef.get_property_bool("BUILTIN_ALIAS_OVERRIDE", false)?;

        let close_at_vm_shutdown = databaseMutRef.db_settings.db_close_on_exit;
        if databaseMutRef.auto_server_mode && !close_at_vm_shutdown {
            throw!(DbError::get_unsupported_exception("AUTO_SERVER=TRUE && DB_CLOSE_ON_EXIT=FALSE"));
        }

        databaseMutRef.cache_type = string_utils::to_upper_english(&connectionInfoMutRef.remove_property_str("CACHE_TYPE", constant::CACHE_TYPE_DEFAULT));
        databaseMutRef.ignore_catalogs = connectionInfoMutRef.get_property_bool("IGNORE_CATALOGS", databaseMutRef.db_settings.ignore_catalogs)?;
        databaseMutRef.lock_mode = connectionInfoMutRef.get_property_int("LOCK_MODE", constant::DEFAULT_LOCK_MODE)?;

        {
            if databaseMutRef.auto_server_mode &&
                (databaseMutRef.read_only || !databaseMutRef.persistent || databaseMutRef.file_lock_method == file_lock_method::NO || databaseMutRef.file_lock_method == file_lock_method::FS) {
                throw!(DbError::get_unsupported_exception("AUTO_SERVER=TRUE && (readOnly || inMemory || FILE_LOCK=NO || FILE_LOCK=FS)"));
            }

            if databaseMutRef.persistent {
                let lock_file_name = databaseMutRef.database_path.clone().add(constant::SUFFIX_LOCK_FILE);
                if databaseMutRef.read_only {
                    if file_utils::exist(&lock_file_name) {
                        throw!( DbError::get(error_code::DATABASE_ALREADY_OPEN_1, vec![&format!("Lock file exists: {}" , lock_file_name)]));
                    }
                } else if databaseMutRef.file_lock_method != file_lock_method::NO && databaseMutRef.file_lock_method != file_lock_method::FS {
                    todo!()
                    /*lock = new FileLock(traceSystem, lock_file_name, Constants.LOCK_SLEEP);
                    lock.lock(fileLockMethod);
                    if autoServerMode {
                        startServer(lock.getUniqueId());
                    }*/
                }

                // Self::delete_old_temp_files(this.clone())?;
                databaseMutRef.delete_old_temp_files()?;
            }

            databaseMutRef.starting = AtomicBool::new(true);

            if !databaseMutRef.db_settings.mv_store {
                throw!( DbError::get(error_code::GENERAL_ERROR_1,vec!["mv store not enabled"]));
            }

            databaseMutRef.store = Store::new(database_ref.clone(), connectionInfoMutRef.file_encryption_key.clone())?;
        }

        Ok(())
    }

    fn parse_database_short_name(&self) -> String {
        //  let binding = (&*this).borrow_mut();
        //  let database = binding.unwrap();

        let database_path = &self.database_path;
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
            if self.db_settings.database_to_upper {
                string_utils::to_upper_english(database_path)
            } else {
                if self.db_settings.database_to_lower {
                    string_utils::to_lower_english(database_path)
                } else {
                    database_path.to_string()
                }
            };

        string_utils::truncate_string(&database_path, constant::MAX_IDENTIFIER_LENGTH as usize)
    }

    fn delete_old_temp_files(&self) -> Result<()> {
        // let binding = (&*this).borrow_mut();
        // let database = binding.unwrap();

        let parent_dir_path = file_utils::get_parent(&self.database_path)?;

        for path in file_utils::new_directory_stream(parent_dir_path)? {
            if path.ends_with(constant::SUFFIX_TEMP_FILE) && path.starts_with(&self.database_path) {
                file_utils::try_delete(&path);
            }
        }

        Ok(())
    }

    pub fn get_database_path(&self) -> Result<String> {
        if self.persistent {
            if let Ok(s) = file_utils::to_real_path(&self.database_path)?.into_os_string().into_string() {
                Ok(s)
            } else {
                throw!(DbError::get(error_code::GENERAL_ERROR_1,vec!["Database::get_database_path()"]))
            }
        } else {
            Ok(h2_rust_constant::EMPTY_STR.to_string())
        }
    }
}