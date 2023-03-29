use anyhow::Result;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

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