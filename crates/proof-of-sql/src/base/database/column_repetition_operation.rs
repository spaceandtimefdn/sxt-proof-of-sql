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

    fn assert_repetition_ops(
        column: Column<'_, TestScalar>,
        column_repeat: Column<'_, TestScalar>,
        elementwise_repeat: Column<'_, TestScalar>,
    ) {
        let bump = Bump::new();
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result, column_repeat);

        let bump = Bump::new();
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result, elementwise_repeat);
    }

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
    fn test_repetition_ops_cover_remaining_column_variants() {
        assert_repetition_ops(
            Column::TinyInt(&[-2, 4]),
            Column::TinyInt(&[-2, 4, -2, 4]),
            Column::TinyInt(&[-2, -2, 4, 4]),
        );
        assert_repetition_ops(
            Column::SmallInt(&[-20, 40]),
            Column::SmallInt(&[-20, 40, -20, 40]),
            Column::SmallInt(&[-20, -20, 40, 40]),
        );
        assert_repetition_ops(
            Column::BigInt(&[-2_000, 4_000]),
            Column::BigInt(&[-2_000, 4_000, -2_000, 4_000]),
            Column::BigInt(&[-2_000, -2_000, 4_000, 4_000]),
        );
        assert_repetition_ops(
            Column::Int128(&[-20_000_i128, 40_000]),
            Column::Int128(&[-20_000_i128, 40_000, -20_000, 40_000]),
            Column::Int128(&[-20_000_i128, -20_000, 40_000, 40_000]),
        );

        let scalars = [TestScalar::from(7), TestScalar::from(11)];
        let scalar_column_repeat = [scalars[0], scalars[1], scalars[0], scalars[1]];
        let scalar_elementwise_repeat = [scalars[0], scalars[0], scalars[1], scalars[1]];
        assert_repetition_ops(
            Column::Scalar(&scalars),
            Column::Scalar(&scalar_column_repeat),
            Column::Scalar(&scalar_elementwise_repeat),
        );

        let precision = Precision::new(75).unwrap();
        let decimals = [TestScalar::from(123), TestScalar::from(456)];
        let decimal_column_repeat = [decimals[0], decimals[1], decimals[0], decimals[1]];
        let decimal_elementwise_repeat = [decimals[0], decimals[0], decimals[1], decimals[1]];
        assert_repetition_ops(
            Column::Decimal75(precision, -2, &decimals),
            Column::Decimal75(precision, -2, &decimal_column_repeat),
            Column::Decimal75(precision, -2, &decimal_elementwise_repeat),
        );

        let timezone = PoSQLTimeZone::utc();
        assert_repetition_ops(
            Column::TimestampTZ(PoSQLTimeUnit::Millisecond, timezone, &[1_000, 2_000]),
            Column::TimestampTZ(
                PoSQLTimeUnit::Millisecond,
                timezone,
                &[1_000, 2_000, 1_000, 2_000],
            ),
            Column::TimestampTZ(
                PoSQLTimeUnit::Millisecond,
                timezone,
                &[1_000, 1_000, 2_000, 2_000],
            ),
        );
    }
}
