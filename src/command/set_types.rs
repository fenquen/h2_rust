use std::collections::HashSet;
use std::mem;
use std::ops::Index;
use lazy_static::lazy_static;

/// The type of a SET IGNORECASE statement.
pub const IGNORECASE: i32 = 0;

/// The type of a SET MAX_LOG_SIZE statement.
pub const MAX_LOG_SIZE: i32 = IGNORECASE + 1;

/// The type of a SET MODE statement.
pub const MODE: i32 = MAX_LOG_SIZE + 1;

/// The type of a SET READONLY statement.
pub const READONLY: i32 = MODE + 1;

/// The type of a SET LOCK_TIMEOUT statement.
pub const LOCK_TIMEOUT: i32 = READONLY + 1;

/// The type of a SET DEFAULT_LOCK_TIMEOUT statement.
pub const DEFAULT_LOCK_TIMEOUT: i32 = LOCK_TIMEOUT + 1;

/// The type of a SET DEFAULT_TABLE_TYPE statement.
pub const DEFAULT_TABLE_TYPE: i32 = DEFAULT_LOCK_TIMEOUT + 1;

/// The type of a SET CACHE_SIZE statement.
pub const CACHE_SIZE: i32 = DEFAULT_TABLE_TYPE + 1;

/// The type of a SET TRACE_LEVEL_SYSTEM_OUT statement.
pub const TRACE_LEVEL_SYSTEM_OUT: i32 = CACHE_SIZE + 1;

/// The type of a SET TRACE_LEVEL_FILE statement.
pub const TRACE_LEVEL_FILE: i32 = TRACE_LEVEL_SYSTEM_OUT + 1;

/// The type of a SET TRACE_MAX_FILE_SIZE statement.
pub const TRACE_MAX_FILE_SIZE: i32 = TRACE_LEVEL_FILE + 1;

/// The type of a SET COLLATION  statement.
pub const COLLATION: i32 = TRACE_MAX_FILE_SIZE + 1;

/// The type of a SET CLUSTER statement.
pub const CLUSTER: i32 = COLLATION + 1;

/// The type of a SET WRITE_DELAY statement.
pub const WRITE_DELAY: i32 = CLUSTER + 1;

/// The type of a SET DATABASE_EVENT_LISTENER statement.
pub const DATABASE_EVENT_LISTENER: i32 = WRITE_DELAY + 1;

/// The type of a SET MAX_MEMORY_ROWS statement.
pub const MAX_MEMORY_ROWS: i32 = DATABASE_EVENT_LISTENER + 1;

/// The type of a SET LOCK_MODE statement.
pub const LOCK_MODE: i32 = MAX_MEMORY_ROWS + 1;

/// The type of a SET DB_CLOSE_DELAY statement.
pub const DB_CLOSE_DELAY: i32 = LOCK_MODE + 1;

/// The type of a SET THROTTLE statement.
pub const THROTTLE: i32 = DB_CLOSE_DELAY + 1;

/// The type of a SET MAX_MEMORY_UNDO statement.
pub const MAX_MEMORY_UNDO: i32 = THROTTLE + 1;

/// The type of a SET MAX_LENGTH_INPLACE_LOB statement.
pub const MAX_LENGTH_INPLACE_LOB: i32 = MAX_MEMORY_UNDO + 1;

/// The type of a SET ALLOW_LITERALS statement.
pub const ALLOW_LITERALS: i32 = MAX_LENGTH_INPLACE_LOB + 1;

/// The type of a SET SCHEMA statement.
pub const SCHEMA: i32 = ALLOW_LITERALS + 1;

/// The type of a SET OPTIMIZE_REUSE_RESULTS statement.
pub const OPTIMIZE_REUSE_RESULTS: i32 = SCHEMA + 1;

/// The type of a SET SCHEMA_SEARCH_PATH statement.
pub const SCHEMA_SEARCH_PATH: i32 = OPTIMIZE_REUSE_RESULTS + 1;

/// The type of a SET REFERENTIAL_INTEGRITY statement.
pub const REFERENTIAL_INTEGRITY: i32 = SCHEMA_SEARCH_PATH + 1;

/// The type of a SET MAX_OPERATION_MEMORY statement.
pub const MAX_OPERATION_MEMORY: i32 = REFERENTIAL_INTEGRITY + 1;

/// The type of a SET EXCLUSIVE statement.
pub const EXCLUSIVE: i32 = MAX_OPERATION_MEMORY + 1;

/// The type of a SET CREATE_BUILD statement.
pub const CREATE_BUILD: i32 = EXCLUSIVE + 1;

/// The type of a SET \@VARIABLE statement.
pub const VARIABLE: i32 = CREATE_BUILD + 1;

/// The type of a SET QUERY_TIMEOUT statement.
pub const QUERY_TIMEOUT: i32 = VARIABLE + 1;

/// The type of a SET REDO_LOG_BINARY statement.
pub const REDO_LOG_BINARY: i32 = QUERY_TIMEOUT + 1;

/// The type of a SET JAVA_OBJECT_SERIALIZER statement.
pub const JAVA_OBJECT_SERIALIZER: i32 = REDO_LOG_BINARY + 1;

/// The type of a SET RETENTION_TIME statement.
pub const RETENTION_TIME: i32 = JAVA_OBJECT_SERIALIZER + 1;

/// The type of a SET QUERY_STATISTICS statement.
pub const QUERY_STATISTICS: i32 = RETENTION_TIME + 1;

/// The type of a SET QUERY_STATISTICS_MAX_ENTRIES statement.
pub const QUERY_STATISTICS_MAX_ENTRIES: i32 = QUERY_STATISTICS + 1;

/// The type of SET LAZY_QUERY_EXECUTION statement.
pub const LAZY_QUERY_EXECUTION: i32 = QUERY_STATISTICS_MAX_ENTRIES + 1;

/// The type of SET BUILTIN_ALIAS_OVERRIDE statement.
pub const BUILTIN_ALIAS_OVERRIDE: i32 = LAZY_QUERY_EXECUTION + 1;

/// The type of a SET AUTHENTICATOR statement.
pub const AUTHENTICATOR: i32 = BUILTIN_ALIAS_OVERRIDE + 1;

/// The type of a SET IGNORE_CATALOGS statement.
pub const IGNORE_CATALOGS: i32 = AUTHENTICATOR + 1;

/// The type of a SET CATALOG statement.
pub const CATALOG: i32 = IGNORE_CATALOGS + 1;

/// The type of a SET NON_KEYWORDS statement.
pub const NON_KEYWORDS: i32 = CATALOG + 1;

/// The type of a SET TIME ZONE statement.
pub const TIME_ZONE: i32 = NON_KEYWORDS + 1;

/// The type of a SET VARIABLE_BINARY statement.
pub const VARIABLE_BINARY: i32 = TIME_ZONE + 1;

/// The type of a SET DEFAULT_NULL_ORDERING statement.
pub const DEFAULT_NULL_ORDERING: i32 = VARIABLE_BINARY + 1;

/// The type of a SET TRUNCATE_LARGE_LENGTH statement.
pub const TRUNCATE_LARGE_LENGTH: i32 = DEFAULT_NULL_ORDERING + 1;

const COUNT: i32 = TRUNCATE_LARGE_LENGTH + 1;

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

struct A {

}

impl A {
    pub const AA: usize = 1;

    pub fn show() {
        println!("{}", A::AA);
    }
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