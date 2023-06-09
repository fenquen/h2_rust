use std::sync::Arc;
use crate::{enum_str, get_ref};
use crate::h2_rust_common::{Integer, Void};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;

pub enum H2RustType {
    String(Arc<H2RustCell<String>>),
    Integer(Integer),
    Null,
}

impl Clone for H2RustType {
    fn clone(&self) -> Self {
        match self {
            H2RustType::String(a) => { H2RustType::String(a.clone()) }
            H2RustType::Integer(a) => { H2RustType::Integer(a.clone()) }
            H2RustType::Null => { H2RustType::Null }
        }
    }
}

impl H2RustType {
    pub fn castAsStringRef(&self) -> &String {
        match self {
            Self::String(s) => { s.get_ref() }
            _ => {
                panic!("need string")
            }
        }
    }

    pub fn isNull(&self) -> bool {
        match self {
            H2RustType::Null => true,
            _ => false
        }
    }

    pub fn toString(&self) -> Option<String> {
        match self {
            H2RustType::String(a) => Some(a.get_ref().clone()),
            H2RustType::Null => None,
            _ => panic!("not H2RustType::String")
        }
    }
}

