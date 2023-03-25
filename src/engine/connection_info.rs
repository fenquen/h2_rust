use std::collections::HashMap;
use crate::engine::constant;
use crate::properties_type;
use anyhow::Result;

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

        if !url.starts_with(constant::START_URL) {}

        Ok(ConnectionInfo {
            url: this_url,
            original_url: url.clone(),
        })
    }
}
