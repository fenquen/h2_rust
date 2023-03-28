use crate::h2_rust_common::Integer;

/// The error with code <code>2000</code> is thrown when
/// the result set is positioned before the first or after the last row, or
/// not on a valid row for the given operation.
/// Example of wrong usage:
/// <pre>
/// ResultSet rs = stat.executeQuery("SELECT /// FROM DUAL");
/// rs.getString(1);
/// </pre>
/// Correct:
/// <pre>
/// ResultSet rs = stat.executeQuery("SELECT /// FROM DUAL");
/// rs.next();
/// rs.getString(1);
/// </pre>
////
pub const NO_DATA_AVAILABLE: Integer = 2000;

// 07: dynamic SQL error


/// The error with code <code>7001</code> is thrown when
/// trying to call a function with the wrong number of parameters.
/// Example:
/// <pre>
/// CALL ABS(1, 2)
/// </pre>
////
pub const INVALID_PARAMETER_COUNT_2: Integer = 7001;

// 08: connection exception


/// The error with code <code>8000</code> is thrown when
/// there was a problem trying to create a database lock.
/// See the message and cause for details.
////
pub const ERROR_OPENING_DATABASE_1: Integer = 8000;

// 21: cardinality violation


/// The error with code <code>21002</code> is thrown when the number of
/// columns does not match. Possible reasons are: for an INSERT or MERGE
/// statement, the column count does not match the table or the column list
/// specified. For a SELECT UNION statement, both queries return a different
/// number of columns. For a constraint, the number of referenced and
/// referencing columns does not match. Example:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR);
/// INSERT INTO TEST VALUES('Hello');
/// </pre>
////
pub const COLUMN_COUNT_DOES_NOT_MATCH: Integer = 21002;

// 22: data exception


/// The error with code <code>22001</code> is thrown when
/// trying to insert a value that is too long for the column.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR(2));
/// INSERT INTO TEST VALUES(1, 'Hello');
/// </pre>
////
pub const VALUE_TOO_LONG_2: Integer = 22001;


/// The error with code <code>22003</code> is thrown when a value is out of
/// range when converting to another data type. Example:
/// <pre>
/// CALL CAST(1000000 AS TINYINT);
/// SELECT CAST(124.34 AS DECIMAL(2, 2));
/// </pre>
////
pub const NUMERIC_VALUE_OUT_OF_RANGE_1: Integer = 22003;


/// The error with code <code>22004</code> is thrown when a value is out of
/// range when converting to another column's data type.
////
pub const NUMERIC_VALUE_OUT_OF_RANGE_2: Integer = 22004;


/// The error with code <code>22007</code> is thrown when
/// a text can not be converted to a date, time, or timestamp constant.
/// Examples:
/// <pre>
/// CALL DATE '2007-January-01';
/// CALL TIME '14:61:00';
/// CALL TIMESTAMP '2001-02-30 12:00:00';
/// </pre>
////
pub const INVALID_DATETIME_CONSTANT_2: Integer = 22007;


/// The error with code <code>22012</code> is thrown when trying to divide
/// a value by zero. Example:
/// <pre>
/// CALL 1/0;
/// </pre>
////
pub const DIVISION_BY_ZERO_1: Integer = 22012;


/// The error with code <code>22013</code> is thrown when preceding or
/// following size in a window function is null or negative. Example:
/// <pre>
/// FIRST_VALUE(N) OVER(ORDER BY N ROWS -1 PRECEDING)
/// </pre>
////
pub const INVALID_PRECEDING_OR_FOLLOWING_1: Integer = 22013;


/// The error with code <code>22018</code> is thrown when
/// trying to convert a value to a data type where the conversion is
/// undefined, or when an error occurred trying to convert. Example:
/// <pre>
/// CALL CAST(DATE '2001-01-01' AS BOOLEAN);
/// CALL CAST('CHF 99.95' AS );
/// </pre>
////
pub const DATA_CONVERSION_ERROR_1: Integer = 22018;


/// The error with code <code>22025</code> is thrown when using an invalid
/// escape character sequence for LIKE or REGEXP. The default escape
/// character is '\'. The escape character is required when searching for
/// the characters '%', '_' and the escape character itself. That means if
/// you want to search for the text '10%', you need to use LIKE '10\%'. If
/// you want to search for 'C:\temp' you need to use 'C:\\temp'. The escape
/// character can be changed using the ESCAPE clause as in LIKE '10+%' ESCAPE
/// '+'. Example of wrong usage:
/// <pre>
/// CALL 'C:\temp' LIKE 'C:\temp';
/// CALL '1+1' LIKE '1+1' ESCAPE '+';
/// </pre>
/// Correct:
/// <pre>
/// CALL 'C:\temp' LIKE 'C:\\temp';
/// CALL '1+1' LIKE '1++1' ESCAPE '+';
/// </pre>
////
pub const LIKE_ESCAPE_ERROR_1: Integer = 22025;


/// The error with code <code>22030</code> is thrown when
/// an attempt is made to INSERT or UPDATE an ENUM-typed cell,
/// but the value is not one of the values enumerated by the
/// type.
///
/// Example:
/// <pre>
/// CREATE TABLE TEST(CASE ENUM('sensitive','insensitive'));
/// INSERT INTO TEST VALUES('snake');
/// </pre>
////
pub const ENUM_VALUE_NOT_PERMITTED: Integer = 22030;


/// The error with code <code>22032</code> is thrown when an
/// attempt is made to add or modify an ENUM-typed column so
/// that one or more of its enumerators would be empty.
///
/// Example:
/// <pre>
/// CREATE TABLE TEST(CASE ENUM(' '));
/// </pre>
////
pub const ENUM_EMPTY: Integer = 22032;


/// The error with code <code>22033</code> is thrown when an
/// attempt is made to add or modify an ENUM-typed column so
/// that it would have duplicate values.
///
/// Example:
/// <pre>
/// CREATE TABLE TEST(CASE ENUM('sensitive', 'sensitive'));
/// </pre>
////
pub const ENUM_DUPLICATE: Integer = 22033;


/// The error with code <code>22034</code> is thrown when an
/// attempt is made to read non-existing element of an array.
///
/// Example:
/// <pre>
/// VALUES ARRAY[1, 2][3]
/// </pre>
////
pub const ARRAY_ELEMENT_ERROR_2: Integer = 22034;

// 23: constraint violation


/// The error with code <code>23502</code> is thrown when
/// trying to insert NULL into a column that does not allow NULL.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR NOT NULL);
/// INSERT INTO TEST(ID) VALUES(1);
/// </pre>
////
pub const NULL_NOT_ALLOWED: Integer = 23502;


/// The error with code <code>23503</code> is thrown when trying to delete
/// or update a row when this would violate a referential constraint, because
/// there is a child row that would become an orphan. Example:
/// <pre>
/// CREATE TABLE TEST(ID  PRIMARY KEY, PARENT );
/// INSERT INTO TEST VALUES(1, 1), (2, 1);
/// ALTER TABLE TEST ADD CONSTRAINT TEST_ID_PARENT
///       FOREIGN KEY(PARENT) REFERENCES TEST(ID) ON DELETE RESTRICT;
/// DELETE FROM TEST WHERE ID :Integer= 1;
/// </pre>
////
pub const REFERENTIAL_INTEGRITY_VIOLATED_CHILD_EXISTS_1: Integer = 23503;


/// The error with code <code>23505</code> is thrown when trying to insert
/// a row that would violate a unique index or primary key. Example:
/// <pre>
/// CREATE TABLE TEST(ID  PRIMARY KEY);
/// INSERT INTO TEST VALUES(1);
/// INSERT INTO TEST VALUES(1);
/// </pre>
////
pub const DUPLICATE_KEY_1: Integer = 23505;


/// The error with code <code>23506</code> is thrown when trying to insert
/// or update a row that would violate a referential constraint, because the
/// referenced row does not exist. Example:
/// <pre>
/// CREATE TABLE PARENT(ID  PRIMARY KEY);
/// CREATE TABLE CHILD(P_ID  REFERENCES PARENT(ID));
/// INSERT INTO CHILD VALUES(1);
/// </pre>
////
pub const REFERENTIAL_INTEGRITY_VIOLATED_PARENT_MISSING_1: Integer = 23506;


/// The error with code <code>23507</code> is thrown when
/// updating or deleting from a table with a foreign key constraint
/// that should set the default value, but there is no default value defined.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID  PRIMARY KEY, PARENT );
/// INSERT INTO TEST VALUES(1, 1), (2, 1);
/// ALTER TABLE TEST ADD CONSTRAINT TEST_ID_PARENT
///   FOREIGN KEY(PARENT) REFERENCES TEST(ID) ON DELETE SET DEFAULT;
/// DELETE FROM TEST WHERE ID :Integer= 1;
/// </pre>
////
pub const NO_DEFAULT_SET_1: Integer = 23507;


/// The error with code <code>23513</code> is thrown when
/// a check constraint is violated. Example:
/// <pre>
/// CREATE TABLE TEST(ID  CHECK (ID&gt;0));
/// INSERT INTO TEST VALUES(0);
/// </pre>
////
pub const CHECK_CONSTRAINT_VIOLATED_1: Integer = 23513;


/// The error with code <code>23514</code> is thrown when
/// evaluation of a check constraint resulted in an error.
////
pub const CHECK_CONSTRAINT_INVALID: Integer = 23514;

// 28: invalid authorization specification


