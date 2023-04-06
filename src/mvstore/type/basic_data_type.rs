use bytebuffer::ByteBuffer;
use crate::h2_rust_common::Integer;
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::write_buffer::WriteBuffer;


pub trait BasicDataType<T>: DataType<T> {
    fn get_memory(&self, obj: T) -> Integer;

    fn write(&self, write_buffer: WriteBuffer, obj: T);

    fn read(&self, byte_buffer: ByteBuffer) -> T;

}