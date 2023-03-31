use crate::h2_rust_common::Integer;

/// This bit mask means the values should be sorted in ascending order.
pub const ASCENDING: Integer = 0;

/// This bit mask means the values should be sorted in descending order.
pub const DESCENDING: Integer = 1;

/// This bit mask means NULLs should be sorted before other data, no matter
/// if ascending or descending order is used.
pub const NULLS_FIRST: Integer = 2;

/// This bit mask means NULLs should be sorted after other data, no matter
/// if ascending or descending order is used.
pub const NULLS_LAST: Integer = 4;

pub struct SortOrder {}