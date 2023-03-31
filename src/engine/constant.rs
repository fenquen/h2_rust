use crate::h2_rust_common::{h2_rust_constant, Integer, Long};

/// The build date is updated for each pub release.
pub const BUILD_DATE: &str = "2022-06-13";

/// Sequential version number. Even numbers are used for official releases,
/// odd numbers are used for development builds.
pub const BUILD_ID: Integer = 214;

/// Whether this is a snapshot version.
pub const BUILD_SNAPSHOT: bool = false;

/// If H2 is compiled to be included in a product, this should be set to a unique vendor id (to distinguish from official releases).
/// Additionally, a version number should be set to distinguish releases.
///
/// Example: ACME_SVN1651_BUILD3
pub const BUILD_VENDOR_AND_VERSION: &str = h2_rust_constant::EMPTY_STR;

/// The TCP protocol version number 17.
///
/// @since 1.4.197 (2018-03-18)
pub const TCP_PROTOCOL_VERSION_17: Integer = 17;

/// The TCP protocol version number 18.
///
/// @since 1.4.198 (2019-02-22)
pub const TCP_PROTOCOL_VERSION_18: Integer = 18;

/// The TCP protocol version number 19.
///
/// @since 1.4.200 (2019-10-14)
pub const TCP_PROTOCOL_VERSION_19: Integer = 19;

/// The TCP protocol version number 20.
///
/// @since 2.0.202 (2021-11-25)
pub const TCP_PROTOCOL_VERSION_20: Integer = 20;

/// Minimum supported version of TCP protocol.
pub const TCP_PROTOCOL_VERSION_MIN_SUPPORTED: Integer = TCP_PROTOCOL_VERSION_17;

/// Maximum supported version of TCP protocol.
pub const TCP_PROTOCOL_VERSION_MAX_SUPPORTED: Integer = TCP_PROTOCOL_VERSION_20;

/// The major version of this database.
pub const VERSION_MAJOR: Integer = 2;

/// The minor version of this database.
pub const VERSION_MINOR: Integer = 1;

/// The lock mode that means no locking is used at all.
pub const LOCK_MODE_OFF: Integer = 0;

/// The lock mode that means read locks are acquired, but they are released immediately after the statement is executed.
pub const LOCK_MODE_READ_COMMITTED: Integer = 3;

/// The lock mode that means table level locking is used for reads and writes.
pub const LOCK_MODE_TABLE: Integer = 1;

/// The lock mode that means table level locking is used for reads and
/// writes. If a table is locked, System.gc is called to close forgotten connections.
pub const LOCK_MODE_TABLE_GC: Integer = 2;

/// Constant meaning both numbers and text is allowed in SQL statements.
pub const ALLOW_LITERALS_ALL: Integer = 2;

/// Constant meaning no literals are allowed in SQL statements.
pub const ALLOW_LITERALS_NONE: Integer = 0;

/// Constant meaning only numbers are allowed in SQL statements (but no texts).
pub const ALLOW_LITERALS_NUMBERS: Integer = 1;

/// SNAPSHOT isolation level of transaction.
pub const TRANSACTION_SNAPSHOT: Integer = 6;

/// Whether searching in Blob values should be supported.
pub const BLOB_SEARCH: bool = false;

/// The minimum number of entries to keep in the cache.
pub const CACHE_MIN_RECORDS: Integer = 16;

/// The default cache type.
pub const CACHE_TYPE_DEFAULT: &str = "LRU";


/// The value of the cluster setting if clustering is disabled.
pub const CLUSTERING_DISABLED: &str = "''";


/// The value of the cluster setting if clustering is enabled (the actual value is checked later).
pub const CLUSTERING_ENABLED: &str = "TRUE";

/// The database URL used when calling a function if only the column list should be returned.
pub const CONN_URL_COLUMNLIST: &str = "jdbc:columnlist:connection";

/// The database URL used when calling a function if the data should be returned.
pub const CONN_URL_INTERNAL: &str = "jdbc:default:connection";

/**
 * The cost is calculated on rowcount + this offset,
 * to avoid using the wrong or no index if the table
 * contains no rows _currently_ (when preparing the statement)
 */
pub const COST_ROW_OFFSET: Integer = 1000;

