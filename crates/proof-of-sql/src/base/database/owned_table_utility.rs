//! Utility functions for creating OwnedTables and OwnedColumns.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
//! let result = owned_table::<Curve25519Scalar>([
//!     bigint("a", [1, 2, 3]),
//!     boolean("b", [true, false, true]),
//!     int128("c", [1, 2, 3]),
//!     scalar("d", [1, 2, 3]),
//!     varchar("e", ["a", "b", "c"]),
//!     decimal75("f", 12, 1, [1, 2, 3]),
//! ]);
//! ```
use super::{OwnedColumn, OwnedTable};
use crate::base::scalar::Scalar;
use core::ops::Deref;
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone, PoSQLTimestamp},
    Identifier,
};

/// Creates an OwnedTable from a list of (Identifier, OwnedColumn) pairs.
/// This is a convenience wrapper around OwnedTable::try_from_iter primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [OwnedTable::try_from_iter] for more details.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3]),
///     boolean("b", [true, false, true]),
///     int128("c", [1, 2, 3]),
///     scalar("d", [1, 2, 3]),
///     varchar("e", ["a", "b", "c"]),
///     decimal75("f", 12, 1, [1, 2, 3]),
/// ]);
/// ```
pub fn owned_table<S: Scalar>(
    iter: impl IntoIterator<Item = (Identifier, OwnedColumn<S>)>,
) -> OwnedTable<S> {
    OwnedTable::try_from_iter(iter).unwrap()
}

/// Creates a (Identifier, OwnedColumn) pair for a smallint column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     smallint("a", [1_i16, 2, 3]),
/// ]);
pub fn smallint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i16>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::SmallInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for an int column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     int("a", [1, 2, 3]),
/// ]);
pub fn int<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i32>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Int(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a bigint column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     bigint("a", [1, 2, 3]),
/// ]);
pub fn bigint<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i64>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::BigInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a boolean column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     boolean("a", [true, false, true]),
/// ]);
/// ```
pub fn boolean<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<bool>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Boolean(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a int128 column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     int128("a", [1, 2, 3]),
/// ]);
/// ```
pub fn int128<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<i128>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Int128(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a scalar column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     scalar("a", [1, 2, 3]),
/// ]);
/// ```
pub fn scalar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Scalar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a varchar column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     varchar("a", ["a", "b", "c"]),
/// ]);
/// ```
pub fn varchar<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = impl Into<String>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::VarChar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [owned_table].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, scalar::Curve25519Scalar};
/// let result = owned_table::<Curve25519Scalar>([
///     decimal75("a", 12, 1, [1, 2, 3]),
/// ]);
/// ```
pub fn decimal75<S: Scalar>(
    name: impl Deref<Target = str>,
    precision: u8,
    scale: i8,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Identifier, OwnedColumn<S>) {
    (
        name.parse().unwrap(),
        OwnedColumn::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            data.into_iter().map(Into::into).collect(),
        ),
    )
}

/// Creates a (Identifier, OwnedColumn) pair for a timestamp column.
/// This is primarily intended for use in conjunction with [owned_table].
///
/// # Parameters
/// - `name`: The name of the column.
/// - `data`: The data for the column, provided as an iterator over `i64` values representing time since the unix epoch.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*,
///     scalar::Curve25519Scalar,
/// };
/// use proof_of_sql_parser::{
///    posql_time::{PoSQLTimeZone, PoSQLTimeUnit}};
///
/// let result = owned_table::<Curve25519Scalar>([
///     timestamp("event_time", vec![
///            "1969-12-31T23:59:59Z", // One second before the Unix epoch
///            "1970-01-01T00:00:00Z", // The Unix epoch
///            "1970-01-01T00:00:01Z", // One second after the Unix epoch
///        ].iter().map(|s| s.to_string())),
/// ]);
/// ```
pub fn timestamp<S: Scalar>(
    name: impl Deref<Target = str>,
    data: impl IntoIterator<Item = String>,
) -> (Identifier, OwnedColumn<S>) {
    let mut parsed_data: Vec<PoSQLTimestamp> = data
        .into_iter()
        .map(|timestamp_str| {
            PoSQLTimestamp::try_from(&timestamp_str).expect("Failed to parse timestamp")
        })
        .collect();

    // Handling the case where parsed_data might be empty by providing a default PoSQLTimestamp
    if parsed_data.is_empty() {
        parsed_data
            .push(PoSQLTimestamp::to_timestamp(0).expect("Failed to create default timestamp"));
    }

    // Build the OwnedColumn using the time unit from the first timestamp in parsed_data
    (
        name.parse().unwrap(),
        OwnedColumn::TimestampTZ(
            // We assume all timestamps have the same time unit, so take the unit from the first timestamp.
            parsed_data
                .first()
                .expect("No timestamps provided")
                .timeunit,
            PoSQLTimeZone::Utc,
            parsed_data
                .into_iter()
                .map(|ts| match ts.timeunit {
                    PoSQLTimeUnit::Nanosecond => ts.timestamp.timestamp_nanos_opt().unwrap(),
                    PoSQLTimeUnit::Microsecond => {
                        ts.timestamp.timestamp_nanos_opt().unwrap() / 1_000
                    }
                    PoSQLTimeUnit::Millisecond => {
                        ts.timestamp.timestamp_nanos_opt().unwrap() / 1_000_000
                    }
                    PoSQLTimeUnit::Second => ts.timestamp.timestamp(),
                })
                .collect(),
        ),
    )
}
#[macro_export]
/// Macro to convert a given Unix timestamp in seconds into an RFC 3339 formatted string.
macro_rules! epoch_to_rfc3339 {
    ($x:expr) => {
        Utc.timestamp_opt($x, 0).unwrap().to_rfc3339()
    };
}
