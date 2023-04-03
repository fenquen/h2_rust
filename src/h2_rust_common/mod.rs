use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use crate::h2_rust_common::h2_rust_constant::{NEGATIVE, POSITIVE};
use anyhow::Result;
use crate::api::error_code;
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::message::db_error::DbError;

pub mod macros;
pub mod h2_rust_utils;
pub mod h2_rust_constant;

pub type Properties = HashMap<String, String>;
pub type Integer = i32;

pub type Long = i64;
pub type Byte = i8;

pub fn throw<T, E: Error + Send + Sync + 'static>(e: E) -> Result<T> {
    core::result::Result::Err(e)?
}

pub enum Nullable<T> {
    NotNull(T),
    Null,
}

impl<T> Nullable<T> {
    pub fn unwrap(&self) -> &T {
        match self {
            NotNull(t) => t,
            Null => panic!("null")
        }
    }

    pub fn unwrap_mut(&mut self) -> &mut T {
        match self {
            NotNull(t) => t,
            Null => panic!("null")
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Null => true,
            _ => false
        }
    }
}

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Null
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        if let Some(t) = value {
            NotNull(t)
        } else {
            Null
        }
    }
}