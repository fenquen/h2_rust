use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_utils, Integer, Long};
use crate::message::db_error::DbError;
use crate::{throw, unsigned_right_shift};

pub fn get_config_int_param(config: &HashMap<String, Box<dyn Any>>, key: &str, default_value: Integer) -> Integer {
    if let Some(param) = config.get(key) {
        let param = &**param;
        match param.downcast_ref::<Integer>() {
            Some(as_integer) => {
                *as_integer
            }
            None => {
                match param.downcast_ref::<String>() {
                    Some(as_string) => {
                        match h2_rust_utils::integer_decode(as_string) {
                            Ok(r) => r,
                            Err(_) => default_value
                        }
                    }
                    None => {
                        default_value
                    }
                }
            }
        }
    } else {
        default_value
    }
}

pub fn check_argument(test: bool, message: &str) -> Result<()> {
    if !test {
        throw!(DbError::get(error_code::GENERAL_ERROR_1,vec![message]));
    }
    Ok(())
}

pub fn is_page_saved(position: Long) -> bool {
    (position & !1) != 0
}

pub fn get_page_chunk_id(position: Long) -> Integer {
    unsigned_right_shift!(position, 38, Long) as Integer
}