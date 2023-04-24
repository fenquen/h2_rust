use bytebuffer::ByteBuffer;
use crate::h2_rust_common::h2_rust_type::H2RustType;
use crate::h2_rust_common::Integer;
use crate::mvstore::r#type::data_type::DataType;
use crate::mvstore::write_buffer::WriteBuffer;


pub trait BasicDataType: DataType {

    fn write(&self, write_buffer: WriteBuffer, obj: H2RustType);

    fn read(&self, byte_buffer: ByteBuffer) -> H2RustType;
}