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
    use crate::base::{
        database::OwnedColumn,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn uint8_creates_correct_ident() {
        let (ident, _) = uint8::<TestScalar>("col_a", [1u8, 2, 3]);
        assert_eq!(ident.value, "col_a");
    }

    #[test]
    fn uint8_creates_uint8_column_with_correct_data() {
        let (_, col) = uint8::<TestScalar>("a", [10u8, 20, 30]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Uint8(alloc::vec![10, 20, 30]));
    }

    #[test]
    fn uint8_empty_data_creates_empty_column() {
        let (_, col) = uint8::<TestScalar>("a", [] as [u8; 0]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Uint8(alloc::vec![]));
    }

    #[test]
    fn uint8_single_element() {
        let (_, col) = uint8::<TestScalar>("a", [255u8]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Uint8(alloc::vec![255u8]));
    }

    #[test]
    fn tinyint_creates_correct_column() {
        let (_, col) = tinyint::<TestScalar>("a", [-1i8, 0, 1]);
        assert_eq!(col, OwnedColumn::<TestScalar>::TinyInt(alloc::vec![-1i8, 0, 1]));
    }

    #[test]
    fn tinyint_boundary_values() {
        let (_, col) = tinyint::<TestScalar>("a", [i8::MIN, i8::MAX]);
        assert_eq!(col, OwnedColumn::<TestScalar>::TinyInt(alloc::vec![i8::MIN, i8::MAX]));
    }

    #[test]
    fn smallint_creates_correct_column() {
        let (_, col) = smallint::<TestScalar>("a", [100i16, 200, 300]);
        assert_eq!(col, OwnedColumn::<TestScalar>::SmallInt(alloc::vec![100i16, 200, 300]));
    }

    #[test]
    fn smallint_negative_and_positive() {
        let (_, col) = smallint::<TestScalar>("x", [-100i16, 0, 100]);
        assert_eq!(col, OwnedColumn::<TestScalar>::SmallInt(alloc::vec![-100i16, 0, 100]));
    }

    #[test]
    fn int_creates_correct_column() {
        let (_, col) = int::<TestScalar>("a", [1i32, 2, 3]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Int(alloc::vec![1i32, 2, 3]));
    }

    #[test]
    fn int_negative_values() {
        let (_, col) = int::<TestScalar>("a", [-1i32, -2, -3]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Int(alloc::vec![-1i32, -2, -3]));
    }

    #[test]
    fn bigint_creates_correct_column() {
        let (_, col) = bigint::<TestScalar>("a", [1i64, 2, 3]);
        assert_eq!(col, OwnedColumn::<TestScalar>::BigInt(alloc::vec![1i64, 2, 3]));
    }

    #[test]
    fn bigint_with_zero_and_negatives() {
        let (_, col) = bigint::<TestScalar>("a", [-10i64, 0, 10]);
        assert_eq!(col, OwnedColumn::<TestScalar>::BigInt(alloc::vec![-10i64, 0, 10]));
    }

    #[test]
    fn bigint_single_element() {
        let (_, col) = bigint::<TestScalar>("a", [99i64]);
        assert_eq!(col, OwnedColumn::<TestScalar>::BigInt(alloc::vec![99i64]));
    }

    #[test]
    fn boolean_creates_correct_column() {
        let (_, col) = boolean::<TestScalar>("a", [true, false, true]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Boolean(alloc::vec![true, false, true]));
    }

    #[test]
    fn boolean_all_true() {
        let (_, col) = boolean::<TestScalar>("a", [true, true, true]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Boolean(alloc::vec![true, true, true]));
    }

    #[test]
    fn boolean_all_false() {
        let (_, col) = boolean::<TestScalar>("a", [false, false]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Boolean(alloc::vec![false, false]));
    }

    #[test]
    fn int128_creates_correct_column() {
        let (_, col) = int128::<TestScalar>("a", [1i128, 2, 3]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Int128(alloc::vec![1i128, 2, 3]));
    }

    #[test]
    fn int128_large_values() {
        let v = i128::MAX;
        let (_, col) = int128::<TestScalar>("a", [v]);
        assert_eq!(col, OwnedColumn::<TestScalar>::Int128(alloc::vec![v]));
    }

    #[test]
    fn scalar_creates_column_with_correct_length() {
        let (_, col) = scalar::<TestScalar>("a", [1u64, 2, 3]);
        assert!(matches!(col, OwnedColumn::Scalar(_)));
        if let OwnedColumn::Scalar(v) = col {
            assert_eq!(v.len(), 3);
        }
    }

    #[test]
    fn varchar_creates_correct_column() {
        let (_, col) = varchar::<TestScalar>("a", ["hello", "world"]);
        assert_eq!(
            col,
            OwnedColumn::<TestScalar>::VarChar(alloc::vec![
                alloc::string::String::from("hello"),
                alloc::string::String::from("world"),
            ])
        );
    }

    #[test]
    fn varchar_empty_data_creates_empty_column() {
        let (_, col) = varchar::<TestScalar>("a", [] as [&str; 0]);
        assert_eq!(col, OwnedColumn::<TestScalar>::VarChar(alloc::vec![]));
    }

    #[test]
    fn varchar_with_empty_strings() {
        let (_, col) = varchar::<TestScalar>("a", ["", "x", ""]);
        if let OwnedColumn::VarChar(v) = col {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], "");
            assert_eq!(v[1], "x");
        } else {
            panic!("expected VarChar");
        }
    }

    #[test]
    fn varbinary_creates_correct_column() {
        let (_, col) = varbinary::<TestScalar>("a", [alloc::vec![1u8, 2], alloc::vec![3u8, 4]]);
        assert_eq!(
            col,
            OwnedColumn::<TestScalar>::VarBinary(alloc::vec![
                alloc::vec![1u8, 2],
                alloc::vec![3u8, 4],
            ])
        );
    }

    #[test]
    fn varbinary_empty() {
        let (_, col) = varbinary::<TestScalar>("a", [] as [alloc::vec::Vec<u8>; 0]);
        assert_eq!(col, OwnedColumn::<TestScalar>::VarBinary(alloc::vec![]));
    }

    #[test]
    fn decimal75_creates_column_with_correct_precision_and_scale() {
        let (_, col) = decimal75::<TestScalar>("a", 12, 1, [1u64, 2, 3]);
        let precision = Precision::new(12).unwrap();
        assert!(matches!(col, OwnedColumn::Decimal75(p, 1, _) if p == precision));
        if let OwnedColumn::Decimal75(_, _, data) = col {
            assert_eq!(data.len(), 3);
        }
    }

    #[test]
    fn decimal75_with_zero_scale() {
        let (_, col) = decimal75::<TestScalar>("a", 10, 0, [100u64]);
        let precision = Precision::new(10).unwrap();
        assert!(matches!(col, OwnedColumn::Decimal75(p, 0, _) if p == precision));
    }

    #[test]
    fn timestamptz_creates_correct_column_utc() {
        let data = alloc::vec![1625072400i64, 1625076000];
        let (ident, col) = timestamptz::<TestScalar>(
            "event_time",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            data.clone(),
        );
        assert_eq!(ident.value, "event_time");
        assert_eq!(
            col,
            OwnedColumn::<TestScalar>::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                data,
            )
        );
    }

    #[test]
    fn timestamptz_millisecond_unit() {
        let (_, col) = timestamptz::<TestScalar>(
            "t",
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
            alloc::vec![0i64],
        );
        assert!(matches!(col, OwnedColumn::TimestampTZ(PoSQLTimeUnit::Millisecond, _, _)));
    }

    #[test]
    fn owned_table_empty_creates_zero_column_table() {
        let t = owned_table::<TestScalar>([]);
        assert_eq!(t.num_columns(), 0);
    }

    #[test]
    fn owned_table_single_bigint_column() {
        let t = owned_table::<TestScalar>([bigint("a", [1i64, 2, 3])]);
        assert_eq!(t.num_columns(), 1);
        assert_eq!(t.num_rows(), 3);
    }

    #[test]
    fn owned_table_multiple_columns() {
        let t = owned_table::<TestScalar>([
            bigint("a", [1i64, 2, 3]),
            boolean("b", [true, false, true]),
        ]);
        assert_eq!(t.num_columns(), 2);
        assert_eq!(t.num_rows(), 3);
    }

    #[test]
    fn owned_table_column_accessible_by_index() {
        let t = owned_table::<TestScalar>([bigint("a", [42i64])]);
        assert_eq!(
            t.column_by_index(0),
            Some(&OwnedColumn::BigInt(alloc::vec![42i64]))
        );
    }

    #[test]
    fn owned_table_all_numeric_types() {
        let t = owned_table::<TestScalar>([
            uint8("u", [1u8]),
            tinyint("t", [2i8]),
            smallint("s", [3i16]),
            int("i", [4i32]),
            bigint("b", [5i64]),
            int128("l", [6i128]),
        ]);
        assert_eq!(t.num_columns(), 6);
        assert_eq!(t.num_rows(), 1);
    }
}
