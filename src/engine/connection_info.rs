use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Add;
use std::path::Path;
use std::sync::{Arc, Once};
use crate::engine::{constant as engine_constant, constant, db_settings};
use anyhow::Result;
use lazy_static::lazy_static;
use toml::value::Index;
use crate::api::error_code;
use crate::h2_rust_common::{Properties, h2_rust_utils, h2_rust_constant, Integer, Nullable, Byte};
use crate::message::db_error::DbError;
use crate::command::set_types;
use crate::engine::db_settings::DbSettings;
use crate::{h2_rust_common, throw};
use crate::h2_rust_common::Nullable::NotNull;
use crate::store::fs::file_utils;
use crate::util::{io_utils, string_utils, utils};

static COMMON_SETTINGS: [&str; 12] = [
    "ACCESS_MODE_DATA", "AUTO_RECONNECT", "AUTO_SERVER", "AUTO_SERVER_PORT",
    "CACHE_TYPE",
    "FILE_LOCK",
    "JMX",
    "NETWORK_TIMEOUT",
    "OLD_INFORMATION_SCHEMA", "OPEN_NEW",
    "PAGE_SIZE",
    "RECOVER"];

lazy_static! {
    pub static ref KNOWN_SETTINGS:HashSet<String> = {

        let settings = [
            "AUTHREALM", "AUTHZPWD", "AUTOCOMMIT",
            "CIPHER", "CREATE",
            "FORBID_CREATION",
            "IGNORE_UNKNOWN_SETTINGS", "IFEXISTS", "INIT",
            "NO_UPGRADE",
            "PASSWORD", "PASSWORD_HASH",
            "RECOVER_TEST",
            "USER"];

        let mut knowing_settings = HashSet::<String>::with_capacity(128);

        // todo 优化实现addAll
        h2_rust_utils::add_all(&mut knowing_settings,set_types::get_types());

        for common_setting in COMMON_SETTINGS {
            if knowing_settings.contains(common_setting) {
                panic!("knowing_settings has already contain {}", common_setting);
            }

            knowing_settings.insert(common_setting.to_string());
        }

        for setting in settings {
            if knowing_settings.contains(setting) {
                panic!("knowing_settings has already contain {}", setting);
            }

            knowing_settings.insert(setting.to_string());
        }

        knowing_settings
    };

    pub static ref IGNORED_BY_PARSER:HashSet<String> = {
        let mut ignored_by_parser = HashSet::with_capacity(64);

        h2_rust_utils::add_all(&mut ignored_by_parser, &COMMON_SETTINGS);
        h2_rust_utils::add_all(&mut ignored_by_parser,&["ASSERT", "BINARY_COLLATION", "DB_CLOSE_ON_EXIT", "PAGE_STORE", "UUID_COLLATION"]);

        ignored_by_parser
    };
}

#[derive(Default)]
pub struct ConnectionInfo {
    pub url: String,
    pub original_url: String,
    pub prop: Properties,
    pub user: String,
    /// database name
    pub name: String,
    pub name_normalized: String,
    pub persistent: bool,
    pub remote: bool,
    pub ssl: bool,
    pub unnamed_in_memory: bool,
    pub user_password_hash: Arc<Nullable<Vec<u8>>>,
    pub file_password_hash: Arc<Nullable<Vec<u8>>>,
}

impl ConnectionInfo {
    pub fn new(url: String,
               info: &Properties,
               user: String,
               password: String) -> Result<Self> {
        let mut connection_info: ConnectionInfo = Default::default();

        connection_info.init(url, info, user, password)?;

        Ok(connection_info)
    }

    fn init(&mut self,
            url: String,
            info: &Properties,
            user: String,
            password: String) -> Result<()> {
        self.original_url = url.clone();
        self.url = url;

        if !self.url.starts_with(engine_constant::START_URL) {
            throw!(self.get_format_exception());
        }

        if !info.is_empty() {
            self.read_properties(info)?;
        }

        if !user.is_empty() {
            self.prop.insert("USER".to_string(), user);
        }

        if !password.is_empty() {
            self.prop.insert("PASSWORD".to_string(), password);
        }

        Self::read_settings_from_url(self);

        // 以下书写是错误的  cannot borrow `*self` as mutable more than once at a time
        // Self::set_user_name(self, Self::remove_property(self, "USER", h2_rust_constant::EMPTY_STR));
        let user = Self::remove_property_str(self, "USER", h2_rust_constant::EMPTY_STR);
        Self::set_user_name(self, user);

        self.name = self.url[constant::START_URL.len()..].to_string();

        Self::parse_name(self);

        Self::convert_passwords(self)?;

        Ok(())
    }

