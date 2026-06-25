//! Utility functions for creating [`Table`]s and [`Column`]s.
//! These functions are primarily intended for use in tests.
//!
//! # Example
//! ```
//! use bumpalo::Bump;
//! use proof_of_sql::base::{database::table_utility::*};
//! # use proof_of_sql::base::scalar::MontScalar;
//! # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
//! let alloc = Bump::new();
//! let result = table::<MyScalar>([
//!     borrowed_bigint("a", [1, 2, 3], &alloc),
//!     borrowed_boolean("b", [true, false, true], &alloc),
//!     borrowed_int128("c", [1, 2, 3], &alloc),
//!     borrowed_scalar("d", [1, 2, 3], &alloc),
//!     borrowed_varchar("e", ["a", "b", "c"], &alloc),
//!     borrowed_decimal75("f", 12, 1, [1, 2, 3], &alloc),
//! ]);
//! ```
use super::{Column, Table, TableOptions};
use crate::base::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
};
use alloc::{string::String, vec::Vec};
use bumpalo::Bump;
use sqlparser::ast::Ident;

/// Creates an [`Table`] from a list of `(Ident, Column)` pairs.
/// This is a convenience wrapper around [`Table::try_from_iter`] primarily for use in tests and
/// intended to be used along with the other methods in this module (e.g. [`borrowed_bigint`],
/// [`borrowed_boolean`], etc).
/// The function will panic under a variety of conditions. See [`Table::try_from_iter`] for more details.
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_bigint("a", [1, 2, 3], &alloc),
///     borrowed_boolean("b", [true, false, true], &alloc),
///     borrowed_int128("c", [1, 2, 3], &alloc),
///     borrowed_scalar("d", [1, 2, 3], &alloc),
///     borrowed_varchar("e", ["a", "b", "c"], &alloc),
///     borrowed_decimal75("f", 12, 1, [1, 2, 3], &alloc),
/// ]);
/// ```
///
/// # Panics
/// - Panics if converting the iterator into an `Table<'a, S>` fails.
pub fn table<'a, S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, Column<'a, S>)>,
) -> Table<'a, S> {
    Table::try_from_iter(iter).unwrap()
}

/// Creates an [`Table`] from a list of `(Ident, Column)` pairs with a specified row count.
/// The main reason for this function is to allow for creating tables that may potentially have
/// no columns, but still have a specified row count.
///
/// # Panics
/// - Panics if the given row count doesn't match the number of rows in any of the columns.
pub fn table_with_row_count<'a, S: Scalar>(
    iter: impl IntoIterator<Item = (Ident, Column<'a, S>)>,
    row_count: usize,
) -> Table<'a, S> {
    Table::try_from_iter_with_options(iter, TableOptions::new(Some(row_count))).unwrap()
}

/// Creates a (Ident, `Column`) pair for a uint8 column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_uint8("a", [1_u8, 2, 3], &alloc),
/// ]);
///```
pub fn borrowed_uint8<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<u8>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<u8> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Uint8(alloc_data))
}

/// Creates a (Ident, `Column`) pair for a tinyint column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_tinyint("a", [1_i8, 2, 3], &alloc),
/// ]);
///```
pub fn borrowed_tinyint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i8>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i8> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::TinyInt(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a smallint column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_smallint("a", [1_i16, 2, 3], &alloc),
/// ]);
/// ```
///
pub fn borrowed_smallint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i16>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i16> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::SmallInt(alloc_data))
}

/// Creates a `(Ident, Column)` pair for an int column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_int("a", [1, 2, 3], &alloc),
/// ]);
/// ```
///
pub fn borrowed_int<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i32>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i32> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Int(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a bigint column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```rust
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_bigint("a", [1, 2, 3], &alloc),
/// ]);
/// ```
pub fn borrowed_bigint<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i64>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i64> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::BigInt(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a boolean column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_boolean("a", [true, false, true], &alloc),
/// ]);
/// ```
pub fn borrowed_boolean<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<bool>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<bool> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Boolean(alloc_data))
}

