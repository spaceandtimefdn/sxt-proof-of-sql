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
    fn test_column_repetition_op_numeric_scalar_decimal_and_time() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::TinyInt(&[-1, 2, 3]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_tinyint().unwrap(), &[-1, 2, 3, -1, 2, 3]);

        let column: Column<TestScalar> = Column::SmallInt(&[-10, 20]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 3);
        assert_eq!(result.as_smallint().unwrap(), &[-10, 20, -10, 20, -10, 20]);

        let column: Column<TestScalar> = Column::BigInt(&[100, -200]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_bigint().unwrap(), &[100, -200, 100, -200]);

        let column: Column<TestScalar> = Column::Int128(&[1_000_000_000_000, -7]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result.as_int128().unwrap(),
            &[1_000_000_000_000, -7, 1_000_000_000_000, -7]
        );

        let scalars = [1, 2].iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::Scalar(&scalars);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let expected_scalars = [1, 2, 1, 2]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(result, Column::Scalar(&expected_scalars));

        let decimals = [10, -20].iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> =
            Column::Decimal75(Precision::new(6).unwrap(), 2, &decimals);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let expected_decimals = [10, -20, 10, -20]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::Decimal75(Precision::new(6).unwrap(), 2, &expected_decimals)
        );

        let column: Column<TestScalar> = Column::TimestampTZ(
            PoSQLTimeUnit::Microsecond,
            PoSQLTimeZone::new(-5),
            &[10, 20],
        );
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result,
            Column::TimestampTZ(
                PoSQLTimeUnit::Microsecond,
                PoSQLTimeZone::new(-5),
                &[10, 20, 10, 20]
            )
        );
    }

    #[test]
    fn test_elementwise_repetition_op_numeric_scalar_decimal_and_time() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::TinyInt(&[-1, 2, 3]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_tinyint().unwrap(), &[-1, -1, 2, 2, 3, 3]);

        let column: Column<TestScalar> = Column::SmallInt(&[-10, 20]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 3);
        assert_eq!(result.as_smallint().unwrap(), &[-10, -10, -10, 20, 20, 20]);

        let column: Column<TestScalar> = Column::BigInt(&[100, -200]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_bigint().unwrap(), &[100, 100, -200, -200]);

        let column: Column<TestScalar> = Column::Int128(&[1_000_000_000_000, -7]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result.as_int128().unwrap(),
            &[1_000_000_000_000, 1_000_000_000_000, -7, -7]
        );

        let scalars = [1, 2].iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::Scalar(&scalars);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let expected_scalars = [1, 1, 2, 2]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(result, Column::Scalar(&expected_scalars));

        let decimals = [10, -20].iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> =
            Column::Decimal75(Precision::new(6).unwrap(), 2, &decimals);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let expected_decimals = [10, 10, -20, -20]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::Decimal75(Precision::new(6).unwrap(), 2, &expected_decimals)
        );

        let column: Column<TestScalar> =
            Column::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc(), &[10, 20]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(
            result,
            Column::TimestampTZ(
                PoSQLTimeUnit::Nanosecond,
                PoSQLTimeZone::utc(),
                &[10, 10, 20, 20]
            )
        );
    }

    #[test]
    fn test_column_repetition_op_with_zero_repetitions_returns_empty_column() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Int(&[1, 2, 3]);

        let column_repeated = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 0);
        let elementwise_repeated = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 0);

        assert!(column_repeated.as_int().unwrap().is_empty());
        assert!(elementwise_repeated.as_int().unwrap().is_empty());
    }
}
