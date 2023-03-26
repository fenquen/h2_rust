use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, format, Formatter};
use std::path::Path;
use lazy_static::lazy_static;
use crate::api::error_code;
use crate::api::error_code::ErrorCode;
use crate::common::util;
use crate::common::Properties;

lazy_static! {
  static ref MESSAEGS:Properties = util::load_properties(Path::new("_messages_en.properties")).unwrap();
}

#[derive(Debug)]
pub struct DbError {
    /// 有errCode转化而来
    pub sql_state: String,

    /// 通过sql_state对照账单得到相应的话术
    pub message: String,

    pub error_code: u64,

}

impl DbError {
    pub fn get(error_code: ErrorCode) {
        error_code::get_state(error_code);
    }

    fn translate(sql_state: &str, params: &Vec<&str>) -> String {
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
       println!("{}", DbError::translate("07001",&vec!["a","a"]));
    }
}