//! Utility functions for creating [`OwnedTable`]s and [`OwnedColumn`]s.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use proof_of_sql::base::{database::owned_table_utility::*};
//! # use proof_of_sql::base::scalar::MontScalar;
//! # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
//! let result = owned_table::<MyScalar>([
//!     bigint("a", [1, 2, 3]),
//!     boolean("b", [true, false, true]),
//!     int128("c", [1, 2, 3]),
//!     scalar("d", [1, 2, 3]),
//!     varchar("e", ["a", "b", "c"]),
//!     decimal75("f", 12, 1, [1, 2, 3]),
//! ]);
//! ```
use super::{OwnedColumn, OwnedTable};
use crate::base::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
};
use alloc::{string::String, vec::Vec};
use sqlparser::ast::Ident;

/// Creates an [`OwnedTable`] from a list of `(Ident, OwnedColumn)` pairs.
/// This is a convenience wrapper around [`OwnedTable::try_from_iter`] primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [bigint], [boolean], etc).
/// The function will panic under a variety of conditions. See [`OwnedTable::try_from_iter`] for more details.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///      bigint("a", [1, 2, 3]),
///      boolean("b", [true, false, true]),
///      int128("c", [1, 2, 3]),
///      scalar("d", [1, 2, 3]),
///      varchar("e", ["a", "b", "c"]),
///      decimal75("f", 12, 1, [1, 2, 3]),
/// ]);
/// ```
/// ///
/// # Panics
/// - Panics if converting the iterator into an `OwnedTable<S>` fails.
pub fn owned_table<S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, OwnedColumn<S>)>,
) -> OwnedTable<S> {
    OwnedTable::try_from_iter(iter).unwrap()
}

/// Creates a (Ident, `OwnedColumn`) pair for a uint8 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     uint8("a", [1_u8, 2, 3]),
/// ]);
///```
pub fn uint8<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<u8>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Uint8(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a (Ident, `OwnedColumn`) pair for a tinyint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     tinyint("a", [1_i8, 2, 3]),
/// ]);
///```
pub fn tinyint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i8>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::TinyInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a smallint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     smallint("a", [1_i16, 2, 3]),
/// ]);
/// ```
pub fn smallint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i16>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::SmallInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for an int column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     int("a", [1, 2, 3]),
/// ]);
/// ```
pub fn int<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i32>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Int(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a bigint column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```rust
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     bigint("a", [1, 2, 3]),
/// ]);
/// ```
pub fn bigint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i64>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::BigInt(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a boolean column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     boolean("a", [true, false, true]),
/// ]);
/// ```
pub fn boolean<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<bool>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Boolean(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a int128 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     int128("a", [1, 2, 3]),
/// ]);
/// ```
pub fn int128<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i128>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Int128(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a scalar column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     scalar("a", [1, 2, 3]),
/// ]);
/// ```
pub fn scalar<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Scalar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a varchar column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     varchar("a", ["a", "b", "c"]),
/// ]);
/// ```
pub fn varchar<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<String>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::VarChar(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a varbinary column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///    varbinary("a", [[1, 2, 3], [4, 5, 6], [7, 8, 9]]),
/// ]);
/// ```
pub fn varbinary<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<Vec<u8>>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::VarBinary(data.into_iter().map(Into::into).collect()),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [`owned_table`].
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     decimal75("a", 12, 1, [1, 2, 3]),
/// ]);
/// ```
///
/// # Panics
/// - Panics if creating the `Precision` from the specified precision value fails.
pub fn decimal75<S: Scalar>(
    name: impl Into<Ident>,
    precision: u8,
    scale: i8,
    data: impl IntoIterator<Item = impl Into<S>>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            data.into_iter().map(Into::into).collect(),
        ),
    )
}

/// Creates a `(Ident, OwnedColumn)` pair for a timestamp column.
/// This is primarily intended for use in conjunction with [`owned_table`].
///
/// # Parameters
/// - `name`: The name of the column.
/// - `time_unit`: The time unit of the timestamps.
/// - `timezone`: The timezone for the timestamps.
/// - `data`: The data for the column, provided as an iterator over `i64` values representing time since the unix epoch.
///
/// # Example
/// ```
/// use proof_of_sql::base::{database::owned_table_utility::*, posql_time::{PoSQLTimeZone, PoSQLTimeUnit}};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let result = owned_table::<MyScalar>([
///     timestamptz("event_time", PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![1625072400, 1625076000, 1625079600]),
/// ]);
/// ```
pub fn timestamptz<S: Scalar>(
    name: impl Into<Ident>,
    time_unit: PoSQLTimeUnit,
    timezone: PoSQLTimeZone,
    data: impl IntoIterator<Item = i64>,
) -> (Ident, OwnedColumn<S>) {
    (
        name.into(),
        OwnedColumn::TimestampTZ(time_unit, timezone, data.into_iter().collect()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};

    #[test]
    fn constructors_build_expected_owned_columns() {
        let table = owned_table::<TestScalar>([
            boolean("bools", [true, false]),
            uint8("u8s", [1_u8, 2]),
            tinyint("i8s", [-1_i8, 2]),
            smallint("i16s", [-10_i16, 20]),
            int("i32s", [-100_i32, 200]),
            bigint("i64s", [-1000_i64, 2000]),
            int128("i128s", [-10000_i128, 20000]),
            scalar("scalars", [3_u64, 4]),
            varchar("strings", ["a", "b"]),
            varbinary("bytes", [vec![1_u8, 2], vec![3]]),
            decimal75("decimals", 12, 2, [5_u64, 6]),
            timestamptz(
                "timestamps",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [7_i64, 8],
            ),
        ]);

        assert_eq!(table.num_columns(), 12);
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table["bools"], OwnedColumn::Boolean(vec![true, false]));
        assert_eq!(table["u8s"], OwnedColumn::Uint8(vec![1, 2]));
        assert_eq!(table["i8s"], OwnedColumn::TinyInt(vec![-1, 2]));
        assert_eq!(table["i16s"], OwnedColumn::SmallInt(vec![-10, 20]));
        assert_eq!(table["i32s"], OwnedColumn::Int(vec![-100, 200]));
        assert_eq!(table["i64s"], OwnedColumn::BigInt(vec![-1000, 2000]));
        assert_eq!(table["i128s"], OwnedColumn::Int128(vec![-10000, 20000]));
        assert_eq!(
            table["scalars"],
            OwnedColumn::Scalar(vec![TestScalar::from(3_u64), TestScalar::from(4_u64)])
        );
        assert_eq!(
            table["strings"],
            OwnedColumn::VarChar(vec!["a".to_string(), "b".to_string()])
        );
        assert_eq!(
            table["bytes"],
            OwnedColumn::VarBinary(vec![vec![1, 2], vec![3]])
        );
        assert_eq!(
            table["decimals"],
            OwnedColumn::Decimal75(
                Precision::new(12).unwrap(),
                2,
                vec![TestScalar::from(5_u64), TestScalar::from(6_u64)],
            )
        );
        assert_eq!(
            table["timestamps"],
            OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![7, 8])
        );
    }

    #[test]
    #[should_panic(expected = "ColumnLengthMismatch")]
    fn owned_table_panics_for_mismatched_column_lengths() {
        owned_table::<TestScalar>([bigint("a", [1, 2]), bigint("b", [3])]);
    }
}