/// The error with code <code>28000</code> is thrown when
/// there is no such user registered in the database, when the user password
/// does not match, or when the database encryption password does not match
/// (if database encryption is used).
////
pub const WRONG_USER_OR_PASSWORD: Integer = 28000;

// 3B: savepoint exception


/// The error with code <code>40001</code> is thrown when
/// the database engine has detected a deadlock. The transaction of this
/// session has been rolled back to solve the problem. A deadlock occurs when
/// a session tries to lock a table another session has locked, while the
/// other session wants to lock a table the first session has locked. As an
/// example, session 1 has locked table A, while session 2 has locked table
/// B. If session 1 now tries to lock table B and session 2 tries to lock
/// table A, a deadlock has occurred. Deadlocks that involve more than two
/// sessions are also possible. To solve deadlock problems, an application
/// should lock tables always in the same order, such as always lock table A
/// before locking table B. For details, see <a
/// href:Integer="https://en.wikipedia.org/wiki/Deadlock">Wikipedia Deadlock</a>.
////
pub const DEADLOCK_1: Integer = 40001;

// 42: syntax error or access rule violation


/// The error with code <code>42000</code> is thrown when
/// trying to execute an invalid SQL statement.
/// Example:
/// <pre>
/// CREATE ALIAS REMAINDER FOR "IEEEremainder";
/// </pre>
////
pub const SYNTAX_ERROR_1: Integer = 42000;


/// The error with code <code>42001</code> is thrown when
/// trying to execute an invalid SQL statement.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// INSERT INTO TEST(1);
/// </pre>
////
pub const SYNTAX_ERROR_2: Integer = 42001;


/// The error with code <code>42101</code> is thrown when
/// trying to create a table or view if an object with this name already
/// exists. Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// CREATE TABLE TEST(ID  PRIMARY KEY);
/// </pre>
////
pub const TABLE_OR_VIEW_ALREADY_EXISTS_1: Integer = 42101;


/// The error with code <code>42102</code> is thrown when
/// trying to query, modify or drop a table or view that does not exists
/// in this schema and database. A common cause is that the wrong
/// database was opened.
/// Example:
/// <pre>
/// SELECT /// FROM ABC;
/// </pre>
////
pub const TABLE_OR_VIEW_NOT_FOUND_1: Integer = 42102;


/// The error with code <code>42103</code> is thrown when
/// trying to query, modify or drop a table or view that does not exists
/// in this schema and database but similar names were found. A common cause
/// is that the names are written in different case.
/// Example:
/// <pre>
/// SELECT /// FROM ABC;
/// </pre>
////
pub const TABLE_OR_VIEW_NOT_FOUND_WITH_CANDIDATES_2: Integer = 42103;


/// The error with code <code>42104</code> is thrown when
/// trying to query, modify or drop a table or view that does not exists
/// in this schema and database but it is empty anyway. A common cause is
/// that the wrong database was opened.
/// Example:
/// <pre>
/// SELECT /// FROM ABC;
/// </pre>
////
pub const TABLE_OR_VIEW_NOT_FOUND_DATABASE_EMPTY_1: Integer = 42104;


/// The error with code <code>42111</code> is thrown when
/// trying to create an index if an index with the same name already exists.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR);
/// CREATE INDEX IDX_ID ON TEST(ID);
/// CREATE TABLE ADDRESS(ID );
/// CREATE INDEX IDX_ID ON ADDRESS(ID);
/// </pre>
////
pub const INDEX_ALREADY_EXISTS_1: Integer = 42111;


/// The error with code <code>42112</code> is thrown when
/// trying to drop or reference an index that does not exist.
/// Example:
/// <pre>
/// DROP INDEX ABC;
/// </pre>
////
pub const INDEX_NOT_FOUND_1: Integer = 42112;


/// The error with code <code>42121</code> is thrown when trying to create
/// a table or insert into a table and use the same column name twice.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID , ID );
/// </pre>
////
pub const DUPLICATE_COLUMN_NAME_1: Integer = 42121;


/// The error with code <code>42122</code> is thrown when
/// referencing an non-existing column.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// SELECT NAME FROM TEST;
/// </pre>
////
pub const COLUMN_NOT_FOUND_1: Integer = 42122;


/// The error with code <code>42131</code> is thrown when
/// identical expressions should be used, but different
/// expressions were found.
/// Example:
/// <pre>
/// SELECT MODE(A ORDER BY B) FROM TEST;
/// </pre>
////
pub const IDENTICAL_EXPRESSIONS_SHOULD_BE_USED: Integer = 42131;


/// The error with code <code>42602</code> is thrown when
/// invalid name of identifier is used.
/// Example:
/// <pre>
/// statement.enquoteIdentifier("\"", true);
/// </pre>
////
pub const INVALID_NAME_1: Integer = 42602;


/// The error with code <code>42622</code> is thrown when
/// name of identifier is too long.
/// Example:
/// <pre>
/// char[] c :Integer= new char[1000];
/// Arrays.fill(c, 'A');
/// statement.executeQuery("SELECT 1 " + new String(c));
/// </pre>
////
pub const NAME_TOO_LONG_2: Integer = 42622;

// 54: program limit exceeded


/// The error with code <code>54011</code> is thrown when
/// too many columns were specified in a table, select statement,
/// or row value.
/// Example:
/// <pre>
/// CREATE TABLE TEST(C1 INTEGER, C2 INTEGER, ..., C20000 INTEGER);
/// </pre>
////
pub const TOO_MANY_COLUMNS_1: Integer = 54011;

// 0A: feature not supported

// HZ: remote database access

//


/// The error with code <code>50000</code> is thrown when
/// something unexpected occurs, for example an internal stack
/// overflow. For details about the problem, see the cause of the
/// exception in the stack trace.
////
pub const GENERAL_ERROR_1: Integer = 50000;


/// The error with code <code>50004</code> is thrown when
/// creating a table with an unsupported data type, or
/// when the data type is unknown because parameters are used.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID VERYSMALLINT);
/// </pre>
////
pub const UNKNOWN_DATA_TYPE_1: Integer = 50004;


/// The error with code <code>50100</code> is thrown when calling an
/// unsupported JDBC method or database feature. See the stack trace for
/// details.
////
pub const FEATURE_NOT_SUPPORTED_1: Integer = 50100;


/// The error with code <code>50200</code> is thrown when
/// another connection locked an object longer than the lock timeout
/// set for this connection, or when a deadlock occurred.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// -- connection 1:
/// SET AUTOCOMMIT FALSE;
/// INSERT INTO TEST VALUES(1);
/// -- connection 2:
/// SET AUTOCOMMIT FALSE;
/// INSERT INTO TEST VALUES(1);
/// </pre>
////
pub const LOCK_TIMEOUT_1: Integer = 50200;


/// The error with code <code>57014</code> is thrown when
/// a statement was canceled using Statement.cancel() or
/// when the query timeout has been reached.
/// Examples:
/// <pre>
/// stat.setQueryTimeout(1);
/// stat.cancel();
/// </pre>
////
pub const STATEMENT_WAS_CANCELED: Integer = 57014;


/// The error with code <code>90000</code> is thrown when
/// a function that does not return a result set was used in the FROM clause.
/// Example:
/// <pre>
/// SELECT /// FROM SIN(1);
/// </pre>
////
pub const FUNCTION_MUST_RETURN_RESULT_SET_1: Integer = 90000;


/// The error with code <code>90001</code> is thrown when
/// Statement.executeUpdate() was called for a SELECT statement.
/// This is not allowed according to the JDBC specs.
////
pub const METHOD_NOT_ALLOWED_FOR_QUERY: Integer = 90001;


/// The error with code <code>90002</code> is thrown when
/// Statement.executeQuery() was called for a statement that does
/// not return a result set (for example, an UPDATE statement).
/// This is not allowed according to the JDBC specs.
////
pub const METHOD_ONLY_ALLOWED_FOR_QUERY: Integer = 90002;


/// The error with code <code>90003</code> is thrown when
/// trying to convert a String to a binary value. Two hex digits
/// per byte are required. Example of wrong usage:
/// <pre>
/// CALL X'00023';
/// Hexadecimal string with odd number of characters: 00023
/// </pre>
/// Correct:
/// <pre>
/// CALL X'000023';
/// </pre>
////
pub const HEX_STRING_ODD_1: Integer = 90003;


/// The error with code <code>90004</code> is thrown when
/// trying to convert a text to binary, but the expression contains
/// a non-hexadecimal character.
/// Example:
/// <pre>
/// CALL X'ABCDEFGH';
/// CALL CAST('ABCDEFGH' AS BINARY);
/// </pre>
/// Conversion from text to binary is supported, but the text must
/// represent the hexadecimal encoded bytes.
////
pub const HEX_STRING_WRONG_1: Integer = 90004;


/// The error with code <code>90005</code> is thrown when
/// trying to create a trigger with invalid combination of flags.
////
pub const INVALID_TRIGGER_FLAGS_1: Integer = 90005;


/// The error with code <code>90006</code> is thrown when
/// trying to get a value from a sequence that has run out of numbers
/// and does not have cycling enabled.
////
pub const SEQUENCE_EXHAUSTED: Integer = 90006;


/// The error with code <code>90007</code> is thrown when
/// trying to call a JDBC method on an object that has been closed.
////
pub const OBJECT_CLOSED: Integer = 90007;


