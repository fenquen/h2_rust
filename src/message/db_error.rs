use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, format, Formatter};
use std::path::Path;
use lazy_static::lazy_static;
use crate::api::error_code;
use crate::h2_rust_common::{Integer, h2_rust_utils};
use crate::h2_rust_common::Properties;

lazy_static! {
  static ref MESSAEGS:Properties = h2_rust_utils::load_properties(Path::new("_messages_en.properties")).unwrap();
}

#[derive(Debug)]
pub struct DbError {
    /// 有errCode转化而来,远程用到
    pub sql_state: String,

    /// 通过sql_state对照账单得到相应的话术,远程用到
    pub message: String,

    pub error_code: Integer,

}

impl DbError {
    pub fn get_internal_error(s: &str) -> Self {
        Self::get(error_code::GENERAL_ERROR_1, vec![s])
    }

    pub fn get_unsupported_exception(s: &str) -> Self {
        Self::get(error_code::FEATURE_NOT_SUPPORTED_1, vec![s])
    }

    pub fn get(error_code: Integer, params: Vec<&str>) -> Self {
        // error_code对应的文本
        let sql_state = error_code::get_state(error_code);
        let message = Self::translate(&sql_state, params);

        DbError {
            sql_state,
            message,
            error_code,
        }
    }

    fn translate(sql_state: &str, params: Vec<&str>) -> String {
        // 是相应的模板文本
        let mut message = match MESSAEGS.get(sql_state) {
            Some(s) => s.clone(),
            None => return format!("Message {} not found", sql_state)
        };

        let mut a = 0;
        for param in params {
            message = message.replace(&format!(r"({})", a), param);
            a = a + 1;
        }

        message
    }

    pub fn get_invalid_value_exception(param: &str, value: &impl AsRef<str>) -> Self {
        Self::get(error_code::INVALID_VALUE_2, vec![value.as_ref(), param])
    }
}

impl Display for DbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "message:{},sql_state:{},error_code:{}",
               self.message, self.sql_state, self.error_code)
    }
}

impl Error for DbError {}

#[cfg(test)]
mod test {
    use crate::message::db_error::DbError;

    #[test]
    fn test_translate() {
        use crate::message::db_error;
        println!("{}", DbError::translate("07001", vec!["a", "a"]));
    }

    #[test]
    fn test_get_invalid_value_exception(){
        let s = "1".to_string();
        DbError::get_invalid_value_exception("a",&s);
        let a = &s;
    }
}