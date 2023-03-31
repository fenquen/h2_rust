use anyhow::Result;
use crate::api::error_code;
use crate::message::db_error::DbError;
use crate::store::file_lock_method;
use crate::store::file_lock_method::FileLockMethod;
use crate::throw;

pub struct FileLock {}


pub fn get_file_lock_method(method: &str) -> Result<FileLockMethod> {
    if method.is_empty() || method.eq_ignore_ascii_case("FILE") {
        return Ok(file_lock_method::FILE);
    }

    if method.eq_ignore_ascii_case("NO") {
        return Ok(file_lock_method::NO);
    }

    if method.eq_ignore_ascii_case("SOCKET") {
        return Ok(file_lock_method::SOCKET);
    }

    if method.eq_ignore_ascii_case("FS") {
        return Ok(file_lock_method::FS);
    }

    throw!(DbError::get(error_code::UNSUPPORTED_LOCK_METHOD_1, vec![method]))
}