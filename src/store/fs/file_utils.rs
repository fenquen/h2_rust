use anyhow::Result;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::api::error_code;
use crate::message::db_error::DbError;
use crate::throw;

pub fn is_absolute(file_name: &str) -> bool {
    let path = Path::new(file_name);
    path.is_absolute()
}

pub fn to_real_path(file_name: impl AsRef<Path>) -> Result<PathBuf> {
    let path_buf = fs::canonicalize(file_name)?;
    Ok(path_buf)
}

pub fn get_name(path: impl AsRef<Path>) -> String {
    let os_str = path.as_ref().file_name().unwrap();
    os_str.to_str().unwrap().to_string()
}

pub fn exist(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn can_write(path: impl AsRef<Path>) -> bool {
    if let Ok(f) = File::open(path) {
        if let Ok(meta) = f.metadata() {
            !meta.permissions().readonly()
        } else {
            false
        }
    } else {
        false
    }
}

pub fn get_parent(path: impl AsRef<Path>) -> Result<PathBuf> {
    match path.as_ref().parent() {
        Some(p) => Ok(p.to_path_buf()),
        None => throw!(DbError::get(error_code::GENERAL_ERROR_1,
                                 vec![&format!("can't get the parent path :{}", path.as_ref().to_str().unwrap())]))
    }
}

/// 得到当前目录下的条目的path
pub fn new_directory_stream(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let mut vec = Vec::new();
    let read_dir = fs::read_dir(path.as_ref())?;
    for dir_entry_result in read_dir {
        let dir_entry = dir_entry_result?;
        vec.push(dir_entry.path().to_str().unwrap().to_string());
    }

    Ok(vec)
}

pub fn try_delete(path: impl AsRef<Path>) -> bool {
    if let Ok(()) = fs::remove_file(path.as_ref()) {
        true
    } else {
        false
    }
}

mod test {
    use std::fs;
    use crate::store::fs::file_utils::new_directory_stream;

    #[test]
    pub fn test_read_dir() {
        for a in new_directory_stream("/Users/a").unwrap() {
            println!("{}", a);
        }
    }
}