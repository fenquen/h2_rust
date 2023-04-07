use std::cmp::Ordering;
use std::sync::Arc;
use bytebuffer::ByteBuffer;
use lazy_static::lazy_static;
use crate::h2_rust_common::Integer;
use crate::mvstore::r#type::basic_data_type::BasicDataType;
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::write_buffer::WriteBuffer;

lazy_static! {
    pub static ref INSTANCE:Arc<StringDataType> = Arc::new(StringDataType);
}

pub struct StringDataType;

impl DataType<String> for StringDataType {
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
    }
}