/// The error with code <code>90008</code> is thrown when
/// trying to use a value that is not valid for the given operation.
/// Example:
/// <pre>
/// CREATE SEQUENCE TEST INCREMENT 0;
/// </pre>
////
pub const INVALID_VALUE_2: Integer = 90008;


/// The error with code <code>90009</code> is thrown when
/// trying to create a sequence with an invalid combination
/// of attributes (min value, max value, start value, etc).
////
pub const SEQUENCE_ATTRIBUTES_INVALID_7: Integer = 90009;


/// The error with code <code>90010</code> is thrown when
/// trying to format a timestamp or number using TO_CHAR
/// with an invalid format.
////
pub const INVALID_TO_CHAR_FORMAT: Integer = 90010;


/// The error with code <code>90011</code> is thrown when
/// trying to open a connection to a database using an implicit relative
/// path, such as "jdbc:h2:test" (in which case the database file would be
/// stored in the current working directory of the application). This is not
/// allowed because it can lead to confusion where the database file is, and
/// can result in multiple databases because different working directories
/// are used. Instead, use "jdbc:h2:~/name" (relative to the current user
/// home directory), use an absolute path, set the base directory (baseDir),
/// use "jdbc:h2:./name" (explicit relative path), or set the system property
/// "h2.implicitRelativePath" to "true" (to prevent this check). For Windows,
/// an absolute path also needs to include the drive ("C:/..."). Please see
/// the documentation on the supported URL format. Example:
/// <pre>
/// jdbc:h2:test
/// </pre>
////
pub const URL_RELATIVE_TO_CWD: Integer = 90011;


/// The error with code <code>90012</code> is thrown when
/// trying to execute a statement with an parameter.
/// Example:
/// <pre>
/// CALL SIN(?);
/// </pre>
////
pub const PARAMETER_NOT_SET_1: Integer = 90012;


/// The error with code <code>90013</code> is thrown when when trying to access
/// a database object with a catalog name that does not match the database
/// name.
/// <pre>
/// SELECT /// FROM database_that_does_not_exist.table_name
/// </pre>
////
pub const DATABASE_NOT_FOUND_1: Integer = 90013;


/// The error with code <code>90014</code> is thrown when
/// trying to parse a date with an unsupported format string, or
/// when the date can not be parsed.
/// Example:
/// <pre>
/// CALL PARSEDATETIME('2001 January', 'yyyy mm');
/// </pre>
////
pub const PARSE_ERROR_1: Integer = 90014;


/// The error with code <code>90015</code> is thrown when
/// using an aggregate function with a data type that is not supported.
/// Example:
/// <pre>
/// SELECT SUM('Hello') FROM DUAL;
/// </pre>
////
pub const SUM_OR_AVG_ON_WRONG_DATATYPE_1: Integer = 90015;


/// The error with code <code>90016</code> is thrown when
/// a column was used in the expression list or the order by clause of a
/// group or aggregate query, and that column is not in the GROUP BY clause.
/// Example of wrong usage:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR);
/// INSERT INTO TEST VALUES(1, 'Hello'), (2, 'World');
/// SELECT ID, MAX(NAME) FROM TEST;
/// Column ID must be in the GROUP BY list.
/// </pre>
/// Correct:
/// <pre>
/// SELECT ID, MAX(NAME) FROM TEST GROUP BY ID;
/// </pre>
////
pub const MUST_GROUP_BY_COLUMN_1: Integer = 90016;


/// The error with code <code>90017</code> is thrown when
/// trying to define a second primary key constraint for this table.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID  PRIMARY KEY, NAME VARCHAR);
/// ALTER TABLE TEST ADD CONSTRAINT PK PRIMARY KEY(NAME);
/// </pre>
////
pub const SECOND_PRIMARY_KEY: Integer = 90017;


/// The error with code <code>90018</code> is thrown when
/// the connection was opened, but never closed. In the finalizer of the
/// connection, this forgotten close was detected and the connection was
/// closed automatically, but relying on the finalizer is not good practice
/// as it is not guaranteed and behavior is virtual machine dependent. The
/// application should close the connection. This exception only appears in
/// the .trace.db file. Example of wrong usage:
/// <pre>
/// Connection conn;
/// conn :Integer= DriverManager.getConnection(&quot;jdbc:h2:&tilde;/test&quot;);
/// conn :Integer= null;
/// The connection was not closed by the application and is
/// garbage collected
/// </pre>
/// Correct:
/// <pre>
/// conn.close();
/// </pre>
////
pub const TRACE_CONNECTION_NOT_CLOSED: Integer = 90018;


/// The error with code <code>90019</code> is thrown when
/// trying to drop the current user, if there are no other admin users.
/// Example:
/// <pre>
/// DROP USER SA;
/// </pre>
////
pub const CANNOT_DROP_CURRENT_USER: Integer = 90019;


/// The error with code <code>90020</code> is thrown when trying to open a
/// database in embedded mode if this database is already in use in another
/// process (or in a different class loader). Multiple connections to the
/// same database are supported in the following cases:
/// <ul><li>In embedded mode (URL of the form jdbc:h2:~/test) if all
/// connections are opened within the same process and class loader.
/// </li><li>In server and cluster mode (URL of the form
/// jdbc:h2:tcp://localhost/test) using remote connections.
/// </li></ul>
/// The mixed mode is also supported. This mode requires to start a server
/// in the same process where the database is open in embedded mode.
////
pub const DATABASE_ALREADY_OPEN_1: Integer = 90020;


/// The error with code <code>90021</code> is thrown when
/// trying to change a specific database property that conflicts with other
/// database properties.
////
pub const UNSUPPORTED_SETTING_COMBINATION: Integer = 90021;


/// The error with code <code>90022</code> is thrown when
/// trying to call a unknown function.
/// Example:
/// <pre>
/// CALL SPECIAL_SIN(10);
/// </pre>
////
pub const FUNCTION_NOT_FOUND_1: Integer = 90022;


/// The error with code <code>90023</code> is thrown when trying to set a
/// primary key on a nullable column or when trying to drop NOT NULL
/// constraint on primary key or identity column.
/// Examples:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR);
/// ALTER TABLE TEST ADD CONSTRAINT PK PRIMARY KEY(ID);
/// </pre>
/// <pre>
/// CREATE TABLE TEST(ID  PRIMARY KEY, NAME VARCHAR);
/// ALTER TABLE TEST ALTER COLUMN ID DROP NOT NULL;
/// </pre>
/// <pre>
/// CREATE TABLE TEST(ID  GENERATED ALWAYS AS IDENTITY, NAME VARCHAR);
/// ALTER TABLE TEST ALTER COLUMN ID DROP NOT NULL;
/// </pre>
////
pub const COLUMN_MUST_NOT_BE_NULLABLE_1: Integer = 90023;


/// The error with code <code>90024</code> is thrown when
/// a file could not be renamed.
////
pub const FILE_RENAME_FAILED_2: Integer = 90024;


/// The error with code <code>90025</code> is thrown when
/// a file could not be deleted, because it is still in use
/// (only in Windows), or because an error occurred when deleting.
////
pub const FILE_DELETE_FAILED_1: Integer = 90025;


/// The error with code <code>90026</code> is thrown when
/// an object could not be serialized.
////
pub const SERIALIZATION_FAILED_1: Integer = 90026;


/// The error with code <code>90027</code> is thrown when
/// an object could not be de-serialized.
////
pub const DESERIALIZATION_FAILED_1: Integer = 90027;


/// The error with code <code>90028</code> is thrown when
/// an input / output error occurred. For more information, see the root
/// cause of the exception.
////
pub const IO_EXCEPTION_1: Integer = 90028;


/// The error with code <code>90029</code> is thrown when
/// calling ResultSet.deleteRow(), insertRow(), or updateRow()
/// when the current row is not updatable.
/// Example:
/// <pre>
/// ResultSet rs :Integer= stat.executeQuery("SELECT /// FROM TEST");
/// rs.next();
/// rs.insertRow();
/// </pre>
////
pub const NOT_ON_UPDATABLE_ROW: Integer = 90029;


/// The error with code <code>90030</code> is thrown when
/// the database engine has detected a checksum mismatch in the data
/// or index. To solve this problem, restore a backup or use the
/// Recovery tool (org.h2.tools.Recover).
////
pub const FILE_CORRUPTED_1: Integer = 90030;


/// The error with code <code>90031</code> is thrown when
/// an input / output error occurred. For more information, see the root
/// cause of the exception.
////
pub const IO_EXCEPTION_2: Integer = 90031;


/// The error with code <code>90032</code> is thrown when
/// trying to drop or alter a user that does not exist.
/// Example:
/// <pre>
/// DROP USER TEST_USER;
/// </pre>
////
pub const USER_NOT_FOUND_1: Integer = 90032;


/// The error with code <code>90033</code> is thrown when
/// trying to create a user or role if a user with this name already exists.
/// Example:
/// <pre>
/// CREATE USER TEST_USER;
/// CREATE USER TEST_USER;
/// </pre>
////
pub const USER_ALREADY_EXISTS_1: Integer = 90033;


/// The error with code <code>90034</code> is thrown when
/// writing to the trace file failed, for example because the there
/// is an I/O exception. This message is printed to System.out,
/// but only once.
////
pub const TRACE_FILE_ERROR_2: Integer = 90034;