    fn get_format_exception(&self) -> DbError {
        DbError::get(error_code::URL_FORMAT_ERROR_2, vec![engine_constant::URL_FORMAT, &self.url])
    }

    fn read_properties(&mut self, info: &Properties) -> Result<()> {
        let db_settings = Self::get_db_settings(self)?;

        for key in info.keys() {
            let uppercase_key = key.to_uppercase();

            if self.prop.contains_key(&uppercase_key) {
                Err(DbError::get(error_code::DUPLICATE_PROPERTY_1, vec![&uppercase_key]))?;
            }

            let value = match info.get(key) {
                Some(v) => v,
                None => continue
            };

            if Self::is_known_setting(&uppercase_key) {
                self.prop.insert(uppercase_key, value.to_string());
                continue;
            }

            if db_settings.contains_key(&uppercase_key) {
                self.prop.insert(uppercase_key, value.to_string());
            }
        }

        Ok(())
    }

    fn is_known_setting(setting: &str) -> bool {
        KNOWN_SETTINGS.contains(setting)
    }

    pub fn get_db_settings(&self) -> Result<DbSettings> {
        let mut settings = HashMap::with_capacity(db_settings::TABLE_SIZE as usize);
        for key in self.prop.keys() {
            if !Self::is_known_setting(key) && db_settings::DEFAULT.contains_key(key) {
                settings.insert(key.to_string(),
                                self.prop.get(key).map_or_else(|| h2_rust_constant::EMPTY_STR.to_string(),
                                                               |s| s.to_string()),
                );
            }
        }

        DbSettings::new(settings)
    }

    fn read_settings_from_url(&mut self) -> Result<()> {
        let index = match self.url.find(h2_rust_constant::SEMI_COLUMN) {
            Some(index) => index,
            None => return Ok(())
        };

        let settings = self.url[index..].to_string();
        self.url = self.url[0..index].to_string();

        let mut unknown_setting = h2_rust_constant::EMPTY_STR.to_string();

        for setting in settings.split(h2_rust_constant::SEMI_COLUMN) {
            if setting.is_empty() {
                continue;
            }

            let equal = match setting.find(h2_rust_constant::EQUAL) {
                Some(index) => index,
                None => throw!(Self::get_format_exception(self))
            };

            let key = setting[..equal].to_uppercase();
            let value = &setting[equal + 1..];

            if Self::is_known_setting(&key) || db_settings::DEFAULT.contains_key(&key) {
                if let Some(old) = self.prop.get(&key) {
                    if old.eq(value) {
                        throw!(DbError::get(error_code::DUPLICATE_PROPERTY_1,vec![&key]));
                    }
                } else {
                    self.prop.insert(key, value.to_string());
                }
            } else {
                unknown_setting = key;
            }
        }

        if !unknown_setting.is_empty() {
            // 不能容忍未知的配置
            if let Some(s) = self.prop.get("IGNORE_UNKNOWN_SETTINGS") {
                if !utils::parse_bool(s, false, false)? {
                    throw!(DbError::get(error_code::UNSUPPORTED_SETTING_1,vec![&unknown_setting]));
                }
            }
        }

        Ok(())
    }

    pub fn remove_property_str(&mut self, key: &str, default_value: &str) -> String {
        match self.prop.remove(key) {
            Some(v) => v,
            None => default_value.to_string()
        }
    }

    pub fn remove_property_bool(&mut self, key: &str, default_value: bool) -> Result<bool> {
        let s = Self::remove_property_str(self, key, h2_rust_constant::EMPTY_STR);
        utils::parse_bool(&s, default_value, false)
    }

    pub fn set_user_name(&mut self, name: String) {
        self.user = name.to_uppercase();
    }

