use crate::h2_rust_common::Integer;

pub const START_URL: &str = "jdbc:h2:";
pub const URL_FORMAT: &str = "jdbc:h2:{ {.|mem:}[name] | [file:]fileName | {tcp|ssl}:[//]server[:port][,server2[:port]]/name }";

pub const SUFFIX_MV_FILE: &str = ".mv.db";


pub const SUFFIX_OLD_DATABASE_FILE: &str = ".data.db";
pub const DEFAULT_MAX_LENGTH_INPLACE_LOB: Integer = 256;
pub const DEFAULT_PAGE_SIZE: Integer = 4096;
pub const MAX_IDENTIFIER_LENGTH:Integer=256;