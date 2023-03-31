use std::collections::HashSet;
use std::mem;
use std::ops::Index;
use lazy_static::lazy_static;
use crate::h2_rust_common::Integer;

/// The type of a SET IGNORECASE statement.
pub const IGNORECASE: Integer = 0;

/// The type of a SET MAX_LOG_SIZE statement.
pub const MAX_LOG_SIZE: Integer = IGNORECASE + 1;

/// The type of a SET MODE statement.
pub const MODE: Integer = MAX_LOG_SIZE + 1;

/// The type of a SET READONLY statement.
pub const READONLY: Integer = MODE + 1;

/// The type of a SET LOCK_TIMEOUT statement.
pub const LOCK_TIMEOUT: Integer = READONLY + 1;

/// The type of a SET DEFAULT_LOCK_TIMEOUT statement.
pub const DEFAULT_LOCK_TIMEOUT: Integer = LOCK_TIMEOUT + 1;

/// The type of a SET DEFAULT_TABLE_TYPE statement.
pub const DEFAULT_TABLE_TYPE: Integer = DEFAULT_LOCK_TIMEOUT + 1;

/// The type of a SET CACHE_SIZE statement.
pub const CACHE_SIZE: Integer = DEFAULT_TABLE_TYPE + 1;

/// The type of a SET TRACE_LEVEL_SYSTEM_OUT statement.
pub const TRACE_LEVEL_SYSTEM_OUT: Integer = CACHE_SIZE + 1;

/// The type of a SET TRACE_LEVEL_FILE statement.
pub const TRACE_LEVEL_FILE: Integer = TRACE_LEVEL_SYSTEM_OUT + 1;

/// The type of a SET TRACE_MAX_FILE_SIZE statement.
pub const TRACE_MAX_FILE_SIZE: Integer = TRACE_LEVEL_FILE + 1;

/// The type of a SET COLLATION  statement.
pub const COLLATION: Integer = TRACE_MAX_FILE_SIZE + 1;

/// The type of a SET CLUSTER statement.
pub const CLUSTER: Integer = COLLATION + 1;

/// The type of a SET WRITE_DELAY statement.
pub const WRITE_DELAY: Integer = CLUSTER + 1;

/// The type of a SET DATABASE_EVENT_LISTENER statement.
pub const DATABASE_EVENT_LISTENER: Integer = WRITE_DELAY + 1;

/// The type of a SET MAX_MEMORY_ROWS statement.
pub const MAX_MEMORY_ROWS: Integer = DATABASE_EVENT_LISTENER + 1;

/// The type of a SET LOCK_MODE statement.
pub const LOCK_MODE: Integer = MAX_MEMORY_ROWS + 1;

/// The type of a SET DB_CLOSE_DELAY statement.
pub const DB_CLOSE_DELAY: Integer = LOCK_MODE + 1;

/// The type of a SET THROTTLE statement.
pub const THROTTLE: Integer = DB_CLOSE_DELAY + 1;

/// The type of a SET MAX_MEMORY_UNDO statement.
pub const MAX_MEMORY_UNDO: Integer = THROTTLE + 1;

/// The type of a SET MAX_LENGTH_INPLACE_LOB statement.
pub const MAX_LENGTH_INPLACE_LOB: Integer = MAX_MEMORY_UNDO + 1;

/// The type of a SET ALLOW_LITERALS statement.
pub const ALLOW_LITERALS: Integer = MAX_LENGTH_INPLACE_LOB + 1;

/// The type of a SET SCHEMA statement.
pub const SCHEMA: Integer = ALLOW_LITERALS + 1;

/// The type of a SET OPTIMIZE_REUSE_RESULTS statement.
pub const OPTIMIZE_REUSE_RESULTS: Integer = SCHEMA + 1;

/// The type of a SET SCHEMA_SEARCH_PATH statement.
pub const SCHEMA_SEARCH_PATH: Integer = OPTIMIZE_REUSE_RESULTS + 1;

/// The type of a SET REFERENTIAL_INTEGRITY statement.
pub const REFERENTIAL_INTEGRITY: Integer = SCHEMA_SEARCH_PATH + 1;

/// The type of a SET MAX_OPERATION_MEMORY statement.
pub const MAX_OPERATION_MEMORY: Integer = REFERENTIAL_INTEGRITY + 1;

/// The type of a SET EXCLUSIVE statement.
pub const EXCLUSIVE: Integer = MAX_OPERATION_MEMORY + 1;

/// The type of a SET CREATE_BUILD statement.
pub const CREATE_BUILD: Integer = EXCLUSIVE + 1;

/// The type of a SET \@VARIABLE statement.
pub const VARIABLE: Integer = CREATE_BUILD + 1;

/// The type of a SET QUERY_TIMEOUT statement.
pub const QUERY_TIMEOUT: Integer = VARIABLE + 1;

/// The type of a SET REDO_LOG_BINARY statement.
pub const REDO_LOG_BINARY: Integer = QUERY_TIMEOUT + 1;

/// The type of a SET JAVA_OBJECT_SERIALIZER statement.
pub const JAVA_OBJECT_SERIALIZER: Integer = REDO_LOG_BINARY + 1;