    fn parse_name(&mut self) {
        if h2_rust_constant::DOT.eq(&self.name) {
            self.name = "mem:".to_string();
        }

        if self.name.starts_with("tcp:") {
            self.remote = true;
            self.name = self.name["tcp:".len()..].to_string();
        } else if self.name.starts_with("ssl:") {
            self.remote = true;
            self.name = self.name["ssl:".len()..].to_string();
        } else if self.name.starts_with("mem:") {
            self.persistent = false;
            if "mem:".eq(&self.name) {
                self.unnamed_in_memory = true;
            }
        } else if self.name.starts_with("file:") {
            self.persistent = true;
            self.name = self.name["file:".len()..].to_string();
        } else {
            self.persistent = true;
        }

        if self.persistent && !self.remote {
            self.name = io_utils::name_separators_to_native(&self.name);
        }
    }

    fn convert_passwords(&mut self) -> Result<()> {
        let password = Self::remove_password(self);
        let password_hash = Self::remove_property_bool(self, "PASSWORD_HASH", false)?;

        self.user_password_hash = Arc::new(NotNull(Self::hash_password(password_hash, &self.user, &password)?));

        Ok(())
    }

    fn remove_password(&mut self) -> String {
        if let Some(password) = self.prop.remove("PASSWORD") {
            password
        } else {
            h2_rust_constant::EMPTY_STR.to_string()
        }
    }

    fn hash_password(password_hash: bool, user_name: &str, password: &str) -> Result<Vec<u8>> {
        if password_hash {
            return string_utils::convert_hex_to_byte_vec(password);
        }

        if user_name.is_empty() && password.is_empty() {
            return Ok(Vec::new());
        }

        Ok(Vec::new())
    }

    pub fn set_property(&mut self, key: &str, value: &str) {
        if !value.is_empty() {
            self.prop.insert(key.to_string(), value.to_string());
        }
    }

    pub fn get_property_bool(&self, key: &str, default_value: bool) -> Result<bool> {
        let s = Self::get_property_string(self, key, h2_rust_constant::EMPTY_STR);
        utils::parse_bool(key, false, false)
    }

    pub fn get_property_int(&self, key: &str, default_value: Integer) -> Result<Integer> {
        if let Some(s) = Self::get_property(&self, key) {
            let a = Integer::from_str_radix(&s, 10)?;
            Ok(a)
        } else {
            Ok(default_value)
        }
    }

    pub fn get_property_string(&self, key: &str, default_value: &str) -> String {
        if let Some(s) = Self::get_property(&self, key) {
            s
        } else {
            default_value.to_string()
        }
    }

    pub fn get_property(&self, key: &str) -> Option<String> {
        if let Some(s) = self.prop.get(key) {
            Some(s.to_string())
        } else {
            None
        }
    }

    pub fn get_database_path(&mut self) -> Result<String> {
        if !self.persistent {
            return Ok(self.name.to_string());
        }

        if self.name_normalized.is_empty() {
            if !file_utils::is_absolute("") &&
                !self.name.contains("./") &&
                !self.name.contains(".\\") &&
                !self.name.contains(":/") &&
                !self.name.contains(":\\") {
                throw!(DbError::get(error_code::URL_RELATIVE_TO_CWD,vec![&self.original_url]));
            }

            let real = file_utils::to_real_path(&self.name.clone().add(constant::SUFFIX_MV_FILE))?;
            let file_name = file_utils::get_name(&real);

            if file_name.len() < constant::SUFFIX_MV_FILE.len() + 1 {
                throw!(DbError::get(error_code::INVALID_DATABASE_NAME_1,vec![&self.name]));
            }

            if let Ok(real) = real.into_os_string().into_string() {
                self.name_normalized = real[..real.len() - constant::SUFFIX_MV_FILE.len()].to_string();
            } else {
                throw!(DbError::get(error_code::GENERAL_ERROR_1,vec!["real.into_os_string().into_string()"]));
            }
        }

        Ok(self.name_normalized.to_string())
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test_build_connection_info() {
        use std::collections::HashMap;
        use crate::engine::connection_info::ConnectionInfo;

        match ConnectionInfo::new(String::from("jdbc:h2:file:./data/rust"),
                                  &HashMap::<String, String>::new(),
                                  String::from("a"),
                                  String::from("a")) {
            Ok(_) => {}
            Err(e) => { println!("{}", e) }
        }
    }
}