use crate::base::{
    database::{Column, ColumnType, LiteralValue},
    scalar::Scalar,
};
use bumpalo::Bump;
use snafu::Snafu;

/// The result of evaluating an expression.
///
/// Inspired by [`datafusion_expr_common::ColumnarValue`]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ColumnarValue<'a, S: Scalar> {
    /// A [ `ColumnarValue::Column` ] is a list of values.
    Column(Column<'a, S>),
    /// A [ `ColumnarValue::Literal` ] is a single value with indeterminate size.
    Literal(LiteralValue),
}

/// Errors from operations on [`ColumnarValue`]s.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum ColumnarValueError {
    /// Attempt to convert a `[ColumnarValue::Column]` to a column of a different length
    ColumnLengthMismatch {
        /// The length of the `[ColumnarValue::Column]`
        columnar_value_length: usize,
        /// The length we attempted to convert the `[ColumnarValue::Column]` to
        attempt_to_convert_length: usize,
    },
}

impl<'a, S: Scalar> ColumnarValue<'a, S> {
    /// Provides the column type associated with the column
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Column(column) => column.column_type(),
            Self::Literal(literal) => literal.column_type(),
        }
    }

    /// Converts the [`ColumnarValue`] to a [`Column`]
    pub fn into_column(
        &self,
        num_rows: usize,
        alloc: &'a Bump,
    ) -> Result<Column<'a, S>, ColumnarValueError> {
        match self {
            Self::Column(column) => {
                if column.len() == num_rows {
                    Ok(*column)
                } else {
                    Err(ColumnarValueError::ColumnLengthMismatch {
                        columnar_value_length: column.len(),
                        attempt_to_convert_length: num_rows,
                    })
                }
            }
            Self::Literal(literal) => {
                Ok(Column::from_literal_with_length(literal, num_rows, alloc))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
    };
    use core::convert::Into;

    #[test]
    fn we_can_get_column_type_of_columnar_values() {
        let column = ColumnarValue::Column(Column::<TestScalar>::Int(&[1, 2, 3]));
        assert_eq!(column.column_type(), ColumnType::Int);

        let column = ColumnarValue::<TestScalar>::Literal(LiteralValue::Boolean(true));
        assert_eq!(column.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_can_transform_columnar_values_into_columns() {
        let bump = Bump::new();

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::Int(&[1, 2, 3]));
        let column = columnar_value.into_column(3, &bump).unwrap();
        assert_eq!(column, Column::Int(&[1, 2, 3]));

        let columnar_value = ColumnarValue::<TestScalar>::Literal(LiteralValue::Boolean(false));
        let column = columnar_value.into_column(5, &bump).unwrap();
        assert_eq!(column, Column::Boolean(&[false; 5]));

        // Check whether it works if `num_rows` is 0
        let columnar_value = ColumnarValue::<TestScalar>::Literal(LiteralValue::TinyInt(2));
        let column = columnar_value.into_column(0, &bump).unwrap();
        assert_eq!(column, Column::TinyInt(&[]));

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::SmallInt(&[]));
        let column = columnar_value.into_column(0, &bump).unwrap();
        assert_eq!(column, Column::SmallInt(&[]));
    }

    #[test]
    fn we_can_transform_literal_columnar_values_into_typed_columns() {
        let bump = Bump::new();

        let cases = [
            (
                LiteralValue::Uint8(7),
                Column::<TestScalar>::Uint8(&[7, 7, 7]),
            ),
            (LiteralValue::SmallInt(-8), Column::SmallInt(&[-8, -8, -8])),
            (LiteralValue::Int(9), Column::Int(&[9, 9, 9])),
            (LiteralValue::BigInt(-10), Column::BigInt(&[-10, -10, -10])),
            (
                LiteralValue::Int128(11),
                Column::Int128(&[11_i128, 11_i128, 11_i128]),
            ),
            (
                LiteralValue::TimeStampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), 12),
                Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[12, 12, 12]),
            ),
        ];

        for (literal, expected) in cases {
            let columnar_value = ColumnarValue::Literal(literal);
            let column = columnar_value.into_column(3, &bump).unwrap();
            assert_eq!(column, expected);
        }
    }

    #[test]
    fn we_can_transform_scalar_backed_literals_into_columns() {
        let bump = Bump::new();

        let scalar: TestScalar = [1_u64, 2, 3, 4].into();
        let columnar_value = ColumnarValue::Literal(LiteralValue::Scalar([1_u64, 2, 3, 4]));
        let column = columnar_value.into_column(2, &bump).unwrap();
        assert_eq!(column, Column::Scalar(&[scalar, scalar]));

        let decimal_scalar = TestScalar::from(7010_i32);
        let columnar_value = ColumnarValue::Literal(LiteralValue::Decimal75(
            Precision::new(9).unwrap(),
            2,
            7010.into(),
        ));
        let column = columnar_value.into_column(2, &bump).unwrap();
        assert_eq!(
            column,
            Column::Decimal75(
                Precision::new(9).unwrap(),
                2,
                &[decimal_scalar, decimal_scalar]
            )
        );
    }

    #[test]
    fn we_can_transform_variable_length_literals_into_columns() {
        let bump = Bump::new();

        let text = "alpha".to_string();
        let text_scalar = TestScalar::from(&text);
        let columnar_value = ColumnarValue::Literal(LiteralValue::VarChar(text));
        let column = columnar_value.into_column(2, &bump).unwrap();
        assert_eq!(
            column,
            Column::VarChar((&["alpha", "alpha"], &[text_scalar, text_scalar]))
        );

        let bytes = vec![1_u8, 2, 3, 4];
        let bytes_scalar = TestScalar::from_byte_slice_via_hash(&bytes);
        let columnar_value = ColumnarValue::Literal(LiteralValue::VarBinary(bytes));
        let column = columnar_value.into_column(2, &bump).unwrap();
        assert_eq!(
            column,
            Column::VarBinary((
                &[&[1_u8, 2, 3, 4][..], &[1_u8, 2, 3, 4][..]],
                &[bytes_scalar, bytes_scalar]
            ))
        );
    }

    #[test]
    fn we_cannot_transform_columnar_values_into_columns_of_different_length() {
        let bump = Bump::new();

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::Int(&[1, 2, 3]));
        let res = columnar_value.into_column(2, &bump);
        assert_eq!(
            res,
            Err(ColumnarValueError::ColumnLengthMismatch {
                columnar_value_length: 3,
                attempt_to_convert_length: 2,
            })
        );

        let strings = ["a", "b", "c"];
        let scalars: Vec<TestScalar> = strings.iter().map(Into::into).collect();
        let columnar_value =
            ColumnarValue::Column(Column::<TestScalar>::VarChar((&strings, &scalars)));
        let res = columnar_value.into_column(0, &bump);
        assert_eq!(
            res,
            Err(ColumnarValueError::ColumnLengthMismatch {
                columnar_value_length: 3,
                attempt_to_convert_length: 0,
            })
        );

        let columnar_value = ColumnarValue::Column(Column::<TestScalar>::Boolean(&[]));
        let res = columnar_value.into_column(1, &bump);
        assert_eq!(
            res,
            Err(ColumnarValueError::ColumnLengthMismatch {
                columnar_value_length: 0,
                attempt_to_convert_length: 1,
            })
        );
    }
}
