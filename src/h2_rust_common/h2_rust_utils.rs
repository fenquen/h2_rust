use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{is_separator, Path};
use std::time::SystemTime;
use crate::properties_type;
use anyhow::Result;
use sys_info::MemInfo;
use crate::api::error_code;
use crate::h2_rust_common::{h2_rust_constant as common_constant, Integer, Long, Nullable, Properties};
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

pub fn cast<T: 'static>(object: Option<Box<dyn Any>>) -> Option<Box<T>> {
    match object {
        Some(ref b) => {
            match (&**b).downcast_ref::<T>() {
                Some(s) => {}
                None => return None
            }
        }
        None => return None
    };

    let b = object.unwrap();
    let d = Box::leak(b);

    Some(unsafe { Box::from_raw(d as *mut dyn Any as *mut T) })
}


pub fn getTotalPhysicalMemorySize() -> Result<Long> {
    match sys_info::mem_info() {
        Ok(mem_info) => Ok(mem_info.total as Long * 1024),// 原始得到的kb
        Err(e) => Err(anyhow::Error::from(e))
    }
}

pub fn getTimestamp() -> Long {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as Long
}

mod test {
    use std::any::Any;
    use std::collections::HashMap;
    use crate::h2_rust_common::h2_rust_utils::{cast, integer_decode};

    #[test]
    fn integer_decode_test() {
        match integer_decode("s") {
            Ok(a) => println!("{}", a),
            Err(e) => println!("{}", e)
        }
    }

    #[test]
    fn test_get_memory() {
        use sys_info;
        match sys_info::mem_info() {
            Ok(mem_info) => { println!("{}", mem_info.total) }
            Err(e) => {}
        }
    }

    #[test]
    fn test_cast() {
        let mut map = HashMap::<String, Box<dyn Any>>::new();
        map.insert("a".to_string(), Box::new("1".to_string()));

        let a = Some(Box::new("a".to_string()) as Box<dyn Any>);
        let a = cast::<String>(a);
        println!("{}", a.unwrap());
    }
}