/**
 * The number of milliseconds after which to check for a deadlock if locking
 * is not successful.
 */
pub const DEADLOCK_CHECK: Integer = 100;

/**
 * The default port number of the HTTP server (for the H2 Console).
 * This value is also in the documentation and in the Server javadoc.
 */
pub const DEFAULT_HTTP_PORT: Integer = 8082;

/**
 * The default value for the LOCK_MODE setting.
 */
pub const DEFAULT_LOCK_MODE: Integer = LOCK_MODE_READ_COMMITTED;

/**
 * The default maximum length of an LOB that is stored with the record
 * itself, and not in a separate place.
 */
pub const DEFAULT_MAX_LENGTH_INPLACE_LOB: Integer = 256;

/**
 * The default for the setting MAX_OPERATION_MEMORY.
 */
pub const DEFAULT_MAX_OPERATION_MEMORY: Integer = 100_000;

/**
 * The default page size to use for new databases.
 */
pub const DEFAULT_PAGE_SIZE: Integer = 4096;

/**
 * The default result set concurrency for statements created with
 * Connection.createStatement() or prepareStatement(String sql).
 */
pub const DEFAULT_RESULT_SET_CONCURRENCY: Integer = 1007;

/**
 * The default port of the TCP server.
 * This port is also used in the documentation and in the Server javadoc.
 */
pub const DEFAULT_TCP_PORT: Integer = 9092;

/**
 * The default delay in milliseconds before the transaction log is written.
 */
pub const DEFAULT_WRITE_DELAY: Integer = 500;

/**
 * The password is hashed this many times
 * to slow down dictionary attacks.
 */
pub const ENCRYPTION_KEY_HASH_ITERATIONS: Integer = 1024;

/**
 * The block of a file. It is also the encryption block size.
 */
pub const FILE_BLOCK_SIZE: Integer = 16;

/**
 * For testing, the lock timeout is smaller than for interactive use cases.
 * This value could be increased to about 5 or 10 seconds.
 */
pub const INITIAL_LOCK_TIMEOUT: Integer = 2000;

/**
 * The block size for I/O operations.
 */
pub const IO_BUFFER_SIZE: Integer = 4 * 1024;

/**
 * The block size used to compress data in the LZFOutputStream.
 */
pub const IO_BUFFER_SIZE_COMPRESS: Integer = 128 * 1024;

/**
 * The number of milliseconds to wait between checking the .lock.db file
 * still exists once a database is locked.
 */
pub const LOCK_SLEEP: Integer = 1000;

/**
 * The maximum allowed length of identifiers.
 */
pub const MAX_IDENTIFIER_LENGTH: Integer = 256;

/**
 * The maximum number of columns in a table, select statement or row value.
 */
pub const MAX_COLUMNS: Integer = 16_384;

/**
 * The maximum allowed length for character string, binary string, and other
 * data types based on them; excluding LOB data types.
 * <p>
 * This needs to be less than (2^31-8)/2 to avoid running into the limit on
 * encoding data fields when storing rows.
 */
pub const MAX_STRING_LENGTH: Integer = 1000_000_000;

/**
 * The maximum allowed precision of numeric data types.
 */
pub const MAX_NUMERIC_PRECISION: Integer = 100_000;

/**
 * The maximum allowed cardinality of array.
 */
pub const MAX_ARRAY_CARDINALITY: Integer = 65_536;

/**
 * The highest possible parameter index.
 */
pub const MAX_PARAMETER_INDEX: Integer = 100_000;

/// The memory needed by a regular object with at least one field.
///
/// Java 6, 64 bit: 24
/// Java 6, 32 bit: 12
pub const MEMORY_OBJECT: Integer = 24;

/**
 * The memory needed by an array.
 */
pub const MEMORY_ARRAY: Integer = 24;

/**
 * The memory needed by a pointer.
 */
// Java 6, 64 bit: 8
// Java 6, 32 bit: 4
pub const MEMORY_POINTER: Integer = 8;

/// The memory needed by a Row.
pub const MEMORY_ROW: Integer = 40;

/// The name prefix used for indexes that are not explicitly named.
pub const PREFIX_INDEX: &str = "INDEX_";

/// The name prefix used for synthetic nested join tables.
pub const PREFIX_JOIN: &str = "SYSTEM_JOIN_";

