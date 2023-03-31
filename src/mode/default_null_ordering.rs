use anyhow::Result;
use std::collections::HashMap;
use crate::api::error_code;
use crate::h2_rust_common::Integer;
use crate::message::db_error::DbError;
use crate::result::sort_orders;
use crate::throw;

/// NULL values are considered as smaller than other values during sorting.
pub static LOW: DefaultNullOrdering = DefaultNullOrdering::new(sort_orders::NULLS_FIRST,
                                                               sort_orders::NULLS_LAST);

/// NULL values are considered as larger than other values during sorting.
pub static HIGH: DefaultNullOrdering = DefaultNullOrdering::new(sort_orders::NULLS_LAST,
                                                                sort_orders::NULLS_FIRST);

/// NULL values are sorted after other values, no matter if ascending or descending order is used.
pub static LAST: DefaultNullOrdering = DefaultNullOrdering::new(sort_orders::NULLS_LAST,
                                                                sort_orders::NULLS_LAST);

/// NULL values are sorted before other values, no matter if ascending or descending order is used.
pub static FIRST: DefaultNullOrdering = DefaultNullOrdering::new(sort_orders::NULLS_FIRST,
                                                                 sort_orders::NULLS_FIRST);

pub struct DefaultNullOrdering {
    default_asc_nulls: Integer,
    default_desc_nulls: Integer,
    null_asc: Integer,
    null_desc: Integer,
}

impl DefaultNullOrdering {
    pub const fn new(default_asc_nulls: Integer, default_desc_nulls: Integer) -> Self {
        let null_asc = if default_asc_nulls == sort_orders::NULLS_FIRST { -1 } else { 1 };
        let null_desc = if default_desc_nulls == sort_orders::NULLS_FIRST { -1 } else { 1 };
        DefaultNullOrdering {
            default_asc_nulls,
            default_desc_nulls,
            null_asc,
            null_desc,
        }
    }

    pub fn value_of(name: &str) -> Option<&'static Self> {
        match name {
            "LOW" => Some(&LOW),
            "HIGH" => Some(&HIGH),
            "LAST" => Some(&LAST),
            "FIRST" => Some(&FIRST),
            _ => None
        }
    }
}