/// The type of a SET RETENTION_TIME statement.
pub const RETENTION_TIME: Integer = JAVA_OBJECT_SERIALIZER + 1;

/// The type of a SET QUERY_STATISTICS statement.
pub const QUERY_STATISTICS: Integer = RETENTION_TIME + 1;

/// The type of a SET QUERY_STATISTICS_MAX_ENTRIES statement.
pub const QUERY_STATISTICS_MAX_ENTRIES: Integer = QUERY_STATISTICS + 1;

/// The type of SET LAZY_QUERY_EXECUTION statement.
pub const LAZY_QUERY_EXECUTION: Integer = QUERY_STATISTICS_MAX_ENTRIES + 1;

/// The type of SET BUILTIN_ALIAS_OVERRIDE statement.
pub const BUILTIN_ALIAS_OVERRIDE: Integer = LAZY_QUERY_EXECUTION + 1;

/// The type of a SET AUTHENTICATOR statement.
pub const AUTHENTICATOR: Integer = BUILTIN_ALIAS_OVERRIDE + 1;

/// The type of a SET IGNORE_CATALOGS statement.
pub const IGNORE_CATALOGS: Integer = AUTHENTICATOR + 1;

/// The type of a SET CATALOG statement.
pub const CATALOG: Integer = IGNORE_CATALOGS + 1;

/// The type of a SET NON_KEYWORDS statement.
pub const NON_KEYWORDS: Integer = CATALOG + 1;

/// The type of a SET TIME ZONE statement.
pub const TIME_ZONE: Integer = NON_KEYWORDS + 1;

/// The type of a SET VARIABLE_BINARY statement.
pub const VARIABLE_BINARY: Integer = TIME_ZONE + 1;

/// The type of a SET DEFAULT_NULL_ORDERING statement.
pub const DEFAULT_NULL_ORDERING: Integer = VARIABLE_BINARY + 1;

/// The type of a SET TRUNCATE_LARGE_LENGTH statement.
pub const TRUNCATE_LARGE_LENGTH: Integer = DEFAULT_NULL_ORDERING + 1;

const COUNT: Integer = TRUNCATE_LARGE_LENGTH + 1;

lazy_static! {
    pub static ref TYPES: Vec<&'static str> = {
       let mut types =  Vec::<&str>::with_capacity(COUNT as usize);
        types.push("IGNORECASE");
        types.push("MAX_LOG_SIZE");
        types.push("MODE");
        types.push("READONLY");
        types.push("LOCK_TIMEOUT");
        types.push("DEFAULT_LOCK_TIMEOUT");
        types.push("DEFAULT_TABLE_TYPE");
        types.push("CACHE_SIZE");
        types.push("TRACE_LEVEL_SYSTEM_OUT");
        types.push("TRACE_LEVEL_FILE");
        types.push("TRACE_MAX_FILE_SIZE");
        types.push("COLLATION");
        types.push("CLUSTER");
        types.push("WRITE_DELAY");
        types.push("DATABASE_EVENT_LISTENER");
        types.push("MAX_MEMORY_ROWS");
        types.push("LOCK_MODE");
        types.push("DB_CLOSE_DELAY");
        types.push("THROTTLE");
        types.push("MAX_MEMORY_UNDO");
        types.push("MAX_LENGTH_INPLACE_LOB");
        types.push("ALLOW_LITERALS");
        types.push("SCHEMA");
        types.push("OPTIMIZE_REUSE_RESULTS");
        types.push("SCHEMA_SEARCH_PATH");
        types.push("REFERENTIAL_INTEGRITY");
        types.push("MAX_OPERATION_MEMORY");
        types.push("EXCLUSIVE");
        types.push("CREATE_BUILD");
        types.push("@");
        types.push("QUERY_TIMEOUT");
        types.push("REDO_LOG_BINARY");
        types.push("JAVA_OBJECT_SERIALIZER");
        types.push("RETENTION_TIME");
        types.push("QUERY_STATISTICS");
        types.push("QUERY_STATISTICS_MAX_ENTRIES");
        types.push("LAZY_QUERY_EXECUTION");
        types.push("BUILTIN_ALIAS_OVERRIDE");
        types.push("AUTHENTICATOR");
        types.push("IGNORE_CATALOGS");
        types.push("CATALOG");
        types.push("NON_KEYWORDS");
        types.push("TIME ZONE");
        types.push("VARIABLE_BINARY");
        types.push("DEFAULT_NULL_ORDERING");
        types.push("TRUNCATE_LARGE_LENGTH");
        types
    };
}

pub fn get_type(name: &str) -> usize {
    match TYPES.iter().enumerate().find(|(a, &x)| x.eq(name)) {
        Some((index, _)) => index,
        None => 1,
    }
}

pub fn get_types() -> &'static Vec<&'static str> {
    &*TYPES
}

pub fn get_type_name(index: usize) -> &'static str {
    TYPES.get(index).unwrap()
}

mod test {
    use std::ops::Index;
    use crate::command::set_types::{get_type, get_types, TYPES};

    #[test]
    fn a() {
        let a = &*TYPES;
        println!("{}", get_type("@"));
    }
}