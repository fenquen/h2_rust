use std::borrow::ToOwned;

pub const START_URL: &str = "jdbc:h2:";
pub const URL_FORMAT: &str = "jdbc:h2:{ {.|mem:}[name] | [file:]fileName | {tcp|ssl}:[//]server[:port][,server2[:port]]/name }";