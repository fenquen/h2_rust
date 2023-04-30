use std::cmp::Ordering;
use std::sync::Arc;
use bytebuffer::ByteBuffer;
use lazy_static::lazy_static;
use crate::build_arc_h2RustCell;
use crate::h2_rust_common::{h2_rust_constant, Integer};
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::h2_rust_type::H2RustType::String;
use crate::mvstore::r#type::basic_data_type::BasicDataType;
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::write_buffer::WriteBuffer;
use crate::h2_rust_common::h2_rust_cell::H2RustCell;

lazy_static! {
    pub static ref INSTANCE:Arc<StringDataType> = Arc::new(StringDataType);
}

pub struct StringDataType;

impl DataType for StringDataType {
    fn compare(&self, a: &H2RustType, b: &H2RustType) -> Ordering {
        match a {
            String(string_a) => {
                match b {
                    String(string_b) => {
                        string_a.get_ref().cmp(string_b.get_ref())
                    }
                    _ => panic!("string_b is not String")
                }
            }
            _ => panic!("string_a is not String")
        }
    }

    fn get_memory(&self, obj: &H2RustType) -> Integer {
        match obj {
            String(string) => { 24 + 2 * string.get_ref().len() as Integer }
            _ => panic!("not String")
        }
    }

    fn write_2(&self, buff: &WriteBuffer, obj: &H2RustType) {
        match obj {
            String(string) => todo!(),
            _ => panic!("not String")
        }
    }

    fn read_1(&self, byteBuffer: &ByteBuffer) -> H2RustType {
        String(build_arc_h2RustCell!(h2_rust_constant::EMPTY_STR.to_string()))
    }

    fn create_storage(&self, size: Integer) -> Vec<H2RustType> {
        Vec::with_capacity(size as usize)
    }
}
/*
fn compare(&self, a: &String, b: &String) -> Ordering {
        a.cmp(b)
    }

    fn get_memory(&self, obj: String) -> Integer {
        24 + 2 * obj.len() as Integer
    }

    fn write_2(&self, buff: &WriteBuffer, obj: &String) {}

    fn read_1(&self, buff: &ByteBuffer) -> String {
        "".to_string()
    }

    fn create_storage(&self, size: Integer) -> Vec<String> {
        Vec::with_capacity(size as usize)
    }*/