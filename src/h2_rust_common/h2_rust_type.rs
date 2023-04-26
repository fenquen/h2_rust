use crate::enum_str;
use crate::h2_rust_common::Integer;

enum_str! {pub enum H2RustType {
    String(String),
    Integer(Integer),
}}

impl H2RustType {
    pub fn cast_string(&self) -> H2RustType {
        match self {
            Self::String(s) => { Self::String(s.clone()) }
            _ => { panic!() }
        }
    }
}

