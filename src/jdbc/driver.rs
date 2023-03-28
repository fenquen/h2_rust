use std::collections::HashMap;
use anyhow::Result;
use crate::engine::connection_info::ConnectionInfo;
use crate::engine::constant;
use crate::jdbc::jdbc_connection::JdbcConnection;
use crate::properties_type;

pub fn connect(url: String, properties: properties_type!()) -> Result<JdbcConnection> {
    if url.starts_with(constant::START_URL) {
        return Ok(JdbcConnection {});
    }


    todo!()
}
