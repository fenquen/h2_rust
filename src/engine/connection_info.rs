use std::collections::HashMap;
use crate::engine::constant as engine_constant;
use crate::properties_type;
use anyhow::Result;
use crate::api::error_code;
use crate::message::db_error::DbError;

pub struct ConnectionInfo {
    pub url: String,
    pub original_url: String,
}

impl ConnectionInfo {
    pub fn new(url: String,
               properties: properties_type!(),
               user: String,
               password: impl ToString) -> Result<ConnectionInfo> {
        let mut this_url = url.clone();

        if !url.starts_with(engine_constant::START_URL) {
            Self::get_format_exception(&url)?;
        }

        Ok(ConnectionInfo {
            url: this_url,
            original_url: url.clone(),
        })
    }

    pub fn get_format_exception(url: &str) -> Result<ConnectionInfo> {
        Err(DbError::get(error_code::URL_FORMAT_ERROR_2,
                         vec![engine_constant::URL_FORMAT, url]))?
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test_build_connection_info() {
        use std::collections::HashMap;
        use crate::engine::connection_info::ConnectionInfo;

        match ConnectionInfo::new(String::from("dasdasdasd"),
                                  HashMap::<String, String>::new(),
                                  String::from("a"),
                                  "a") {
            Ok(_) => {}
            Err(e) => { println!("{}", e) }
        }
    }
}