/// Creates a `(Ident, Column)` pair for an int128 column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_int128("a", [1, 2, 3], &alloc),
/// ]);
/// ```
pub fn borrowed_int128<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<i128>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<i128> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Int128(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a scalar column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_scalar("a", [1, 2, 3], &alloc),
/// ]);
/// ```
pub fn borrowed_scalar<S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<S>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<S> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (name.into(), Column::Scalar(alloc_data))
}

/// Creates a `(Ident, Column)` pair for a varchar column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_varchar("a", ["a", "b", "c"], &alloc),
/// ]);
/// ```
pub fn borrowed_varchar<'a, S: Scalar>(
    name: impl Into<Ident>,
    data: impl IntoIterator<Item = impl Into<String>>,
    alloc: &'a Bump,
) -> (Ident, Column<'a, S>) {
    let strings: Vec<&'a str> = data
        .into_iter()
        .map(|item| {
            let string = item.into();
            alloc.alloc_str(&string) as &'a str
        })
        .collect();
    let alloc_strings = alloc.alloc_slice_clone(&strings);
    let scalars: Vec<S> = strings.iter().map(|s| (*s).into()).collect();
    let alloc_scalars = alloc.alloc_slice_copy(&scalars);
    (name.into(), Column::VarChar((alloc_strings, alloc_scalars)))
}

/// Creates a `(Ident, Column)` pair for a decimal75 column.
/// This is primarily intended for use in conjunction with [`table`].
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_decimal75("a", 12, 1, [1, 2, 3], &alloc),
/// ]);
/// ```
/// # Panics
/// - Panics if creating the `Precision` from the specified precision value fails.
pub fn borrowed_decimal75<S: Scalar>(
    name: impl Into<Ident>,
    precision: u8,
    scale: i8,
    data: impl IntoIterator<Item = impl Into<S>>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let transformed_data: Vec<S> = data.into_iter().map(Into::into).collect();
    let alloc_data = alloc.alloc_slice_copy(&transformed_data);
    (
        name.into(),
        Column::Decimal75(
            crate::base::math::decimal::Precision::new(precision).unwrap(),
            scale,
            alloc_data,
        ),
    )
}

