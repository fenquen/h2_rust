use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_utils, Integer, Long, ULong};
use crate::message::db_error::DbError;
use crate::{throw, unsigned_right_shift};

/// An error occurred while reading from the file.
pub const ERROR_READING_FAILED: Integer = 1;

/// An error occurred when trying to write to the file.
pub const ERROR_WRITING_FAILED: Integer = 2;

/// An internal error occurred. This could be a bug, or a memory corruption
/// (for example caused by out of memory)
pub const ERROR_INTERNAL: Integer = 3;

/// The object is already closed
pub const ERROR_CLOSED: Integer = 4;

/// The file format is not supported.
pub const ERROR_UNSUPPORTED_FORMAT: Integer = 5;

/// The file is corrupt or (for encrypted files) the encryption key is wrong.
pub const ERROR_FILE_CORRUPT: Integer = 6;

/// The file is locked.
pub const ERROR_FILE_LOCKED: Integer = 7;

/// An error occurred when serializing or de-serializing.
pub const ERROR_SERIALIZATION: Integer = 8;

/// The application was trying to read data from a chunk that is no longer available.
pub const ERROR_CHUNK_NOT_FOUND: Integer = 9;

/// The block in the stream store was not found.
pub const ERROR_BLOCK_NOT_FOUND: Integer = 50;

/// The transaction store is corrupt.
pub const ERROR_TRANSACTION_CORRUPT: Integer = 100;

/// An entry is still locked by another transaction.
pub const ERROR_TRANSACTION_LOCKED: Integer = 101;

/// There are too many open transactions.
pub const ERROR_TOO_MANY_OPEN_TRANSACTIONS: Integer = 102;

/// The transaction store is in an illegal state (for example, not yet initialized).
pub const ERROR_TRANSACTION_ILLEGAL_STATE: Integer = 103;

/// The transaction contains too many changes.
pub const ERROR_TRANSACTION_TOO_BIG: Integer = 104;

/// Deadlock discovered and one of transactions involved chosen as victim and rolled back.
pub const ERROR_TRANSACTIONS_DEADLOCK: Integer = 105;

/// The transaction store can not be initialized because data type is not found in type registry.
pub const ERROR_UNKNOWN_DATA_TYPE: Integer = 106;

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