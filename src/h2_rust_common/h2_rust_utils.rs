use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{is_separator, Path};
use crate::properties_type;
use anyhow::Result;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_constant as common_constant, Integer, Nullable, Properties};
use crate::h2_rust_common::h2_rust_constant;
use crate::h2_rust_common::Nullable::{NotNull, Null};
use crate::message::db_error::DbError;

pub fn load_properties(prop_file_path: &Path) -> Result<Properties> {
    let prop_file = File::open(prop_file_path)?;
    let buf_reader = BufReader::new(prop_file);

    let mut map = HashMap::<String, String>::new();

    for line_result in buf_reader.lines() {
        let line = line_result?;
        let element_vec: Vec<_> = line.split(common_constant::EQUAL).collect();
        map.insert(element_vec[0].to_string(), element_vec[1].to_string());
    }

    Ok(map)
}

pub fn add_all(a: &mut HashSet<String>, b: &[impl ToString]) {
    for b0 in b {
        a.insert(b0.to_string());
    }
}

pub fn a() {}

/// 对应了java的Integer.decode()
pub fn integer_decode(s: &str) -> Result<Integer> {
    let mut negative = false;

    if s.starts_with(common_constant::NEGATIVE) {
        negative = true;
    }

    // 去掉打头的+ -
    let s1 = s.trim_start_matches(|c: char| c.eq(&common_constant::POSITIVE) || c.eq(&common_constant::NEGATIVE));

    let result = {
        if s1.starts_with("0x") {
            let s2 = s1.trim_start_matches("0x");
            Integer::from_str_radix(s2, 16)
        } else if s1.starts_with("0X") {
            let s2 = s1.trim_start_matches("0X");
            Integer::from_str_radix(s2, 16)
        } else if s1.starts_with("0") {
            let ss = s1.trim_start_matches("0");
            Integer::from_str_radix(ss, 8)
        } else {
            Integer::from_str_radix(s1, 10)
        }
    };

    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            Err(DbError::get(error_code::DATA_CONVERSION_ERROR_1, vec![&format!("covert text {} to number failed", s)]))?
        }
    }
}

/// 以前是为了偷懒简便 不去区分是""还是null都是使用的""
pub fn get_from_map<T: Clone + 'static>(map: &HashMap<String, Box<dyn Any>>, key: &str) -> Option<T> {
    match map.get(key) {
        Some(b) => {
            match (&**b).downcast_ref::<T>() {
                Some(s) => {
                    Some(s.clone())
                }
                None => None
            }
        }
        None => {
            None
        }
    }
}

mod test {
    use crate::h2_rust_common::h2_rust_utils::integer_decode;

    #[test]
    fn integer_decode_test() {
        match integer_decode("s") {
            Ok(a) => println!("{}", a),
            Err(e) => println!("{}", e)
        }
    }
}