use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::ops::Add;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_constant, h2_rust_utils, Integer, Long, ULong};
use crate::message::db_error::DbError;
use crate::{suffix_plus_plus, throw, unsigned_right_shift};

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

/// The type for leaf page.
pub const PAGE_TYPE_LEAF: Integer = 0;

/// The type for node page.
pub const PAGE_TYPE_NODE: Integer = 1;

/// The bit mask for compressed pages (compression level fast).
pub const PAGE_COMPRESSED: Integer = 2;

/// The bit mask for compressed pages (compression level high).
pub const PAGE_COMPRESSED_HIGH: Integer = 2 + 4;

/// The bit mask for pages with page sequential number.
pub const PAGE_HAS_PAGE_NO: Integer = 8;

/// The maximum length of a variable size int.
pub const MAX_VAR_INT_LEN: Integer = 5;

/// The maximum length of a variable size long.
pub const MAX_VAR_LONG_LEN: Integer = 10;

/// The maximum integer that needs less space when using variable size
/// encoding (only 3 bytes instead of 4). 
pub const COMPRESSED_VAR_INT_MAX: Integer = 0x1fffff;

/// The maximum long that needs less space when using variable size
/// encoding (only 7 bytes instead of 8).
pub const COMPRESSED_VAR_LONG_MAX: Long = 0x1ffffffffffff;

/// The marker size of a very large page.
pub const PAGE_LARGE: Integer = 2 * 1024 * 1024;

// The following are key prefixes used in layout map

/// The prefix for chunks ("chunk."). This, plus the chunk id (hex encoded)
/// is the key, and the serialized chunk metadata is the value.
pub const META_CHUNK: &str = "chunk.";

/// The prefix for root positions of maps ("root."). This, plus the map id
/// (hex encoded) is the key, and the position (hex encoded) is the value.
pub const META_ROOT: &str = "root.";

/// The following are key prefixes used in meta map

/// The prefix for names ("name."). This, plus the name of the map, is the
/// key, and the map id (hex encoded) is the value.
pub const META_NAME: &str = "name.";

/// The prefix for maps ("map."). This, plus the map id (hex encoded) is the
/// key, and the serialized in the map metadata is the value.
pub const META_MAP: &str = "map.";

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
        throw!(DbError::get(error_code::GENERAL_ERROR_1, vec![message]));
    }
    Ok(())
}

pub fn is_page_saved(position: Long) -> bool {
    (position & !1) != 0
}

pub fn get_page_chunk_id(position: Long) -> Integer {
    unsigned_right_shift!(position, 38, Long) as Integer
}

pub fn parseMap(s: &String) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    let size = s.chars().count();
    let mut a = 0;

    while a < size {
        let startKey = a;
        // 找到最近的':'
        a = match s[a..].find(h2_rust_constant::COLON_CHAR) {
            None => {
                throw!(DbError::get(error_code::FILE_CORRUPTED_1, vec![&format!("not a map: {}",s)]))
            }
            Some(a) => { a }
        };

        let key = &s[startKey..suffix_plus_plus!(a)];
        let string_usize = parseMapValue(s, a, size)?;
        a = string_usize.1;
        map.insert(key.to_string(), string_usize.0);
    }

    Ok(map)
}

fn parseMapValue(s: &String, mut a: usize, size: usize) -> Result<(String, usize)> {
    let mut result = String::with_capacity(1024);

    while a < size {
        let mut c = s.chars().nth(suffix_plus_plus!(a)).unwrap();
        if c == h2_rust_constant::COMMA_CHAR { // 应该是 key:value,key:value 这样的pair的分割的
            break;
        }

        if c == '\"' {
            while a < size {
                c = s.chars().nth(suffix_plus_plus!(a)).unwrap();
                if c == '\\' {
                    if a == size {
                        throw!(DbError::get(error_code::FILE_CORRUPTED_1, vec![&format!("not a map: {}",s)]))
                    }

                    c = s.chars().nth(suffix_plus_plus!(a)).unwrap();
                } else if c == '\"' {
                    break;
                }

                result = result.add(&c.to_string());
            }
        } else {
            result = result.add(&c.to_string());
        }
    }

    Ok((result, a))
}