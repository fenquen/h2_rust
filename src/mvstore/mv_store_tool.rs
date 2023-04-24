use anyhow::Result;
use std::ops::Add;
use crate::engine::constant;
use crate::store::fs::file_utils;

pub fn compact_clean_up(file_path: &str) -> Result<()> {
    let temp_path = file_path.to_string().add(constant::SUFFIX_MV_STORE_TEMP_FILE);
    if file_utils::exist(&temp_path) {
        file_utils::delete(&temp_path)?;
    }

    let new_path = file_path.to_string().add(constant::SUFFIX_MV_STORE_NEW_FILE);
    if file_utils::exist(&new_path) {
        if file_utils::exist(&file_path) {
            file_utils::delete(&new_path)?;
        } else {
            file_utils::move1(&new_path, &file_path)?;
        }
    }

    Ok(())
}