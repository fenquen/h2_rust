use anyhow::Result;
use crate::api::error_code;
use crate::engine::constant;
use crate::h2_rust_common::{h2_rust_constant, Properties};
use crate::jdbc::jdbc_connection::JdbcConnection;
use crate::message::db_error::DbError;
use crate::throw;

const DEFAULT_URL: &str = "jdbc:default:connection";

pub struct Driver {}

impl Driver {
    pub fn connect(&self, url: String, properties: Properties) -> Result<JdbcConnection> {
        if url.is_empty() {
            throw!(DbError::get(error_code::URL_FORMAT_ERROR_2, vec![constant::URL_FORMAT, h2_rust_constant::EMPTY_STR]));
        }

        if url.starts_with(constant::START_URL) {
            return Ok(JdbcConnection::new(url.to_string(),
                                          properties,
                                          h2_rust_constant::EMPTY_STR.to_string(),
                                          h2_rust_constant::EMPTY_STR.to_string(),
                                          false)?);
        }


        if DEFAULT_URL.eq(&url) {}

        throw!(DbError::get(error_code::URL_FORMAT_ERROR_2,vec![constant::URL_FORMAT,&url]))
    }
}
