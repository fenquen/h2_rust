use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::ops::Add;
use std::os::unix::fs::FileExt;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_constant, h2_rust_utils, Integer, Long, ULong};
use crate::message::db_error::DbError;
use crate::{suffix_plus_plus, throw, unsigned_right_shift};
use crate::h2_rust_common::byte_buffer::ByteBuffer;

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

pub fn getPageChunkId(position: Long) -> Integer {
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

pub fn readHexIntOrLong<T: 'static + Display, TargetType: Copy + 'static>(map: &HashMap<String, T>,
                                                                          key: &str,
                                                                          defaultValue: TargetType) -> Result<TargetType>
    where Long: Convertable<TargetType> {
    let valueOption = map.get(key);
    if valueOption.is_none() {
        return Ok(defaultValue);
    }

    let value = valueOption.unwrap();
    let valueDyn = value as &dyn Any;

    match valueDyn.downcast_ref::<TargetType>() {
        Some(targetTypeRef) => return Ok(*targetTypeRef),
        _ => {}
    }

    match valueDyn.downcast_ref::<String>() {
        Some(string) => {
            match Long::from_str_radix(string, 16) {
                Ok(long) => Ok(long.convert()),
                Err(e) => {
                    throw!(DbError::get(error_code::FILE_CORRUPTED_1, vec![&format!("error parsing the value {}", value)]))
                }
            }
        }
        _ => {
            throw!(DbError::get(error_code::FILE_CORRUPTED_1, vec![&format!("error parsing the value {}", value)]))
        }
    }
}

pub trait Convertable<TargetType: Copy> {
    fn convert(&self) -> TargetType;
}

impl Convertable<Long> for Long {
    fn convert(&self) -> Long {
        *self
    }
}

impl Convertable<Integer> for Long {
    fn convert(&self) -> Integer {
        *self as Integer
    }
}

/// Get the offset from the position <br>
/// @param tocElement packed table of content element
pub fn getPageOffset(tocElement: Long) -> Integer {
    (tocElement >> 6) as Integer
}

/// get the maximum length for the given page position.
pub fn getPageMaxLength(position: Long) -> Integer {
    let code = ((position >> 1) & 31) as Integer;
    decodePageLength(code)
}


/// Get the maximum length for the given code,For the code 31, PAGE_LARGE is returned <br>
/// return the maximum length
pub fn decodePageLength(encodedPageLength: Integer) -> Integer {
    if encodedPageLength == 31 {
        PAGE_LARGE
    } else {
        (2 + (encodedPageLength & 1)) << ((encodedPageLength >> 1) + 4)
    }
}

pub fn readFully(file: &File, mut position: usize, byteBuffer: &mut ByteBuffer) -> Result<()> {
    // java中fileChannel.read(byteBuffer)用的也是如下的套路
    // 也是找了个中间的buffer 然后再让真正的dest去吸取
    let mut a = Vec::<u8>::with_capacity(byteBuffer.getCapacity());
    let aa: &mut [u8] = &mut a;

    loop {
        let len = file.read_at(aa, position as u64)?;
        byteBuffer.putSlice_(aa, 0, len);

        position += len;

        if !byteBuffer.hasRemaining() {
            break;
        }
    }

    byteBuffer.rewind();

    Ok(())
}

pub fn writeFully(file: &File, mut position: usize, src: &mut ByteBuffer) -> Result<()> {
    loop {
        let slice = src.extract();
        let len = file.write_at(slice, position as u64)?;
        position += len;
        src.advance(len);

        if !src.hasRemaining() {
            break;
        }
    }

    Ok(())
}

pub fn getPageType(position: Long) -> Integer {
    (position as Integer) & 1
}

/// Calculate a check value for the given integer. A check value is mean to
/// verify the data is consistent with a high probability, but not meant to
/// protect against media failure or deliberate changes.
pub fn getCheckValue(x: Integer) -> i16 {
    ((x >> 16) ^ x) as i16
}

pub fn readVarInt(byteBuffer: &mut ByteBuffer) -> i32 {
    let b = byteBuffer.getI8() as i32;
    if b >= 0 {
        return b;
    }

    // a separate function so that this one can be inlined
    return readVarIntRest(byteBuffer, b);
}

pub fn readVarIntRest(byteBuffer: &mut ByteBuffer, mut b: i32) -> i32 {
    let mut x = b & 0x7f;
    b = byteBuffer.getI8() as i32;
    if b >= 0 {
        return x | (b << 7);
    }

    x |= (b & 0x7f) << 7;
    b = byteBuffer.getI8() as i32;
    if b >= 0 {
        return x | (b << 14);
    }

    x |= (b & 0x7f) << 14;
    b = byteBuffer.getI8() as i32;
    if b >= 0 {
        return x | b << 21;
    }

    x |= ((b & 0x7f) << 21) | ((byteBuffer.getI8() as i32) << 28);

    x
}

pub fn readVarLong(bytebuffer: &mut ByteBuffer) -> i64 {
    let mut x: i64 = bytebuffer.getI8() as i64;
    if x >= 0 {
        return x;
    }

    x &= 0x7f;

    let mut s = 7;
    loop {
        if s >= 64 {
            break;
        }

        let b: i64 = bytebuffer.getI8() as i64;
        x |= (b & 0x7f) << s;

        if b >= 0 {
            break;
        }

        s += 7
    }

    x
}

pub fn readString1(byteBuffer: &mut ByteBuffer) -> String {
    let len = readVarInt(byteBuffer) as usize;
    readString2(byteBuffer, len)
}

pub fn readString2(byteBuffer: &mut ByteBuffer, len: usize) -> String {
    let mut chars: Vec<u8> = Vec::with_capacity(len);
    for a in 0..len {
        let x = byteBuffer.getI8() as i32 & 0xff;
        if x < 0x80 {
            chars[a] = x as u8;
        } else if x >= 0xe0 {
            chars[a] = ((x & 0xf) << 12 + (byteBuffer.getI8() & 0x3f) << 6 + byteBuffer.getI8() & 0x3f) as u8;
        } else {
            chars[a] = ((x & 0x1f) << 6 + byteBuffer.getI8() & 0x3f) as u8;
        }
    }
    String::from_utf8(chars).unwrap()
}

