use std::fmt::format;
use std::sync::Arc;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::db_settings::DbSettings;
use anyhow::Result;
use crate::api::error_code;
use crate::engine::{constant, database};
use crate::engine::mode::Mode;
use crate::h2_rust_common::{h2_rust_constant, Integer, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::message::db_error::DbError;
use crate::store::file_lock;
use crate::store::file_lock_method::FileLockMethod;
use crate::store::fs::encrypt::file_encrypt;
use crate::throw;
use crate::util::string_utils;

#[derive(Default)]
pub struct Database {
    db_settings: Arc<Nullable<DbSettings>>,
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
}

impl Database {
    pub fn new(connection_info: &mut ConnectionInfo, cipher: &String) {
        let mut database: Database = Default::default();
        database.init(connection_info, cipher);
    }

    fn init(&mut self, connection_info: &mut ConnectionInfo, cipher: &String) -> Result<()> {
        let database_path = connection_info.get_database_path();

        self.db_settings = Arc::new(NotNull(connection_info.get_db_settings()?));
        self.persistent = connection_info.persistent;
        self.file_password_hash = connection_info.file_password_hash.clone();
        self.database_path = connection_info.get_database_path()?;
        self.max_length_inplace_lob = constant::DEFAULT_MAX_LENGTH_INPLACE_LOB;
        self.cipher = cipher.clone();
        self.auto_server_mode = connection_info.get_property_bool("AUTO_SERVER", false)?;
        self.auto_server_port = connection_info.get_property_int("AUTO_SERVER_PORT", 0)?;
        self.page_size = connection_info.get_property_int("PAGE_SIZE", constant::DEFAULT_PAGE_SIZE)?;

        self.database_short_name = Self::parse_database_short_name(self);

        if !self.cipher.is_empty() && self.page_size % file_encrypt::BLOCK_SIZE != 0 {
            throw!( DbError::get_unsupported_exception(&format!("CIPHER && PAGE_SIZE={}",  self.page_size)));
        }

        let access_mode_data = string_utils::to_lower_english(&connection_info.get_property_string("ACCESS_MODE_DATA", "rw"));
        if "r".eq(&access_mode_data) {
            self.read_only = true;
        }

        let lock_method_name = connection_info.get_property_string("FILE_LOCK", h2_rust_constant::EMPTY_STR);
        self.file_lock_method =
            if !lock_method_name.is_empty() {
                file_lock::get_file_lock_method(&lock_method_name)?
            } else {
                if self.auto_server_mode {
                    FileLockMethod::FILE
                } else {
                    FileLockMethod::FS
                }
            };

        self.database_url = connection_info.url.clone();

        self.mode = Nullable::from(Mode::get_regular());
        let s = connection_info.remove_property_str("MODE", h2_rust_constant::EMPTY_STR);
        if !s.is_empty() {
            self.mode = Nullable::from(Mode::get_instance(&s));
            if self.mode.is_null() {
                throw!(DbError::get(error_code::UNKNOWN_MODE_1, vec![&s]));
            }
        }

        let s = connection_info.remove_property_str("DEFAULT_NULL_ORDERING", h2_rust_constant::EMPTY_STR);
        if !s.is_empty() {
           /* try {
                defaultNullOrdering = DefaultNullOrdering.valueOf(StringUtils.toUpperEnglish(s));
            } catch (RuntimeException e) {
                throw DbException.getInvalidValueException("DEFAULT_NULL_ORDERING", s);
            }*/
        }

        Ok(())
    }

    fn parse_database_short_name(&self) -> String {
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
            if self.db_settings.unwrap().database_to_upper {
                string_utils::to_upper_english(database_path)
            } else {
                if self.db_settings.unwrap().database_to_lower {
                    string_utils::to_lower_english(database_path)
                } else {
                    database_path.to_string()
                }
            };

        string_utils::truncate_string(&database_path, constant::MAX_IDENTIFIER_LENGTH as usize)
    }
}