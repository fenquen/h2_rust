use std::collections::HashMap;
use anyhow::Result;
use lazy_static::lazy_static;
use crate::engine::mode::ModeEnum::REGULAR;
use crate::enum_str;
use crate::util::string_utils;
lazy_static! {
    static ref MODES:HashMap<String, Mode> = {
        let mut mods = HashMap::new();

        let mut mode = Mode::new(ModeEnum::REGULAR);

        mode.allow_empty_in_predicate = true;
        mode.date_time_value_within_transaction = true;
        mode.top_in_select = true;
        mode.limit = true;
        mode.minus_is_except = true;
        mode.identity_data_type = true;
        mode.serial_data_types = true;
        mode.auto_increment_clause = true;

        mods.insert(string_utils::to_upper_english(&mode.name), mode);

        mods
    };
}

#[derive(Default)]
pub struct Mode {
    pub allow_empty_in_predicate: bool,
    pub date_time_value_within_transaction: bool,
    pub top_in_select: bool,
    pub limit: bool,
    pub minus_is_except: bool,
    pub identity_data_type: bool,
    pub serial_data_types: bool,
    pub auto_increment_clause: bool,
    pub name: String,

    pub mode_enum: ModeEnum,
}

impl Mode {
    pub fn new(mode_enum: ModeEnum) -> Self {
        let mut mode: Mode = Default::default();
        mode.name = mode_enum.name().to_string();
        mode.mode_enum = mode_enum;
        mode
    }

    pub fn get_regular() -> Option<&'static Mode> {
        Self::get_instance(REGULAR.name())
    }

    pub fn get_instance(name: &str) -> Option<&'static Mode> {
        MODES.get(&string_utils::to_upper_english(name))
    }
}

enum_str! {
    pub enum ModeEnum {
        REGULAR,
        STRICT,
        LEGACY,
        DB2,
        Derby,
        MariaDB,
        MSSQLServer,
        HSQLDB,
        MySQL,
        Oracle,
        PostgreSQL,
}
}

impl Default for ModeEnum {
    fn default() -> Self {
        REGULAR
    }
}


/// Determines how rows with {@code NULL} values in indexed columns are handled in unique indexes.
pub enum UniqueIndexNullsHandling {
    /// Multiple rows with identical values in indexed columns with at least one <br>
    /// indexed {@code NULL} value are allowed in unique index.
    AllowDuplicatesWithAnyNull,

    /// Multiple rows with identical values in indexed columns with all indexed
    /// {@code NULL} values are allowed in unique index.
    AllowDuplicatesWithAllNulls,

    /// Multiple rows with identical values in indexed columns are not allowed in unique index.
    ForbidAnyDuplicates,
}

/// Generation of column names for expressions.
pub enum ExpressionNames {
    /// Use optimized SQL representation of expression.
    OptimizedSql,

    /// Use original SQL representation of expression.
    OriginalSql,

    /// Generate empty name.
    EMPTY,

    /// Use ordinal number of a column.
    NUMBER,

    /// Use ordinal number of a column with C prefix.
    CNumber,

    /// Use function name for functions and ?column? for other expressions
    PostgresqlStyle,
}

/// Generation of column names for expressions to be used in a view.
pub enum ViewExpressionNames {
    /// Use both specified and generated names as is.
    AsIs,

    /// Throw exception for unspecified names.
    EXCEPTION,

    /// Use both specified and generated names as is, but replace too long
    /// generated names with {@code Name_exp_###}.
    MysqlStyle,
}

/// When CHAR values are right-padded with spaces.
pub enum CharPadding {
    /// CHAR values are always right-padded with spaces.
    ALWAYS,

    /// Spaces are trimmed from the right side of CHAR values, but CHAR
    /// values in result sets are right-padded with spaces to the declared length
    InResultSets,

    /// Spaces are trimmed from the right side of CHAR values.
    NEVER,
}