use std::collections::HashMap;
use crate::properties_type;
use anyhow::Result;
use crate::h2_rust_common::Properties;

#[derive(Default)]
pub struct JdbcConnection {}

impl JdbcConnection {
    pub fn new(url: String,
               info: Properties,
               user: String,
               password: impl ToString,
               forbid_creation: bool) -> Result<Self> {
        Ok(JdbcConnection {})
    }
}