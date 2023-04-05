use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_utils, Integer};
use crate::message::db_error::DbError;
use crate::throw;

pub fn get_config_int_param(config: &HashMap<String, Box<dyn Any>>, key: &str, default_value: Integer) -> Integer {
    let param = config.get(key);
    if let Some(param) = param {
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