/// The error with code <code>90035</code> is thrown when
/// trying to create a sequence if a sequence with this name already
/// exists.
/// Example:
/// <pre>
/// CREATE SEQUENCE TEST_SEQ;
/// CREATE SEQUENCE TEST_SEQ;
/// </pre>
////
pub const SEQUENCE_ALREADY_EXISTS_1: Integer = 90035;


/// The error with code <code>90036</code> is thrown when
/// trying to access a sequence that does not exist.
/// Example:
/// <pre>
/// SELECT NEXT VALUE FOR SEQUENCE XYZ;
/// </pre>
////
pub const SEQUENCE_NOT_FOUND_1: Integer = 90036;


/// The error with code <code>90037</code> is thrown when
/// trying to drop or alter a view that does not exist.
/// Example:
/// <pre>
/// DROP VIEW XYZ;
/// </pre>
////
pub const VIEW_NOT_FOUND_1: Integer = 90037;


/// The error with code <code>90038</code> is thrown when
/// trying to create a view if a view with this name already
/// exists.
/// Example:
/// <pre>
/// CREATE VIEW DUMMY AS SELECT /// FROM DUAL;
/// CREATE VIEW DUMMY AS SELECT /// FROM DUAL;
/// </pre>
////
pub const VIEW_ALREADY_EXISTS_1: Integer = 90038;

/// The error with code <code>90039</code> is thrown when
/// trying to access a CLOB or BLOB object that timed out.
/// See the database setting LOB_TIMEOUT.
////
pub const LOB_CLOSED_ON_TIMEOUT_1: Integer = 90039;


/// The error with code <code>90040</code> is thrown when
/// a user that is not administrator tries to execute a statement
/// that requires admin privileges.
////
pub const ADMIN_RIGHTS_REQUIRED: Integer = 90040;


/// The error with code <code>90041</code> is thrown when
/// trying to create a trigger and there is already a trigger with that name.
/// <pre>
/// CREATE TABLE TEST(ID );
/// CREATE TRIGGER TRIGGER_A AFTER INSERT ON TEST
///      CALL "org.h2.samples.TriggerSample$MyTrigger";
/// CREATE TRIGGER TRIGGER_A AFTER INSERT ON TEST
///      CALL "org.h2.samples.TriggerSample$MyTrigger";
/// </pre>
////
pub const TRIGGER_ALREADY_EXISTS_1: Integer = 90041;


/// The error with code <code>90042</code> is thrown when
/// trying to drop a trigger that does not exist.
/// Example:
/// <pre>
/// DROP TRIGGER TRIGGER_XYZ;
/// </pre>
////
pub const TRIGGER_NOT_FOUND_1: Integer = 90042;


/// The error with code <code>90043</code> is thrown when
/// there is an error initializing the trigger, for example because the
/// class does not implement the Trigger interface.
/// See the root cause for details.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// CREATE TRIGGER TRIGGER_A AFTER INSERT ON TEST
///      CALL "java.lang.String";
/// </pre>
////
pub const ERROR_CREATING_TRIGGER_OBJECT_3: Integer = 90043;


/// The error with code <code>90044</code> is thrown when
/// an exception or error occurred while calling the triggers fire method.
/// See the root cause for details.
////
pub const ERROR_EXECUTING_TRIGGER_3: Integer = 90044;


/// The error with code <code>90045</code> is thrown when trying to create a
/// constraint if an object with this name already exists. Example:
/// <pre>
/// CREATE TABLE TEST(ID  NOT NULL);
/// ALTER TABLE TEST ADD CONSTRAINT PK PRIMARY KEY(ID);
/// ALTER TABLE TEST ADD CONSTRAINT PK PRIMARY KEY(ID);
/// </pre>
////
pub const CONSTRAINT_ALREADY_EXISTS_1: Integer = 90045;


/// The error with code <code>90046</code> is thrown when
/// trying to open a connection to a database using an unsupported URL
/// format. Please see the documentation on the supported URL format and
/// examples. Example:
/// <pre>
/// jdbc:h2:;;
/// </pre>
////
pub const URL_FORMAT_ERROR_2: Integer = 90046;


/// The error with code <code>90047</code> is thrown when
/// trying to connect to a TCP server with an incompatible client.
////
pub const DRIVER_VERSION_ERROR_2: Integer = 90047;


/// The error with code <code>90048</code> is thrown when
/// the file header of a database files (///.db) does not match the
/// expected version, or if it is corrupted.
////
pub const FILE_VERSION_ERROR_1: Integer = 90048;


/// The error with code <code>90049</code> is thrown when
/// trying to open an encrypted database with the wrong file encryption
/// password or algorithm.
////
pub const FILE_ENCRYPTION_ERROR_1: Integer = 90049;


/// The error with code <code>90050</code> is thrown when trying to open an
/// encrypted database, but not separating the file password from the user
/// password. The file password is specified in the password field, before
/// the user password. A single space needs to be added between the file
/// password and the user password; the file password itself may not contain
/// spaces. File passwords (as well as user passwords) are case sensitive.
/// Example of wrong usage:
/// <pre>
/// String url :Integer= &quot;jdbc:h2:&tilde;/test;CIPHER:Integer=AES&quot;;
/// String passwords :Integer= &quot;filePasswordUserPassword&quot;;
/// DriverManager.getConnection(url, &quot;sa&quot;, pwds);
/// </pre>
/// Correct:
/// <pre>
/// String url :Integer= &quot;jdbc:h2:&tilde;/test;CIPHER:Integer=AES&quot;;
/// String passwords :Integer= &quot;filePassword userPassword&quot;;
/// DriverManager.getConnection(url, &quot;sa&quot;, pwds);
/// </pre>
////
pub const WRONG_PASSWORD_FORMAT: Integer = 90050;

// 90051 was removed


/// The error with code <code>90052</code> is thrown when a single-column
/// subquery is expected but a subquery with other number of columns was
/// specified.
/// Example:
/// <pre>
/// VALUES ARRAY(SELECT A, B FROM TEST)
/// </pre>
////
pub const SUBQUERY_IS_NOT_SINGLE_COLUMN: Integer = 90052;


/// The error with code <code>90053</code> is thrown when
/// a subquery that is used as a value contains more than one row.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR);
/// INSERT INTO TEST VALUES(1, 'Hello'), (1, 'World');
/// SELECT X, (SELECT NAME FROM TEST WHERE ID:Integer=X) FROM DUAL;
/// </pre>
////
pub const SCALAR_SUBQUERY_CONTAINS_MORE_THAN_ONE_ROW: Integer = 90053;


/// The error with code <code>90054</code> is thrown when
/// an aggregate function is used where it is not allowed.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// INSERT INTO TEST VALUES(1), (2);
/// SELECT MAX(ID) FROM TEST WHERE ID :Integer= MAX(ID) GROUP BY ID;
/// </pre>
////
pub const INVALID_USE_OF_AGGREGATE_FUNCTION_1: Integer = 90054;


/// The error with code <code>90055</code> is thrown when
/// trying to open a database with an unsupported cipher algorithm.
/// Supported is AES.
/// Example:
/// <pre>
/// jdbc:h2:~/test;CIPHER:Integer=DES
/// </pre>
////
pub const UNSUPPORTED_CIPHER: Integer = 90055;


/// The error with code <code>90056</code> is thrown when trying to format a
/// timestamp using TO_DATE and TO_TIMESTAMP  with an invalid format.
////
pub const INVALID_TO_DATE_FORMAT: Integer = 90056;


/// The error with code <code>90057</code> is thrown when
/// trying to drop a constraint that does not exist.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// ALTER TABLE TEST DROP CONSTRAINT CID;
/// </pre>
////
pub const CONSTRAINT_NOT_FOUND_1: Integer = 90057;


/// The error with code <code>90058</code> is thrown when trying to call
/// commit or rollback inside a trigger, or when trying to call a method
/// inside a trigger that implicitly commits the current transaction, if an
/// object is locked. This is not because it would release the lock too
/// early.
////
pub const COMMIT_ROLLBACK_NOT_ALLOWED: Integer = 90058;


/// The error with code <code>90059</code> is thrown when
/// a query contains a column that could belong to multiple tables.
/// Example:
/// <pre>
/// CREATE TABLE PARENT(ID , NAME VARCHAR);
/// CREATE TABLE CHILD(PID , NAME VARCHAR);
/// SELECT ID, NAME FROM PARENT P, CHILD C WHERE P.ID :Integer= C.PID;
/// </pre>
////
pub const AMBIGUOUS_COLUMN_NAME_1: Integer = 90059;


/// The error with code <code>90060</code> is thrown when
/// trying to use a file locking mechanism that is not supported.
/// Currently only FILE (the default) and SOCKET are supported
/// Example:
/// <pre>
/// jdbc:h2:~/test;FILE_LOCK:Integer=LDAP
/// </pre>
////
pub const UNSUPPORTED_LOCK_METHOD_1: Integer = 90060;


/// The error with code <code>90061</code> is thrown when
/// trying to start a server if a server is already running at the same port.
/// It could also be a firewall problem. To find out if another server is
/// already running, run the following command on Windows:
/// <pre>
/// netstat -ano
/// </pre>
/// The column PID is the process id as listed in the Task Manager.
/// For Linux, use:
/// <pre>
/// netstat -npl
/// </pre>
////
pub const EXCEPTION_OPENING_PORT_2: Integer = 90061;


/// The error with code <code>90062</code> is thrown when
/// a directory or file could not be created. This can occur when
/// trying to create a directory if a file with the same name already
/// exists, or vice versa.
///
////
pub const FILE_CREATION_FAILED_1: Integer = 90062;


