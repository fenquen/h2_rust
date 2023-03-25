use std::collections::HashMap;
use anyhow::Result;
use crate::engine::constant;
use crate::errors::H2RustError;
use crate::jdbc::jdbc_connection::JdbcConnection;
use crate::properties_type;

pub fn connect(url: String, properties: properties_type!()) -> Result<JdbcConnection> {
    if url.starts_with(constant::START_URL) {
        return Ok(JdbcConnection {});
    }


    Err(H2RustError::SQLError(format!("url:{}不正确", url)))?
}
