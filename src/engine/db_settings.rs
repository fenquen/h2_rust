use std::collections::HashMap;
use std::mem::transmute;
use anyhow::Result;
use lazy_static::lazy_static;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_utils, Integer};
use crate::message::db_error::DbError;
use crate::util::utils;

pub const TABLE_SIZE: Integer = 64;

lazy_static! {
    pub static ref DEFAULT :DbSettings = {
        DbSettings::new(HashMap::with_capacity(TABLE_SIZE as usize)).unwrap()
    } ;
}

#[derive(Default)]
pub struct DbSettings {
    settings: HashMap<String, String>,
    pub database_to_upper: bool,
    pub database_to_lower: bool,
    pub db_close_on_exit: bool,
    pub ignore_catalogs: bool,
    pub mv_store: bool,
}

impl DbSettings {
    pub fn new(settings: HashMap<String, String>) -> Result<DbSettings> {
        let mut db_settings = DbSettings::default();

        db_settings.settings = settings;
        db_settings.init()?;

        Ok(db_settings)
    }

    fn init(&mut self) -> Result<()> {
        let lower = Self::get_bool(self, "DATABASE_TO_LOWER", false)?;
        let upper_set = Self::contains_key(self, "DATABASE_TO_UPPER");
        let mut upper = Self::get_bool(self, "DATABASE_TO_UPPER", true)?;
        if lower && upper {
            if upper_set {
                Err(DbError::get(error_code::UNSUPPORTED_SETTING_COMBINATION, vec!["DATABASE_TO_LOWER & DATABASE_TO_UPPER"]))?;
            }

            upper = false;
        }

        self.database_to_lower = lower;
        self.database_to_upper = upper;

        self.settings.insert("DATABASE_TO_LOWER".to_string(), lower.to_string());
        self.settings.insert("DATABASE_TO_UPPER".to_string(), upper.to_string());

        self.db_close_on_exit = Self::get_bool(self, "DB_CLOSE_ON_EXIT", true)?;
        self.ignore_catalogs = Self::get_bool(self, "IGNORE_CATALOGS", false)?;

        self.mv_store = Self::get_bool(self, "MV_STORE", true)?;

        Ok(())
    }

    fn get_bool(&mut self, key: &str, default_value: bool) -> Result<bool> {
        let s = Self::get_string(self, key, &default_value.to_string());
        utils::parse_bool(&s, default_value, true)
    }

    fn get_int(&mut self, key: &str, default_value: Integer) -> Result<Integer> {
        let s = Self::get_string(self, key, &default_value.to_string());
        h2_rust_utils::integer_decode(&s)
    }

    fn get_string(&mut self, key: &str, default_value: &str) -> String {
        if let Some(s) = self.settings.get(key) {
            return s.to_string();
        }

        // 相对了java版本少了读取system properties
        self.settings.insert(key.to_string(), default_value.to_string());
        default_value.to_string()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.settings.contains_key(key)
    }
}

mod test {
    use std::collections::HashMap;
    use crate::engine::db_settings::DbSettings;

    #[test]
    fn test_a() {
        let h = HashMap::<String, String>::new();
        let mut db_settings = DbSettings::new(h).unwrap();
        let r = db_settings.get_bool("aaaaa", true).unwrap();
        println!("{}", r);
    }
}