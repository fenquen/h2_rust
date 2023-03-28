use std::collections::HashMap;
use std::error::Error;
use crate::h2_rust_common::h2_rust_constant::{NEGATIVE, POSITIVE};
use anyhow::Result;
use crate::api::error_code;
use crate::message::db_error::DbError;

pub mod macros;
pub mod h2_rust_utils;
pub mod h2_rust_constant;

pub type Properties = HashMap<String, String>;
pub type Integer = i32;

pub fn throw<T, E: Error + Send + Sync + 'static>(e: E) -> Result<T> {
    core::result::Result::Err(e)?
}
