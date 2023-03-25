use std::collections::HashMap;
use crate::properties_type;
use anyhow::Result;

pub struct JdbcConnection {}

impl JdbcConnection {
    pub fn new(url: String,
               info: properties_type!(),
               user: String,
               password: impl ToString,
               forbid_creation: bool) -> Result<Self> {
        Ok(JdbcConnection {})
    }
}