/// The error with code <code>90063</code> is thrown when
/// trying to rollback to a savepoint that is not defined.
/// Example:
/// <pre>
/// ROLLBACK TO SAVEPOINT S_UNKNOWN;
/// </pre>
////
pub const SAVEPOINT_IS_INVALID_1: Integer = 90063;


/// The error with code <code>90064</code> is thrown when
/// Savepoint.getSavepointName() is called on an unnamed savepoint.
/// Example:
/// <pre>
/// Savepoint sp :Integer= conn.setSavepoint();
/// sp.getSavepointName();
/// </pre>
////
pub const SAVEPOINT_IS_UNNAMED: Integer = 90064;


/// The error with code <code>90065</code> is thrown when
/// Savepoint.getSavepointId() is called on a named savepoint.
/// Example:
/// <pre>
/// Savepoint sp :Integer= conn.setSavepoint("Joe");
/// sp.getSavepointId();
/// </pre>
////
pub const SAVEPOINT_IS_NAMED: Integer = 90065;


/// The error with code <code>90066</code> is thrown when
/// the same property appears twice in the database URL or in
/// the connection properties.
/// Example:
/// <pre>
/// jdbc:h2:~/test;LOCK_TIMEOUT:Integer=0;LOCK_TIMEOUT:Integer=1
/// </pre>
////
pub const DUPLICATE_PROPERTY_1: Integer = 90066;


/// The error with code <code>90067</code> is thrown when the client could
/// not connect to the database, or if the connection was lost. Possible
/// reasons are: the database server is not running at the given port, the
/// connection was closed due to a shutdown, or the server was stopped. Other
/// possible causes are: the server is not an H2 server, or the network
/// connection is broken.
////
pub const CONNECTION_BROKEN_1: Integer = 90067;


/// The error with code <code>90068</code> is thrown when the given
/// expression that is used in the ORDER BY is not in the result list. This
/// is required for distinct queries, otherwise the result would be
/// ambiguous.
/// Example of wrong usage:
/// <pre>
/// CREATE TABLE TEST(ID , NAME VARCHAR);
/// INSERT INTO TEST VALUES(2, 'Hello'), (1, 'Hello');
/// SELECT DISTINCT NAME FROM TEST ORDER BY ID;
/// Order by expression ID must be in the result list in this case
/// </pre>
/// Correct:
/// <pre>
/// SELECT DISTINCT ID, NAME FROM TEST ORDER BY ID;
/// </pre>
////
pub const ORDER_BY_NOT_IN_RESULT: Integer = 90068;


/// The error with code <code>90069</code> is thrown when
/// trying to create a role if an object with this name already exists.
/// Example:
/// <pre>
/// CREATE ROLE TEST_ROLE;
/// CREATE ROLE TEST_ROLE;
/// </pre>
////
pub const ROLE_ALREADY_EXISTS_1: Integer = 90069;


/// The error with code <code>90070</code> is thrown when
/// trying to drop or grant a role that does not exists.
/// Example:
/// <pre>
/// DROP ROLE TEST_ROLE_2;
/// </pre>
////
pub const ROLE_NOT_FOUND_1: Integer = 90070;


/// The error with code <code>90071</code> is thrown when
/// trying to grant or revoke if no role or user with that name exists.
/// Example:
/// <pre>
/// GRANT SELECT ON TEST TO UNKNOWN;
/// </pre>
////
pub const USER_OR_ROLE_NOT_FOUND_1: Integer = 90071;


/// The error with code <code>90072</code> is thrown when
/// trying to grant or revoke both roles and rights at the same time.
/// Example:
/// <pre>
/// GRANT SELECT, TEST_ROLE ON TEST TO SA;
/// </pre>
////
pub const ROLES_AND_RIGHT_CANNOT_BE_MIXED: Integer = 90072;


/// The error with code <code>90073</code> is thrown when trying to create
/// an alias for a Java method, if two methods exists in this class that have
/// this name and the same number of parameters.
/// Example of wrong usage:
/// <pre>
/// CREATE ALIAS GET_LONG FOR
///      "java.lang.Long.getLong";
/// </pre>
/// Correct:
/// <pre>
/// CREATE ALIAS GET_LONG FOR
///      "java.lang.Long.getLong(java.lang.String, java.lang.Long)";
/// </pre>
////
pub const METHODS_MUST_HAVE_DIFFERENT_PARAMETER_COUNTS_2: Integer = 90073;


/// The error with code <code>90074</code> is thrown when
/// trying to grant a role that has already been granted.
/// Example:
/// <pre>
/// CREATE ROLE TEST_A;
/// CREATE ROLE TEST_B;
/// GRANT TEST_A TO TEST_B;
/// GRANT TEST_B TO TEST_A;
/// </pre>
////
pub const ROLE_ALREADY_GRANTED_1: Integer = 90074;


/// The error with code <code>90075</code> is thrown when
/// trying to alter a table and allow null for a column that is part of a
/// primary key or hash index.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID  PRIMARY KEY);
/// ALTER TABLE TEST ALTER COLUMN ID NULL;
/// </pre>
////
pub const COLUMN_IS_PART_OF_INDEX_1: Integer = 90075;


/// The error with code <code>90076</code> is thrown when
/// trying to create a function alias for a system function or for a function
/// that is already defined.
/// Example:
/// <pre>
/// CREATE ALIAS SQRT FOR "java.lang.Math.sqrt"
/// </pre>
////
pub const FUNCTION_ALIAS_ALREADY_EXISTS_1: Integer = 90076;


/// The error with code <code>90077</code> is thrown when
/// trying to drop a system function or a function alias that does not exist.
/// Example:
/// <pre>
/// DROP ALIAS SQRT;
/// </pre>
////
pub const FUNCTION_ALIAS_NOT_FOUND_1: Integer = 90077;


/// The error with code <code>90078</code> is thrown when
/// trying to create a schema if an object with this name already exists.
/// Example:
/// <pre>
/// CREATE SCHEMA TEST_SCHEMA;
/// CREATE SCHEMA TEST_SCHEMA;
/// </pre>
////
pub const SCHEMA_ALREADY_EXISTS_1: Integer = 90078;


/// The error with code <code>90079</code> is thrown when
/// trying to drop a schema that does not exist.
/// Example:
/// <pre>
/// DROP SCHEMA UNKNOWN;
/// </pre>
////
pub const SCHEMA_NOT_FOUND_1: Integer = 90079;


/// The error with code <code>90080</code> is thrown when
/// trying to rename a object to a different schema, or when trying to
/// create a related object in another schema.
/// For CREATE LINKED TABLE, it is thrown when multiple tables with that
/// name exist in different schemas.
/// Example:
/// <pre>
/// CREATE SCHEMA TEST_SCHEMA;
/// CREATE TABLE TEST(ID );
/// CREATE INDEX TEST_ID ON TEST(ID);
/// ALTER INDEX TEST_ID RENAME TO TEST_SCHEMA.IDX_TEST_ID;
/// </pre>
////
pub const SCHEMA_NAME_MUST_MATCH: Integer = 90080;


/// The error with code <code>90081</code> is thrown when
/// trying to alter a column to not allow NULL, if there
/// is already data in the table where this column is NULL.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// INSERT INTO TEST VALUES(NULL);
/// ALTER TABLE TEST ALTER COLUMN ID VARCHAR NOT NULL;
/// </pre>
////
pub const COLUMN_CONTAINS_NULL_VALUES_1: Integer = 90081;


/// The error with code <code>90082</code> is thrown when
/// trying to drop a system generated sequence.
////
pub const SEQUENCE_BELONGS_TO_A_TABLE_1: Integer = 90082;


/// The error with code <code>90083</code> is thrown when
/// trying to drop a column that is part of a constraint.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID , PID  REFERENCES(ID));
/// ALTER TABLE TEST DROP COLUMN PID;
/// </pre>
////
pub const COLUMN_IS_REFERENCED_1: Integer = 90083;


/// The error with code <code>90084</code> is thrown when
/// trying to drop the last column of a table.
/// Example:
/// <pre>
/// CREATE TABLE TEST(ID );
/// ALTER TABLE TEST DROP COLUMN ID;
/// </pre>
////
pub const CANNOT_DROP_LAST_COLUMN: Integer = 90084;


/// The error with code <code>90085</code> is thrown when
/// trying to manually drop an index that was generated by the system
/// because of a unique or referential constraint. To find
/// the owner of the index without attempt to drop it run
/// <pre>
/// SELECT CONSTRAINT_SCHEMA, CONSTRAINT_NAME
/// FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE
/// WHERE INDEX_SCHEMA :Integer= '&lt;index schema&gt;'
/// AND INDEX_NAME :Integer= '&lt;index name&gt;'
/// FETCH FIRST ROW ONLY
/// </pre>
/// Example of wrong usage:
/// <pre>
/// CREATE TABLE TEST(ID , CONSTRAINT UID UNIQUE(ID));
/// DROP INDEX UID_INDEX_0;
/// Index UID_INDEX_0 belongs to constraint UID
/// </pre>
/// Correct:
/// <pre>
/// ALTER TABLE TEST DROP CONSTRAINT UID;
/// </pre>
////
pub const INDEX_BELONGS_TO_CONSTRAINT_2: Integer = 90085;


