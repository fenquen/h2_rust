use anyhow::Result;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_constant, h2_rust_utils, Integer};
use crate::message::db_error::DbError;
use crate::throw;

pub static EMPTY_INT_ARRAY: Vec<u8> = vec![];

pub fn parse_bool(s: &str, default_value: bool, throw_exception: bool) -> Result<bool> {
    if s.is_empty() {
        return Ok(default_value);
    }

    match s.len() {
        1 => {
            if s.eq("1") || s.eq_ignore_ascii_case("t") || s.eq_ignore_ascii_case("y") {
                return Ok(true);
            }

            if s.eq("0") || s.eq_ignore_ascii_case("f") || s.eq_ignore_ascii_case("n") {
                return Ok(false);
            }
        }
        2 => {
            if s.eq_ignore_ascii_case("no") {
                return Ok(false);
            }
        }
        3 => {
            if s.eq_ignore_ascii_case("yes") {
                return Ok(true);
            }
        }
        4 => {
            if s.eq_ignore_ascii_case("true") {
                return Ok(true);
            }
        }
        5 => {
            if s.eq_ignore_ascii_case("false") {
                return Ok(false);
            }
        }
        _ => {}
    }

    if throw_exception {
        throw!(DbError::get(error_code::DATA_CONVERSION_ERROR_1, vec![&format!("can't convert {} to bool", s)]));
    }

    Ok(default_value)
}

pub fn scaleForAvailableMemory(value: Integer) -> Integer {
    match h2_rust_utils::getTotalPhysicalMemorySize() {
        Ok(total) => {
            let mut a = total as Integer / h2_rust_constant::GB;
            if a == 0 {
                a = 1;
            }
            value * a
        }
        Err(e) => value
    }
}