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
        database::ColumnType, math::decimal::Precision, scalar::test_scalar::TestScalar,
    };
    use alloc::{string::ToString, vec};

    #[test]
    fn we_can_create_owned_columns_with_utility_constructors() {
        assert_eq!(
            uint8::<TestScalar>("uint8", [1_u8, 2]),
            (Ident::new("uint8"), OwnedColumn::Uint8(vec![1_u8, 2]))
        );
        assert_eq!(
            tinyint::<TestScalar>("tinyint", [-1_i8, 2]),
            (Ident::new("tinyint"), OwnedColumn::TinyInt(vec![-1_i8, 2]))
        );
        assert_eq!(
            smallint::<TestScalar>("smallint", [-3_i16, 4]),
            (
                Ident::new("smallint"),
                OwnedColumn::SmallInt(vec![-3_i16, 4])
            )
        );
        assert_eq!(
            int::<TestScalar>("int", [-5_i32, 6]),
            (Ident::new("int"), OwnedColumn::Int(vec![-5_i32, 6]))
        );
        assert_eq!(
            bigint::<TestScalar>("bigint", [-7_i64, 8]),
            (Ident::new("bigint"), OwnedColumn::BigInt(vec![-7_i64, 8]))
        );
        assert_eq!(
            boolean::<TestScalar>("boolean", [true, false]),
            (
                Ident::new("boolean"),
                OwnedColumn::Boolean(vec![true, false])
            )
        );
        assert_eq!(
            int128::<TestScalar>("int128", [-9_i128, 10]),
            (Ident::new("int128"), OwnedColumn::Int128(vec![-9_i128, 10]))
        );
        assert_eq!(
            scalar::<TestScalar>("scalar", [TestScalar::from(11), TestScalar::from(12)]),
            (
                Ident::new("scalar"),
                OwnedColumn::Scalar(vec![TestScalar::from(11), TestScalar::from(12)])
            )
        );
        assert_eq!(
            varchar::<TestScalar>("varchar", ["Space", "Time"]),
            (
                Ident::new("varchar"),
                OwnedColumn::VarChar(["Space", "Time"].iter().map(ToString::to_string).collect())
            )
        );
        assert_eq!(
            varbinary::<TestScalar>("varbinary", [vec![1_u8, 2], vec![3]]),
            (
                Ident::new("varbinary"),
                OwnedColumn::VarBinary(vec![vec![1_u8, 2], vec![3]])
            )
        );
        assert_eq!(
            decimal75::<TestScalar>("decimal", 9, -2, [13_i64, -14]),
            (
                Ident::new("decimal"),
                OwnedColumn::Decimal75(
                    Precision::new(9).unwrap(),
                    -2,
                    vec![TestScalar::from(13), TestScalar::from(-14)]
                )
            )
        );

        let timezone = PoSQLTimeZone::utc();
        assert_eq!(
            timestamptz::<TestScalar>(
                "created_at",
                PoSQLTimeUnit::Millisecond,
                timezone,
                [111_i64, 222],
            ),
            (
                Ident::new("created_at"),
                OwnedColumn::TimestampTZ(PoSQLTimeUnit::Millisecond, timezone, vec![111_i64, 222])
            )
        );
    }

    #[test]
    fn we_can_create_owned_tables_with_utility_constructor() {
        let table = owned_table::<TestScalar>([
            bigint("id", [1_i64, 2]),
            boolean("is_active", [true, false]),
            scalar("score", [TestScalar::from(7), TestScalar::from(8)]),
        ]);

        assert_eq!(table.num_columns(), 3);
        assert_eq!(table.num_rows(), 2);
        assert_eq!(table["id"].column_type(), ColumnType::BigInt);
        assert_eq!(table["is_active"].column_type(), ColumnType::Boolean);
        assert_eq!(table["score"].column_type(), ColumnType::Scalar);
    }
}