/// The error with code <code>90086</code> is thrown when
/// a class can not be loaded because it is not in the classpath
/// or because a related class is not in the classpath.
/// Example:
/// <pre>
/// CREATE ALIAS TEST FOR "java.lang.invalid.Math.sqrt";
/// </pre>
////
pub const CLASS_NOT_FOUND_1: Integer = 90086;

/// The error with code <code>90087</code> is thrown when
/// a method with matching number of arguments was not found in the class.
/// Example:
/// <pre>
/// CREATE ALIAS TO_BINARY FOR "java.lang.Long.toBinaryString(long)";
/// CALL TO_BINARY(10, 2);
/// </pre>
////
pub const METHOD_NOT_FOUND_1: Integer = 90087;

/// The error with code <code>90088</code> is thrown when
/// trying to switch to an unknown mode.
/// Example:
/// <pre>
/// SET MODE UNKNOWN;
/// </pre>
////
pub const UNKNOWN_MODE_1: Integer = 90088;


/// The error with code <code>90089</code> is thrown when
/// trying to change the collation while there was already data in
/// the database. The collation of the database must be set when the
/// database is empty.
/// Example of wrong usage:
/// <pre>
/// CREATE TABLE TEST(NAME VARCHAR PRIMARY KEY);
/// INSERT INTO TEST VALUES('Hello', 'World');
/// SET COLLATION DE;
/// Collation cannot be changed because there is a data table: pub.TEST
/// </pre>
/// Correct:
/// <pre>
/// SET COLLATION DE;
/// CREATE TABLE TEST(NAME VARCHAR PRIMARY KEY);
/// INSERT INTO TEST VALUES('Hello', 'World');
/// </pre>
////
pub const COLLATION_CHANGE_WITH_DATA_TABLE_1: Integer = 90089;

/// The error with code <code>90090</code> is thrown when
/// trying to drop a schema that may not be dropped (the schema pub
/// and the schema INFORMATION_SCHEMA).
/// Example:
/// <pre>
/// DROP SCHEMA pub;
/// </pre>
////
pub const SCHEMA_CAN_NOT_BE_DROPPED_1: Integer = 90090;


/// The error with code <code>90091</code> is thrown when
/// trying to drop the role pub.
/// Example:
/// <pre>
/// DROP ROLE pub;
/// </pre>
////
pub const ROLE_CAN_NOT_BE_DROPPED_1: Integer = 90091;


/// The error with code <code>90093</code> is thrown when
/// trying to connect to a clustered database that runs in standalone
/// mode. This can happen if clustering is not enabled on the database,
/// or if one of the clients disabled clustering because it can not see
/// the other cluster node.
////
pub const CLUSTER_ERROR_DATABASE_RUNS_ALONE: Integer = 90093;


/// The error with code <code>90094</code> is thrown when
/// trying to connect to a clustered database that runs together with a
/// different cluster node setting than what is used when trying to connect.
////
pub const CLUSTER_ERROR_DATABASE_RUNS_CLUSTERED_1: Integer = 90094;


/// The error with code <code>90095</code> is thrown when
/// calling the method STRINGDECODE with an invalid escape sequence.
/// Only Java style escape sequences and Java properties file escape
/// sequences are supported.
/// Example:
/// <pre>
/// CALL STRINGDECODE('\i');
/// </pre>
////
pub const STRING_FORMAT_ERROR_1: Integer = 90095;


/// The error with code <code>90096</code> is thrown when
/// trying to perform an operation with a non-admin user if the
/// user does not have enough rights.
////
pub const NOT_ENOUGH_RIGHTS_FOR_1: Integer = 90096;


/// The error with code <code>90097</code> is thrown when
/// trying to delete or update a database if it is open in read-only mode.
/// Example:
/// <pre>
/// jdbc:h2:~/test;ACCESS_MODE_DATA:Integer=R
/// CREATE TABLE TEST(ID );
/// </pre>
////
pub const DATABASE_IS_READ_ONLY: Integer = 90097;


/// The error with code <code>90098</code> is thrown when the database has
/// been closed, for example because the system ran out of memory or because
/// the self-destruction counter has reached zero. This counter is only used
/// for recovery testing, and not set in normal operation.
////
pub const DATABASE_IS_CLOSED: Integer = 90098;


/// The error with code <code>90099</code> is thrown when an error occurred
/// trying to initialize the database event listener. Example:
/// <pre>
/// jdbc:h2:&tilde;/test;DATABASE_EVENT_LISTENER:Integer='java.lang.String'
/// </pre>
////
pub const ERROR_SETTING_DATABASE_EVENT_LISTENER_2: Integer = 90099;


/// The error with code <code>90101</code> is thrown when
/// the XA API detected unsupported transaction names. This can happen
/// when mixing application generated transaction names and transaction names
/// generated by this databases XAConnection API.
////
pub const WRONG_XID_FORMAT_1: Integer = 90101;


/// The error with code <code>90102</code> is thrown when
/// trying to use unsupported options for the given compression algorithm.
/// Example of wrong usage:
/// <pre>
/// CALL COMPRESS(STRINGTOUTF8(SPACE(100)), 'DEFLATE l 10');
/// </pre>
/// Correct:
/// <pre>
/// CALL COMPRESS(STRINGTOUTF8(SPACE(100)), 'DEFLATE l 9');
/// </pre>
////
pub const UNSUPPORTED_COMPRESSION_OPTIONS_1: Integer = 90102;


/// The error with code <code>90103</code> is thrown when
/// trying to use an unsupported compression algorithm.
/// Example:
/// <pre>
/// CALL COMPRESS(STRINGTOUTF8(SPACE(100)), 'BZIP');
/// </pre>
////
pub const UNSUPPORTED_COMPRESSION_ALGORITHM_1: Integer = 90103;


/// The error with code <code>90104</code> is thrown when
/// the data can not be de-compressed.
/// Example:
/// <pre>
/// CALL EXPAND(X'00FF');
/// </pre>
////
pub const COMPRESSION_ERROR: Integer = 90104;

/// The error with code <code>90105</code> is thrown when
/// an exception occurred in a user-defined method.
/// Example:
/// <pre>
/// CREATE ALIAS SYS_PROP FOR "java.lang.System.getProperty";
/// CALL SYS_PROP(NULL);
/// </pre>
pub const EXCEPTION_IN_FUNCTION_1: Integer = 90105;

/// The error with code <code>90106</code> is thrown when
/// trying to truncate a table that can not be truncated.
/// Tables with referential integrity constraints can not be truncated.
/// Also, system tables and view can not be truncated.
/// Example:
/// <pre>
/// TRUNCATE TABLE INFORMATION_SCHEMA.SETTINGS;
/// </pre>
////
pub const CANNOT_TRUNCATE_1: Integer = 90106;


/// The error with code <code>90107</code> is thrown when
/// trying to drop an object because another object would become invalid.
/// Example:
/// <pre>
/// CREATE TABLE COUNT(X );
/// CREATE TABLE ITEMS(ID  DEFAULT SELECT MAX(X)+1 FROM COUNT);
/// DROP TABLE COUNT;
/// </pre>
////
pub const CANNOT_DROP_2: Integer = 90107;


/// The error with code <code>90108</code> is thrown when not enough heap
/// memory was available. A possible solutions is to increase the memory size
/// using <code>java -Xmx128m ...</code>. Another solution is to reduce
/// the cache size.
////
pub const OUT_OF_MEMORY: Integer = 90108;


/// The error with code <code>90109</code> is thrown when
/// trying to run a query against an invalid view.
/// Example:
/// <pre>
/// CREATE FORCE VIEW TEST_VIEW AS SELECT /// FROM TEST;
/// SELECT /// FROM TEST_VIEW;
/// </pre>
////
pub const VIEW_IS_INVALID_2: Integer = 90109;


/// The error with code <code>90110</code> is thrown when
/// trying to compare or combine values of incomparable data types.
/// Example:
/// <pre>
/// CREATE TABLE test (id  NOT NULL, name VARCHAR);
/// select /// from test where id :Integer= (1, 2);
/// </pre>
////
pub const TYPES_ARE_NOT_COMPARABLE_2: Integer = 90110;


/// The error with code <code>90111</code> is thrown when
/// an exception occurred while accessing a linked table.
////
pub const ERROR_ACCESSING_LINKED_TABLE_2: Integer = 90111;


/// The error with code <code>90112</code> is thrown when a row was deleted
/// twice while locking was disabled. This is an intern exception that should
/// never be thrown to the application, because such deleted should be
/// detected and the resulting exception ignored inside the database engine.
/// <pre>
/// Row not found when trying to delete from index UID_INDEX_0
/// </pre>
////
pub const ROW_NOT_FOUND_WHEN_DELETING_1: Integer = 90112;

/// The error with code <code>90113</code> is thrown when
/// the database URL contains unsupported settings.
/// Example:
/// <pre>
/// jdbc:h2:~/test;UNKNOWN:Integer=TRUE
/// </pre>
////
pub const UNSUPPORTED_SETTING_1: Integer = 90113;


/// The error with code <code>90114</code> is thrown when
/// trying to create a constant if a constant with this name already exists.
/// Example:
/// <pre>
/// CREATE CONSTANT TEST VALUE 1;
/// CREATE CONSTANT TEST VALUE 1;
/// </pre>
////
pub const CONSTANT_ALREADY_EXISTS_1: Integer = 90114;

/// The error with code <code>90115</code> is thrown when
/// trying to drop a constant that does not exists.
/// Example:
/// <pre>
/// DROP CONSTANT UNKNOWN;
/// </pre>
////
pub const CONSTANT_NOT_FOUND_1: Integer = 90115;


