use anyhow::Result;
use crate::api::error_code;
use crate::message::db_error::DbError;
use crate::store::file_lock_method::FileLockMethod;
use crate::throw;

pub struct FileLock {}


pub fn get_file_lock_method(method: &str) -> Result<FileLockMethod> {
    if method.is_empty() || method.eq_ignore_ascii_case("FILE") {
        return Ok(FileLockMethod::FILE);
    }

    if method.eq_ignore_ascii_case("NO") {
        return Ok(FileLockMethod::NO);
    }

    if method.eq_ignore_ascii_case("SOCKET") {
        return Ok(FileLockMethod::SOCKET);
    }

    if method.eq_ignore_ascii_case("FS") {
        return Ok(FileLockMethod::FS);
    }

    throw!(DbError::get(error_code::UNSUPPORTED_LOCK_METHOD_1, vec![method]))
}