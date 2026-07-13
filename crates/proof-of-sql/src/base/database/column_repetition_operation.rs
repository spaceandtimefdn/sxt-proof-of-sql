use super::{
    slice_operation::{repeat_elementwise, repeat_slice},
    Column, ColumnType,
};
use crate::base::scalar::Scalar;
use bumpalo::Bump;
use core::iter::Iterator;

#[expect(clippy::too_many_lines)]
pub trait RepetitionOp {
    fn op<T: Clone>(column: &[T], n: usize) -> impl Iterator<Item = T>;

    /// Run a column repetition operation on a `Column`.
    fn column_op<'a, S>(column: &Column<'a, S>, alloc: &'a Bump, n: usize) -> Column<'a, S>
    where
        S: Scalar,
    {
        let len = n * column.len();
        match column.column_type() {
            ColumnType::Boolean => {
                let mut iter = Self::op(column.as_boolean().expect("Column types should match"), n);
                Column::Boolean(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Uint8 => {
                let mut iter = Self::op(column.as_uint8().expect("Column types should match"), n);
                Column::Uint8(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::TinyInt => {
                let mut iter = Self::op(column.as_tinyint().expect("Column types should match"), n);
                Column::TinyInt(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::SmallInt => {
                let mut iter =
                    Self::op(column.as_smallint().expect("Column types should match"), n);
                Column::SmallInt(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Int => {
                let mut iter = Self::op(column.as_int().expect("Column types should match"), n);
                Column::Int(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::BigInt => {
                let mut iter = Self::op(column.as_bigint().expect("Column types should match"), n);
                Column::BigInt(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Int128 => {
                let mut iter = Self::op(column.as_int128().expect("Column types should match"), n);
                Column::Int128(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Scalar => {
                let mut iter = Self::op(column.as_scalar().expect("Column types should match"), n);
                Column::Scalar(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Decimal75(precision, scale) => {
                let mut iter =
                    Self::op(column.as_decimal75().expect("Column types should match"), n);
                Column::Decimal75(
                    precision,
                    scale,
                    alloc.alloc_slice_fill_with(len, |_| {
                        iter.next().expect("Iterator should have enough elements")
                    }) as &[_],
                )
            }
            ColumnType::VarChar => {
                let (raw_result, raw_scalars) =
                    column.as_varchar().expect("Column types should match");

                // Create iterators for both the result and scalars
                let mut result_iter = Self::op(raw_result, n);
                let mut scalar_iter = Self::op(raw_scalars, n);

                Column::VarChar((
                    alloc.alloc_slice_fill_with(len, |_| {
                        result_iter
                            .next()
                            .expect("Iterator should have enough elements")
                    }) as &[_],
                    alloc.alloc_slice_fill_with(len, |_| {
                        scalar_iter
                            .next()
                            .expect("Iterator should have enough elements")
                    }) as &[_],
                ))
            }
            ColumnType::VarBinary => {
                let (raw_result, raw_scalars) =
                    column.as_varbinary().expect("Column types should match");

                // Create iterators for both the result and scalars
                let mut result_iter = Self::op(raw_result, n);
                let mut scalar_iter = Self::op(raw_scalars, n);

                Column::VarBinary((
                    alloc.alloc_slice_fill_with(len, |_| {
                        result_iter
                            .next()
                            .expect("Iterator should have enough elements")
                    }) as &[_],
                    alloc.alloc_slice_fill_with(len, |_| {
                        scalar_iter
                            .next()
                            .expect("Iterator should have enough elements")
                    }) as &[_],
                ))
            }
            ColumnType::TimestampTZ(tu, tz) => {
                let mut iter = Self::op(
                    column.as_timestamptz().expect("Column types should match"),
                    n,
                );
                Column::TimestampTZ(
                    tu,
                    tz,
                    alloc.alloc_slice_fill_with(len, |_| {
                        iter.next().expect("Iterator should have enough elements")
                    }) as &[_],
                )
            }
        }
    }
}

pub struct ColumnRepeatOp {}
impl RepetitionOp for ColumnRepeatOp {
    fn op<T: Clone>(column: &[T], n: usize) -> impl Iterator<Item = T> {
        repeat_slice(column, n)
    }
}

pub struct ElementwiseRepeatOp {}
impl RepetitionOp for ElementwiseRepeatOp {
    fn op<T: Clone>(column: &[T], n: usize) -> impl Iterator<Item = T> {
        repeat_elementwise(column, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn test_column_repetition_op() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Int(&[1, 2, 3]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_int().unwrap(), &[1, 2, 3, 1, 2, 3]);

        // Varchar
        let strings = vec!["a", "b", "c"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let doubled_strings = vec!["a", "b", "c", "a", "b", "c"];
        let doubled_scalars = doubled_strings
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::VarChar((&doubled_strings, &doubled_scalars))
        );
    }

    #[test]
    fn test_elementwise_repetition_op() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Int(&[1, 2, 3]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_int().unwrap(), &[1, 1, 2, 2, 3, 3]);

        // Varchar
        let strings = vec!["a", "b", "c"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let doubled_strings = vec!["a", "a", "b", "b", "c", "c"];
        let doubled_scalars = doubled_strings
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::VarChar((&doubled_strings, &doubled_scalars))
        );

        let column: Column<TestScalar> = Column::Boolean(&[false, true, false]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result.as_boolean().unwrap(),
            &[false, false, true, true, false, false]
        );

        let column: Column<TestScalar> = Column::Uint8(&[3u8, 5u8, 2u8]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_uint8().unwrap(), &[3u8, 3u8, 5u8, 5u8, 2u8, 2u8]);
    }

    #[test]
    fn test_column_repetition_op_varbinary() {
        let bump = Bump::new();

        let bytes = vec![b"foo".as_ref(), b"bar".as_ref()];
        let scalars = vec![TestScalar::from(1), TestScalar::from(2)];

        let column: Column<TestScalar> = Column::VarBinary((bytes.as_slice(), scalars.as_slice()));
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);

        let expected_bytes = vec![
            b"foo".as_ref(),
            b"bar".as_ref(),
            b"foo".as_ref(),
            b"bar".as_ref(),
        ];
        let expected_scalars: Vec<TestScalar> = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(1),
            TestScalar::from(2),
        ];
        let expected = Column::VarBinary((expected_bytes.as_slice(), expected_scalars.as_slice()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_elementwise_repetition_op_varbinary() {
        let bump = Bump::new();

        let bytes = vec![b"foo".as_ref(), b"bar".as_ref(), b"baz".as_ref()];
        let scalars: Vec<TestScalar> = bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();

        let column: Column<TestScalar> = Column::VarBinary((bytes.as_slice(), scalars.as_slice()));
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);

        let expected_bytes = vec![
            b"foo".as_ref(),
            b"foo".as_ref(),
            b"bar".as_ref(),
            b"bar".as_ref(),
            b"baz".as_ref(),
            b"baz".as_ref(),
        ];
        let expected_scalars: Vec<TestScalar> = expected_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();
        let expected = Column::VarBinary((expected_bytes.as_slice(), expected_scalars.as_slice()));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_column_repetition_op_integer_and_scalar_variants() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Boolean(&[true, false]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_boolean().unwrap(), &[true, false, true, false]);

        let column: Column<TestScalar> = Column::Uint8(&[3_u8, 5_u8]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 3);
        assert_eq!(result.as_uint8().unwrap(), &[3, 5, 3, 5, 3, 5]);

        let column: Column<TestScalar> = Column::TinyInt(&[-2_i8, 4_i8]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_tinyint().unwrap(), &[-2, 4, -2, 4]);

        let column: Column<TestScalar> = Column::SmallInt(&[-7_i16, 8_i16]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_smallint().unwrap(), &[-7, 8, -7, 8]);

        let column: Column<TestScalar> = Column::BigInt(&[-11_i64, 13_i64]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_bigint().unwrap(), &[-11, 13, -11, 13]);

        let column: Column<TestScalar> = Column::Int128(&[-17_i128, 19_i128]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_int128().unwrap(), &[-17, 19, -17, 19]);

        let scalars = [TestScalar::from(23_i64), TestScalar::from(-29_i64)];
        let column = Column::Scalar(&scalars);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result.as_scalar().unwrap(),
            &[
                TestScalar::from(23_i64),
                TestScalar::from(-29_i64),
                TestScalar::from(23_i64),
                TestScalar::from(-29_i64),
            ]
        );
    }

    #[test]
    fn test_elementwise_repetition_op_integer_and_scalar_variants() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::TinyInt(&[-2_i8, 4_i8]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_tinyint().unwrap(), &[-2, -2, 4, 4]);

        let column: Column<TestScalar> = Column::SmallInt(&[-7_i16, 8_i16]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_smallint().unwrap(), &[-7, -7, 8, 8]);

        let column: Column<TestScalar> = Column::BigInt(&[-11_i64, 13_i64]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_bigint().unwrap(), &[-11, -11, 13, 13]);

        let column: Column<TestScalar> = Column::Int128(&[-17_i128, 19_i128]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_int128().unwrap(), &[-17, -17, 19, 19]);

        let scalars = [TestScalar::from(23_i64), TestScalar::from(-29_i64)];
        let column = Column::Scalar(&scalars);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result.as_scalar().unwrap(),
            &[
                TestScalar::from(23_i64),
                TestScalar::from(23_i64),
                TestScalar::from(-29_i64),
                TestScalar::from(-29_i64),
            ]
        );
    }

    #[test]
    fn test_repetition_ops_with_zero_repetitions() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Int(&[1, 2, 3]);
        let repeated = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 0);
        assert_eq!(repeated, Column::Int(&[]));

        let precision = Precision::new(9).unwrap();
        let decimals = [TestScalar::from(101_i64), TestScalar::from(-202_i64)];
        let decimal_column = Column::Decimal75(precision, 2, &decimals);
        let repeated = ElementwiseRepeatOp::column_op(&decimal_column, &bump, 0);
        assert_eq!(repeated, Column::Decimal75(precision, 2, &[]));

        let strings = ["a", "b"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let varchar_column = Column::VarChar((&strings, scalars.as_slice()));
        let repeated = ColumnRepeatOp::column_op::<TestScalar>(&varchar_column, &bump, 0);
        assert_eq!(repeated, Column::VarChar((&[], &[])));

        let timestamps = [42_i64, 84_i64];
        let timestamp_column =
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps);
        let repeated = ElementwiseRepeatOp::column_op::<TestScalar>(&timestamp_column, &bump, 0);
        assert_eq!(
            repeated,
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[])
        );
    }

    #[test]
    fn test_column_repetition_op_decimal_and_timestamp() {
        let bump = Bump::new();

        let precision = Precision::new(12).unwrap();
        let decimals = vec![
            TestScalar::from(1100_i64),
            TestScalar::from(-2200_i64),
            TestScalar::from(3300_i64),
        ];
        let decimal_column = Column::Decimal75(precision, -2, &decimals);
        let repeated_decimals = ColumnRepeatOp::column_op(&decimal_column, &bump, 2);
        assert_eq!(
            repeated_decimals,
            Column::Decimal75(
                precision,
                -2,
                &[
                    TestScalar::from(1100_i64),
                    TestScalar::from(-2200_i64),
                    TestScalar::from(3300_i64),
                    TestScalar::from(1100_i64),
                    TestScalar::from(-2200_i64),
                    TestScalar::from(3300_i64),
                ]
            )
        );

        let timestamps = [1_700_000_001_i64, 1_700_000_002_i64];
        let timestamp_column = Column::TimestampTZ(
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
            &timestamps,
        );
        let repeated_timestamps =
            ColumnRepeatOp::column_op::<TestScalar>(&timestamp_column, &bump, 3);
        assert_eq!(
            repeated_timestamps,
            Column::TimestampTZ(
                PoSQLTimeUnit::Millisecond,
                PoSQLTimeZone::utc(),
                &[
                    1_700_000_001,
                    1_700_000_002,
                    1_700_000_001,
                    1_700_000_002,
                    1_700_000_001,
                    1_700_000_002,
                ]
            )
        );
    }

    #[test]
    fn test_elementwise_repetition_op_decimal_and_timestamp() {
        let bump = Bump::new();

        let precision = Precision::new(18).unwrap();
        let decimals = vec![TestScalar::from(7_i64), TestScalar::from(-9_i64)];
        let decimal_column = Column::Decimal75(precision, 4, &decimals);
        let repeated_decimals = ElementwiseRepeatOp::column_op(&decimal_column, &bump, 3);
        assert_eq!(
            repeated_decimals,
            Column::Decimal75(
                precision,
                4,
                &[
                    TestScalar::from(7_i64),
                    TestScalar::from(7_i64),
                    TestScalar::from(7_i64),
                    TestScalar::from(-9_i64),
                    TestScalar::from(-9_i64),
                    TestScalar::from(-9_i64),
                ]
            )
        );

        let timestamps = [42_i64, 84_i64, 126_i64];
        let timestamp_column =
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps);
        let repeated_timestamps =
            ElementwiseRepeatOp::column_op::<TestScalar>(&timestamp_column, &bump, 2);
        assert_eq!(
            repeated_timestamps,
            Column::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &[42, 42, 84, 84, 126, 126]
            )
        );
    }
}