/// The error with code <code>90116</code> is thrown when
/// trying use a literal in a SQL statement if literals are disabled.
/// If literals are disabled, use PreparedStatement and parameters instead
/// of literals in the SQL statement.
/// Example:
/// <pre>
/// SET ALLOW_LITERALS NONE;
/// CALL 1+1;
/// </pre>
////
pub const LITERALS_ARE_NOT_ALLOWED: Integer = 90116;

/// The error with code <code>90117</code> is thrown when
/// trying to connect to a TCP server from another machine, if remote
/// connections are not allowed. To allow remote connections,
/// start the TCP server using the option -tcpAllowOthers as in:
/// <pre>
/// java org.h2.tools.Server -tcp -tcpAllowOthers
/// </pre>
/// Or, when starting the server from an application, use:
/// <pre>
/// Server server :Integer= Server.createTcpServer("-tcpAllowOthers");
/// server.start();
/// </pre>
////
pub const REMOTE_CONNECTION_NOT_ALLOWED: Integer = 90117;


/// The error with code <code>90118</code> is thrown when
/// trying to drop a table can not be dropped.
/// Example:
/// <pre>
/// DROP TABLE INFORMATION_SCHEMA.SETTINGS;
/// </pre>
////
pub const CANNOT_DROP_TABLE_1: Integer = 90118;


/// The error with code <code>90119</code> is thrown when
/// trying to create a domain if an object with this name already exists,
/// or when trying to overload a built-in data type.
/// Example:
/// <pre>
/// CREATE DOMAIN INTEGER AS VARCHAR;
/// CREATE DOMAIN EMAIL AS VARCHAR CHECK LOCATE('@', VALUE) &gt; 0;
/// CREATE DOMAIN EMAIL AS VARCHAR CHECK LOCATE('@', VALUE) &gt; 0;
/// </pre>
////
pub const DOMAIN_ALREADY_EXISTS_1: Integer = 90119;


/// Deprecated since 1.4.198. Use {@link #DOMAIN_ALREADY_EXISTS_1} instead.
////
pub const USER_DATA_TYPE_ALREADY_EXISTS_1: Integer = DOMAIN_ALREADY_EXISTS_1;


/// The error with code <code>90120</code> is thrown when
/// trying to drop a domain that doesn't exist.
/// Example:
/// <pre>
/// DROP DOMAIN UNKNOWN;
/// </pre>
////
pub const DOMAIN_NOT_FOUND_1: Integer = 90120;


/// Deprecated since 1.4.198. Use {@link #DOMAIN_NOT_FOUND_1} instead.
pub const USER_DATA_TYPE_NOT_FOUND_1: Integer = DOMAIN_NOT_FOUND_1;


/// The error with code <code>90121</code> is thrown when
/// a database operation is started while the virtual machine exits
/// (for example in a shutdown hook), or when the session is closed.
////
pub const DATABASE_CALLED_AT_SHUTDOWN: Integer = 90121;


/// The error with code <code>90122</code> is thrown when
/// WITH TIES clause is used without ORDER BY clause.
////
pub const WITH_TIES_WITHOUT_ORDER_BY: Integer = 90122;


/// The error with code <code>90123</code> is thrown when
/// trying mix regular parameters and indexed parameters in the same
/// statement. Example:
/// <pre>
/// SELECT ?, ?1 FROM DUAL;
/// </pre>
////
pub const CANNOT_MIX_INDEXED_AND_UNINDEXED_PARAMS: Integer = 90123;


/// The error with code <code>90124</code> is thrown when
/// trying to access a file that doesn't exist. This can occur when trying to
/// read a lob if the lob file has been deleted by another application.
////
pub const FILE_NOT_FOUND_1: Integer = 90124;


/// The error with code <code>90125</code> is thrown when
/// PreparedStatement.setBigDecimal is called with object that extends the
/// class BigDecimal, and the system property h2.allowBigDecimalExtensions is
/// not set. Using extensions of BigDecimal is dangerous because the database
/// relies on the behavior of BigDecimal. Example of wrong usage:
/// <pre>
/// BigDecimal bd :Integer= new MyDecimal("$10.3");
/// prep.setBigDecimal(1, bd);
/// Invalid class, expected java.math.BigDecimal but got MyDecimal
/// </pre>
/// Correct:
/// <pre>
/// BigDecimal bd :Integer= new BigDecimal(&quot;10.3&quot;);
/// prep.setBigDecimal(1, bd);
/// </pre>
////
pub const INVALID_CLASS_2: Integer = 90125;


/// The error with code <code>90126</code> is thrown when
/// trying to call the BACKUP statement for an in-memory database.
/// Example:
/// <pre>
/// jdbc:h2:mem:
/// BACKUP TO 'test.zip';
/// </pre>
////
pub const DATABASE_IS_NOT_PERSISTENT: Integer = 90126;


/// The error with code <code>90127</code> is thrown when
/// trying to update or delete a row in a result set if the result set is
/// not updatable. Result sets are only updatable if:
/// the statement was created with updatable concurrency;
/// all columns of the result set are from the same table;
/// the table is a data table (not a system table or view);
/// all columns of the primary key or any unique index are included;
/// all columns of the result set are columns of that table.
////
pub const RESULT_SET_NOT_UPDATABLE: Integer = 90127;


/// The error with code <code>90128</code> is thrown when
/// trying to call a method of the ResultSet that is only supported
/// for scrollable result sets, and the result set is not scrollable.
/// Example:
/// <pre>
/// rs.first();
/// </pre>
////
pub const RESULT_SET_NOT_SCROLLABLE: Integer = 90128;


/// The error with code <code>90129</code> is thrown when
/// trying to commit a transaction that doesn't exist.
/// Example:
/// <pre>
/// PREPARE COMMIT ABC;
/// COMMIT TRANSACTION TEST;
/// </pre>
////
pub const TRANSACTION_NOT_FOUND_1: Integer = 90129;


/// The error with code <code>90130</code> is thrown when
/// an execute method of PreparedStatement was called with a SQL statement.
/// This is not allowed according to the JDBC specification. Instead, use
/// an execute method of Statement.
/// Example of wrong usage:
/// <pre>
/// PreparedStatement prep :Integer= conn.prepareStatement("SELECT /// FROM TEST");
/// prep.execute("DELETE FROM TEST");
/// </pre>
/// Correct:
/// <pre>
/// Statement stat :Integer= conn.createStatement();
/// stat.execute("DELETE FROM TEST");
/// </pre>
////
pub const METHOD_NOT_ALLOWED_FOR_PREPARED_STATEMENT: Integer = 90130;


/// The error with code <code>90131</code> is thrown when using multi version
/// concurrency control, and trying to update the same row from within two
/// connections at the same time, or trying to insert two rows with the same
/// key from two connections. Example:
/// <pre>
/// jdbc:h2:~/test
/// Session 1:
/// CREATE TABLE TEST(ID );
/// INSERT INTO TEST VALUES(1);
/// SET AUTOCOMMIT FALSE;
/// UPDATE TEST SET ID :Integer= 2;
/// Session 2:
/// SET AUTOCOMMIT FALSE;
/// UPDATE TEST SET ID :Integer= 3;
/// </pre>
////
pub const CONCURRENT_UPDATE_1: Integer = 90131;


/// The error with code <code>90132</code> is thrown when
/// trying to drop a user-defined aggregate function that doesn't exist.
/// Example:
/// <pre>
/// DROP AGGREGATE UNKNOWN;
/// </pre>
////
pub const AGGREGATE_NOT_FOUND_1: Integer = 90132;


/// The error with code <code>90133</code> is thrown when
/// trying to change a specific database property while the database is
/// already open.
////
pub const CANNOT_CHANGE_SETTING_WHEN_OPEN_1: Integer = 90133;


/// The error with code <code>90134</code> is thrown when
/// trying to load a Java class that is not part of the allowed classes. By
/// default, all classes are allowed, but this can be changed using the
/// system property h2.allowedClasses.
////
pub const ACCESS_DENIED_TO_CLASS_1: Integer = 90134;


/// The error with code <code>90135</code> is thrown when
/// trying to open a connection to a database that is currently open
/// in exclusive mode. The exclusive mode is set using:
/// <pre>
/// SET EXCLUSIVE TRUE;
/// </pre>
////
pub const DATABASE_IS_IN_EXCLUSIVE_MODE: Integer = 90135;


/// The error with code <code>90136</code> is thrown when
/// trying to reference a window that does not exist.
/// Example:
/// <pre>
/// SELECT LEAD(X) OVER W FROM TEST;
/// </pre>
////
pub const WINDOW_NOT_FOUND_1: Integer = 90136;


/// The error with code <code>90137</code> is thrown when
/// trying to assign a value to something that is not a variable.
/// <pre>
/// SELECT AMOUNT, SET(@V, COALESCE(@V, 0)+AMOUNT) FROM TEST;
/// </pre>
////
pub const CAN_ONLY_ASSIGN_TO_VARIABLE_1: Integer = 90137;