/// Creates a `(Ident, Column)` pair for a timestamp column.
/// This is primarily intended for use in conjunction with [`table`].
///
/// # Parameters
/// - `name`: The name of the column.
/// - `time_unit`: The time unit of the timestamps.
/// - `timezone`: The timezone for the timestamps.
/// - `data`: The data for the column, provided as an iterator over `i64` values representing time since the unix epoch.
/// - `alloc`: The bump allocator to use for allocating the column data.
///
/// # Example
/// ```
/// use bumpalo::Bump;
/// use proof_of_sql::base::{database::table_utility::*, posql_time::{PoSQLTimeZone, PoSQLTimeUnit}};
/// # use proof_of_sql::base::scalar::MontScalar;
/// # pub type MyScalar = MontScalar<ark_curve25519::FrConfig>;
///
/// let alloc = Bump::new();
/// let result = table::<MyScalar>([
///     borrowed_timestamptz("event_time", PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), vec![1625072400, 1625076000, 1625079600], &alloc),
/// ]);
/// ```
pub fn borrowed_timestamptz<S: Scalar>(
    name: impl Into<Ident>,
    time_unit: PoSQLTimeUnit,
    timezone: PoSQLTimeZone,
    data: impl IntoIterator<Item = i64>,
    alloc: &Bump,
) -> (Ident, Column<'_, S>) {
    let vec_data: Vec<i64> = data.into_iter().collect();
    let alloc_data = alloc.alloc_slice_copy(&vec_data);
    (
        name.into(),
        Column::TimestampTZ(time_unit, timezone, alloc_data),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::Column,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    #[test]
    fn borrowed_uint8_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_uint8::<TestScalar>("a", [1u8, 2, 3], &alloc);
        assert_eq!(name, Ident::new("a"));
        assert_eq!(col, Column::Uint8(&[1u8, 2, 3]));
    }

    #[test]
    fn borrowed_tinyint_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_tinyint::<TestScalar>("b", [-1i8, 0, 1], &alloc);
        assert_eq!(name, Ident::new("b"));
        assert_eq!(col, Column::TinyInt(&[-1i8, 0, 1]));
    }

    #[test]
    fn borrowed_smallint_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_smallint::<TestScalar>("c", [100i16, 200], &alloc);
        assert_eq!(name, Ident::new("c"));
        assert_eq!(col, Column::SmallInt(&[100i16, 200]));
    }

    #[test]
    fn borrowed_int_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_int::<TestScalar>("d", [1i32, 2, 3], &alloc);
        assert_eq!(name, Ident::new("d"));
        assert_eq!(col, Column::Int(&[1i32, 2, 3]));
    }

    #[test]
    fn borrowed_bigint_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_bigint::<TestScalar>("e", [10i64, 20, 30], &alloc);
        assert_eq!(name, Ident::new("e"));
        assert_eq!(col, Column::BigInt(&[10i64, 20, 30]));
    }

    #[test]
    fn borrowed_boolean_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_boolean::<TestScalar>("f", [true, false], &alloc);
        assert_eq!(name, Ident::new("f"));
        assert_eq!(col, Column::Boolean(&[true, false]));
    }

    #[test]
    fn borrowed_int128_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_int128::<TestScalar>("g", [1i128, -1], &alloc);
        assert_eq!(name, Ident::new("g"));
        assert_eq!(col, Column::Int128(&[1i128, -1]));
    }

    #[test]
    fn borrowed_scalar_creates_correct_column() {
        let alloc = Bump::new();
        let data = [TestScalar::from(5)];
        let (name, col) = borrowed_scalar::<TestScalar>("h", data, &alloc);
        assert_eq!(name, Ident::new("h"));
        assert_eq!(col, Column::Scalar(&[TestScalar::from(5)]));
    }

    #[test]
    fn borrowed_varchar_creates_correct_column_name() {
        let alloc = Bump::new();
        let (name, col) = borrowed_varchar::<TestScalar>("i", ["hello"], &alloc);
        assert_eq!(name, Ident::new("i"));
        assert!(matches!(col, Column::VarChar(_)));
    }

    #[test]
    fn borrowed_decimal75_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_decimal75::<TestScalar>("k", 10, 2, [TestScalar::from(1)], &alloc);
        assert_eq!(name, Ident::new("k"));
        assert!(matches!(col, Column::Decimal75(_, 2, _)));
    }

    #[test]
    fn borrowed_timestamptz_creates_correct_column() {
        let alloc = Bump::new();
        let (name, col) = borrowed_timestamptz::<TestScalar>(
            "l",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [1_000_000i64],
            &alloc,
        );
        assert_eq!(name, Ident::new("l"));
        assert!(matches!(col, Column::TimestampTZ(PoSQLTimeUnit::Second, _, _)));
    }

    #[test]
    fn table_creates_table_with_correct_column_count() {
        let alloc = Bump::new();
        let t = table::<TestScalar>([
            borrowed_bigint("x", [1i64, 2], &alloc),
            borrowed_boolean("y", [true, false], &alloc),
        ]);
        assert_eq!(t.num_columns(), 2);
    }

    #[test]
    fn table_with_row_count_zero_rows_is_valid() {
        let t = table_with_row_count::<TestScalar>([], 0);
        assert_eq!(t.num_rows(), 0);
    }

    #[test]
    fn table_with_row_count_sets_row_count() {
        let t = table_with_row_count::<TestScalar>([], 5);
        assert_eq!(t.num_rows(), 5);
    }
}