/// The name prefix used for primary key constraints that are not explicitly named.
pub const PREFIX_PRIMARY_KEY: &str = "PRIMARY_KEY_";

/// The name prefix used for query aliases that are not explicitly named.
pub const PREFIX_QUERY_ALIAS: &str = "QUERY_ALIAS_";

/// Every user belongs to this role.
pub const PUB_ROLE_NAME: &str = "pub";

/**
 * The number of bytes in random salt that is used to hash passwords.
 */
pub const SALT_LEN: Integer = 8;

/**
 * The identity of INFORMATION_SCHEMA.
 */
pub const INFORMATION_SCHEMA_ID: Integer = -1;

/**
 * The identity of pub schema.
 */
pub const MAIN_SCHEMA_ID: Integer = 0;

/**
 * The name of the default schema.
 */
pub const SCHEMA_MAIN: &str = "pub";

/**
 * The identity of pg_catalog schema.
 */
pub const PG_CATALOG_SCHEMA_ID: Integer = -1_000;

/**
 * The name of the pg_catalog schema.
 */
pub const SCHEMA_PG_CATALOG: &str = "PG_CATALOG";

/// The default selectivity (used if the selectivity is not calculated).
pub const SELECTIVITY_DEFAULT: Integer = 50;

/// The number of distinct values to keep in memory when running ANALYZE.
pub const SELECTIVITY_DISTINCT_COUNT: Integer = 10_000;

/// The default directory name of the server properties file for the H2 Console.
pub const SERVER_PROPERTIES_DIR: &str = "~";

/// The name of the server properties file for the H2 Console.
pub const SERVER_PROPERTIES_NAME: &str = ".h2.server.properties";

/// Queries that take longer than this number of milliseconds are written to the trace file with the level info.
pub const SLOW_QUERY_LIMIT_MS: Long = 100;

/// The database URL prefix of this database.
pub const START_URL: &str = "jdbc:h2:";

/// The file name suffix of file lock files that are used to make sure a
/// database is open by only one process at any time.
pub const SUFFIX_LOCK_FILE: &str = ".lock.db";

/// The file name suffix of a H2 version 1.1 database file.
pub const SUFFIX_OLD_DATABASE_FILE: &str = ".data.db";

/**
 * The file name suffix of a MVStore file.
 */
pub const SUFFIX_MV_FILE: &str = ".mv.db";

/**
 * The file name suffix of a new MVStore file, used when compacting a store.
 */
pub const SUFFIX_MV_STORE_NEW_FILE: &str = ".newFile";

/**
 * The file name suffix of a temporary MVStore file, used when compacting a
 * store.
 */
pub const SUFFIX_MV_STORE_TEMP_FILE: &str = ".tempFile";

/**
 * The file name suffix of temporary files.
 */
pub const SUFFIX_TEMP_FILE: &str = ".temp.db";

/**
 * The file name suffix of trace files.
 */
pub const SUFFIX_TRACE_FILE: &str = ".trace.db";

/// How often we check to see if we need to apply a throttling delay if SET THROTTLE has been used.
pub const THROTTLE_DELAY: Integer = 50;

/// The database URL format in simplified Backus-Naur form.
pub const URL_FORMAT: &str = "jdbc:h2:{ {.|mem:}[name] | [file:]fileName | {tcp|ssl}:[//]server[:port][,server2[:port]]/name }[;key=value...]";

/// The package name of user defined classes.
pub const USER_PACKAGE: &str = "org.h2.dynamic";

/// The maximum time in milliseconds to keep the cost of a view.
///10000 means 10 seconds.
pub const VIEW_COST_CACHE_MAX_AGE: Integer = 10_000;

/// The name of the index cache that is used for temporary view (subqueries used as tables).
pub const VIEW_INDEX_CACHE_SIZE: Integer = 64;

/// The maximum number of entries in query statistics.
pub const QUERY_STATISTICS_MAX_ENTRIES: Integer = 100;

/// Announced version for PgServer.
pub const PG_VERSION: &str = "8.2.24";

/// The version of this product, consisting of major version, minor version, and build id.
pub const VERSION: &str = "2.1.214";

/// The complete version number of this database, consisting of
/// the major version, the minor version, the build id, and the build date.
pub static FULL_VERSION: &str = "2.1.214";