/// The error with code <code>90138</code> is thrown when
///
/// trying to open a persistent database using an incorrect database name.
/// The name of a persistent database contains the path and file name prefix
/// where the data is stored. The file name part of a database name must be
/// at least two characters.
///
/// Example of wrong usage:
/// <pre>
/// DriverManager.getConnection("jdbc:h2:~/t");
/// DriverManager.getConnection("jdbc:h2:~/test/");
/// </pre>
/// Correct:
/// <pre>
/// DriverManager.getConnection("jdbc:h2:~/te");
/// DriverManager.getConnection("jdbc:h2:~/test/te");
/// </pre>
////
pub const INVALID_DATABASE_NAME_1: Integer = 90138;


/// The error with code <code>90139</code> is thrown when
/// the specified pub const Java method was not found in the class.
/// Example:
/// <pre>
/// CREATE ALIAS TEST FOR "java.lang.Math.test";
/// </pre>
////
pub const PUBLIC_STATIC_JAVA_METHOD_NOT_FOUND_1: Integer = 90139;


/// The error with code <code>90140</code> is thrown when trying to update or
/// delete a row in a result set if the statement was not created with
/// updatable concurrency. Result sets are only updatable if the statement
/// was created with updatable concurrency, and if the result set contains
/// all columns of the primary key or of a unique index of a table.
////
pub const RESULT_SET_READONLY: Integer = 90140;


/// The error with code <code>90141</code> is thrown when
/// trying to change the java object serializer while there was already data
/// in the database. The serializer of the database must be set when the
/// database is empty.
////
pub const JAVA_OBJECT_SERIALIZER_CHANGE_WITH_DATA_TABLE: Integer = 90141;


/// The error with code <code>90142</code> is thrown when
/// trying to set zero for step size.
////
pub const STEP_SIZE_MUST_NOT_BE_ZERO: Integer = 90142;


/// The error with code <code>90143</code> is thrown when
/// trying to fetch a row from the primary index and the row is not there.
////
pub const ROW_NOT_FOUND_IN_PRIMARY_INDEX: Integer = 90143;


/// The error with code <code>90144</code> is thrown when
/// user trying to login into a database with AUTHREALM set and
/// the target database doesn't have an authenticator defined
/// <p>Authenticator experimental feature can be enabled by
/// </p>
/// <pre>
/// SET AUTHENTICATOR TRUE
/// </pre>
////
pub const AUTHENTICATOR_NOT_AVAILABLE: Integer = 90144;


/// The error with code <code>90145</code> is thrown when trying to execute a
/// SELECT statement with non-window aggregates, DISTINCT, GROUP BY, or
/// HAVING clauses together with FOR UPDATE clause.
///
/// <pre>
/// SELECT DISTINCT NAME FOR UPDATE;
/// SELECT MAX(VALUE) FOR UPDATE;
/// </pre>
////
pub const FOR_UPDATE_IS_NOT_ALLOWED_IN_DISTINCT_OR_GROUPED_SELECT: Integer = 90145;


/// The error with code <code>90146</code> is thrown when trying to open a
/// database that does not exist using the flag IFEXISTS:Integer=TRUE
/// <pre>
/// jdbc:h2:./database_that_does_not_exist
/// </pre>
////
pub const DATABASE_NOT_FOUND_WITH_IF_EXISTS_1: Integer = 90146;


/// The error with code <code>90147</code> is thrown when trying to execute a
/// statement which closes the transaction (such as commit and rollback) and
/// autocommit mode is on.
///
/// @see org.h2.engine.SysProperties#FORCE_AUTOCOMMIT_OFF_ON_COMMIT
////
pub const METHOD_DISABLED_ON_AUTOCOMMIT_TRUE: Integer = 90147;


/// The error with code <code>90148</code> is thrown when trying to access
/// the current value of a sequence before execution of NEXT VALUE FOR
/// sequenceName in the current session. Example:
///
/// <pre>
/// SELECT CURRENT VALUE FOR SEQUENCE XYZ;
/// </pre>
////
pub const CURRENT_SEQUENCE_VALUE_IS_NOT_DEFINED_IN_SESSION_1: Integer = 90148;


/// The error with code <code>90149</code> is thrown when trying to open a
/// database that does not exist remotely without enabling remote database
/// creation first.
/// <pre>
/// jdbc:h2:./database_that_does_not_exist
/// </pre>
////
pub const REMOTE_DATABASE_NOT_FOUND_1: Integer = 90149;


/// The error with code <code>90150</code> is thrown when
/// trying to use an invalid precision.
/// Example:
/// <pre>
/// CREATE TABLE TABLE1 ( FAIL INTERVAL YEAR(20) );
/// </pre>
////
pub const INVALID_VALUE_PRECISION: Integer = 90150;


/// The error with code <code>90151</code> is thrown when
/// trying to use an invalid scale or fractional seconds precision.
/// Example:
/// <pre>
/// CREATE TABLE TABLE1 ( FAIL TIME(10) );
/// </pre>
////
pub const INVALID_VALUE_SCALE: Integer = 90151;


/// The error with code <code>90152</code> is thrown when trying to manually
/// drop a unique or primary key constraint that is referenced by a foreign
/// key constraint without a CASCADE clause.
///
/// <pre>
/// CREATE TABLE PARENT(ID  CONSTRAINT P1 PRIMARY KEY);
/// CREATE TABLE CHILD(ID  CONSTRAINT P2 PRIMARY KEY, CHILD  CONSTRAINT C REFERENCES PARENT);
/// ALTER TABLE PARENT DROP CONSTRAINT P1 RESTRICT;
/// </pre>
////
pub const CONSTRAINT_IS_USED_BY_CONSTRAINT_2: Integer = 90152;


/// The error with code <code>90153</code> is thrown when trying to reference
/// a column of another data type when data types aren't comparable or don't
/// have a session-independent compare order between each other.
///
/// <pre>
/// CREATE TABLE PARENT(T TIMESTAMP UNIQUE);
/// CREATE TABLE CHILD(T TIMESTAMP WITH TIME ZONE REFERENCES PARENT(T));
/// </pre>
////
pub const UNCOMPARABLE_REFERENCED_COLUMN_2: Integer = 90153;


/// The error with code <code>90154</code> is thrown when trying to assign a
/// value to a generated column.
///
/// <pre>
/// CREATE TABLE TEST(A , B  GENERATED ALWAYS AS (A + 1));
/// INSERT INTO TEST(A, B) VALUES (1, 1);
/// </pre>
////
pub const GENERATED_COLUMN_CANNOT_BE_ASSIGNED_1: Integer = 90154;


/// The error with code <code>90155</code> is thrown when trying to create a
/// referential constraint that can update a referenced generated column.
///
/// <pre>
/// CREATE TABLE PARENT(ID  PRIMARY KEY, K  GENERATED ALWAYS AS (ID) UNIQUE);
/// CREATE TABLE CHILD(ID  PRIMARY KEY, P );
/// ALTER TABLE CHILD ADD FOREIGN KEY(P) REFERENCES PARENT(K) ON DELETE SET NULL;
/// </pre>
////
pub const GENERATED_COLUMN_CANNOT_BE_UPDATABLE_BY_CONSTRAINT_2: Integer = 90155;


/// The error with code <code>90156</code> is thrown when trying to create a
/// view or a table from a select and some expression doesn't have a column
/// name or alias when it is required by a compatibility mode.
///
/// <pre>
/// SET MODE DB2;
/// CREATE TABLE T1(A , B );
/// CREATE TABLE T2 AS (SELECT A + B FROM T1) WITH DATA;
/// </pre>
////
pub const COLUMN_ALIAS_IS_NOT_SPECIFIED_1: Integer = 90156;


/// The error with code <code>90157</code> is thrown when the integer
/// index that is used in the GROUP BY is not in the SELECT list
////
pub const GROUP_BY_NOT_IN_THE_RESULT: Integer = 90157;


pub fn get_state(error_code: Integer) -> String {
    match error_code {
        // 02: no data
        NO_DATA_AVAILABLE => "02000".to_string(),

        // 07: dynamic SQL error
        INVALID_PARAMETER_COUNT_2 => "07001".to_string(),

        // 08: connection exception
        ERROR_OPENING_DATABASE_1 => "08000".to_string(),

        // 21: cardinality violation
        COLUMN_COUNT_DOES_NOT_MATCH => "21S02".to_string(),

        // 22: data exception
        ARRAY_ELEMENT_ERROR_2 => "2202E".to_string(),

        // 42: syntax error or access rule violation
        TABLE_OR_VIEW_ALREADY_EXISTS_1 => "42S01".to_string(),
        TABLE_OR_VIEW_NOT_FOUND_1 => "42S02".to_string(),
        TABLE_OR_VIEW_NOT_FOUND_WITH_CANDIDATES_2 => "42S03".to_string(),
        TABLE_OR_VIEW_NOT_FOUND_DATABASE_EMPTY_1 => "42S04".to_string(),
        INDEX_ALREADY_EXISTS_1 => "42S11".to_string(),
        INDEX_NOT_FOUND_1 => "42S12".to_string(),
        DUPLICATE_COLUMN_NAME_1 => "42S21".to_string(),
        COLUMN_NOT_FOUND_1 => "42S22".to_string(),
        IDENTICAL_EXPRESSIONS_SHOULD_BE_USED => "42S31".to_string(),

        // 0A: feature not supported

        // HZ: remote database access

        // HY
        GENERAL_ERROR_1 => "HY000".to_string(),
        UNKNOWN_DATA_TYPE_1 => "HY004".to_string(),

        FEATURE_NOT_SUPPORTED_1 => "HYC00".to_string(),
        LOCK_TIMEOUT_1 => "HYT00".to_string(),
        _ => error_code.to_string()
    }
}