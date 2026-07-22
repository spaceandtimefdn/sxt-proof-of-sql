use super::{
    AddOp, ArithmeticOp, ColumnOperationError, ColumnOperationResult, ComparisonOp, DivOp, EqualOp,
    GreaterThanOp, LessThanOp, MulOp, SubOp,
};
use crate::base::{
    database::{
        slice_operation::{slice_and, slice_not, slice_or},
        OwnedColumn,
    },
    scalar::Scalar,
};
use alloc::string::ToString;

impl<S: Scalar> OwnedColumn<S> {
    /// Element-wise NOT operation for a column
    pub fn element_wise_not(&self) -> ColumnOperationResult<Self> {
        match self {
            Self::Boolean(values) => Ok(Self::Boolean(slice_not(values))),
            _ => Err(ColumnOperationError::UnaryOperationInvalidColumnType {
                operator: "NOT".to_string(),
                operand_type: self.column_type(),
            }),
        }
    }

    /// Element-wise AND for two columns
    pub fn element_wise_and(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_and(lhs, rhs))),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "AND".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise OR for two columns
    pub fn element_wise_or(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        if self.len() != rhs.len() {
            return Err(ColumnOperationError::DifferentColumnLength {
                len_a: self.len(),
                len_b: rhs.len(),
            });
        }
        match (self, rhs) {
            (Self::Boolean(lhs), Self::Boolean(rhs)) => Ok(Self::Boolean(slice_or(lhs, rhs))),
            _ => Err(ColumnOperationError::BinaryOperationInvalidColumnType {
                operator: "OR".to_string(),
                left_type: self.column_type(),
                right_type: rhs.column_type(),
            }),
        }
    }

    /// Element-wise equality check for two columns
    pub fn element_wise_eq(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        EqualOp::owned_column_element_wise_comparison(self, rhs)
    }

    /// Element-wise less than or equal to check for two columns
    pub fn element_wise_lt(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        LessThanOp::owned_column_element_wise_comparison(self, rhs)
    }

    /// Element-wise greater than or equal to check for two columns
    pub fn element_wise_gt(&self, rhs: &Self) -> ColumnOperationResult<Self> {
        GreaterThanOp::owned_column_element_wise_comparison(self, rhs)
    }

    /// Element-wise addition for two columns
    pub fn element_wise_add(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        AddOp::owned_column_element_wise_arithmetic(self, rhs)
    }

    /// Element-wise subtraction for two columns
    pub fn element_wise_sub(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        SubOp::owned_column_element_wise_arithmetic(self, rhs)
    }

    /// Element-wise multiplication for two columns
    pub fn element_wise_mul(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        MulOp::owned_column_element_wise_arithmetic(self, rhs)
    }

    /// Element-wise division for two columns
    pub fn element_wise_div(&self, rhs: &OwnedColumn<S>) -> ColumnOperationResult<OwnedColumn<S>> {
        DivOp::owned_column_element_wise_arithmetic(self, rhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};
    use alloc::vec;

    fn assert_adds_to(
        lhs: OwnedColumn<TestScalar>,
        rhs: OwnedColumn<TestScalar>,
        expected: OwnedColumn<TestScalar>,
    ) {
        assert_eq!(lhs.element_wise_add(&rhs).unwrap(), expected);
    }

    fn assert_eqs_to(
        lhs: OwnedColumn<TestScalar>,
        rhs: OwnedColumn<TestScalar>,
        expected: Vec<bool>,
    ) {
        assert_eq!(
            lhs.element_wise_eq(&rhs).unwrap(),
            OwnedColumn::<TestScalar>::Boolean(expected)
        );
    }

    fn decimal_column(values: &[i64], precision: u8, scale: i8) -> OwnedColumn<TestScalar> {
        OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(precision).unwrap(),
            scale,
            values.iter().copied().map(TestScalar::from).collect(),
        )
    }

    #[test]
    fn we_cannot_do_binary_operation_on_columns_with_different_lengths() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false]);

        let result = lhs.element_wise_and(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_eq(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_gt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]);
        let result = lhs.element_wise_add(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]);
        let result = lhs.element_wise_add(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_sub(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_mul(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));

        let result = lhs.element_wise_div(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::DifferentColumnLength { .. })
        ));
    }

    #[test]
    fn we_cannot_do_logical_operation_on_nonboolean_columns() {
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_and(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_or(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_not();
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let result = lhs.element_wise_and(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_or(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_not();
        assert!(matches!(
            result,
            Err(ColumnOperationError::UnaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_do_logical_operation_on_boolean_columns() {
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
        let result = lhs.element_wise_and(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![
                true, false, false, false
            ]))
        );

        let result = lhs.element_wise_or(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![
                true, true, true, false
            ]))
        );

        let result = lhs.element_wise_not();
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![
                false, true, false, true
            ]))
        );
    }

    #[test]
    fn we_can_do_eq_operation() {
        // Integers
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        // Strings
        let lhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]))
        );

        // Booleans
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, -3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        // Decimals and integers
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, -2, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 1, lhs_scalars);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );

        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, -2, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 1, lhs_scalars);
        let result = lhs.element_wise_eq(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]))
        );
    }

    #[test]
    fn we_can_add_integer_columns_across_remaining_upcast_pairs() {
        let uint8 = || OwnedColumn::<TestScalar>::Uint8(vec![3_u8, 4]);
        let tinyint = || OwnedColumn::<TestScalar>::TinyInt(vec![-1_i8, 2]);
        let smallint = || OwnedColumn::<TestScalar>::SmallInt(vec![-1_i16, 2]);
        let int = || OwnedColumn::<TestScalar>::Int(vec![-1_i32, 2]);
        let bigint = || OwnedColumn::<TestScalar>::BigInt(vec![-1_i64, 2]);
        let int128 = || OwnedColumn::<TestScalar>::Int128(vec![-1_i128, 2]);

        assert_adds_to(
            uint8(),
            smallint(),
            OwnedColumn::<TestScalar>::SmallInt(vec![2_i16, 6]),
        );
        assert_adds_to(
            uint8(),
            int(),
            OwnedColumn::<TestScalar>::Int(vec![2_i32, 6]),
        );
        assert_adds_to(
            uint8(),
            bigint(),
            OwnedColumn::<TestScalar>::BigInt(vec![2_i64, 6]),
        );
        assert_adds_to(
            uint8(),
            int128(),
            OwnedColumn::<TestScalar>::Int128(vec![2_i128, 6]),
        );
        assert_adds_to(
            tinyint(),
            smallint(),
            OwnedColumn::<TestScalar>::SmallInt(vec![-2_i16, 4]),
        );
        assert_adds_to(
            tinyint(),
            int128(),
            OwnedColumn::<TestScalar>::Int128(vec![-2_i128, 4]),
        );
        assert_adds_to(
            smallint(),
            tinyint(),
            OwnedColumn::<TestScalar>::SmallInt(vec![-2_i16, 4]),
        );
        assert_adds_to(
            smallint(),
            int(),
            OwnedColumn::<TestScalar>::Int(vec![-2_i32, 4]),
        );
        assert_adds_to(
            smallint(),
            bigint(),
            OwnedColumn::<TestScalar>::BigInt(vec![-2_i64, 4]),
        );
        assert_adds_to(
            smallint(),
            int128(),
            OwnedColumn::<TestScalar>::Int128(vec![-2_i128, 4]),
        );
        assert_adds_to(
            int(),
            tinyint(),
            OwnedColumn::<TestScalar>::Int(vec![-2_i32, 4]),
        );
        assert_adds_to(
            int(),
            smallint(),
            OwnedColumn::<TestScalar>::Int(vec![-2_i32, 4]),
        );
        assert_adds_to(
            bigint(),
            tinyint(),
            OwnedColumn::<TestScalar>::BigInt(vec![-2_i64, 4]),
        );
        assert_adds_to(
            bigint(),
            smallint(),
            OwnedColumn::<TestScalar>::BigInt(vec![-2_i64, 4]),
        );
        assert_adds_to(
            bigint(),
            int(),
            OwnedColumn::<TestScalar>::BigInt(vec![-2_i64, 4]),
        );
        assert_adds_to(
            int128(),
            tinyint(),
            OwnedColumn::<TestScalar>::Int128(vec![-2_i128, 4]),
        );
        assert_adds_to(
            int128(),
            smallint(),
            OwnedColumn::<TestScalar>::Int128(vec![-2_i128, 4]),
        );
        assert_adds_to(
            int128(),
            bigint(),
            OwnedColumn::<TestScalar>::Int128(vec![-2_i128, 4]),
        );
    }

    #[test]
    fn we_reject_uint8_tinyint_arithmetic_and_comparison() {
        let uint8 = OwnedColumn::<TestScalar>::Uint8(vec![1_u8, 2]);
        let tinyint = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, -2]);

        assert!(matches!(
            uint8.element_wise_add(&tinyint),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
        assert!(matches!(
            tinyint.element_wise_add(&uint8),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
        assert!(matches!(
            uint8.element_wise_eq(&tinyint),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
        assert!(matches!(
            tinyint.element_wise_eq(&uint8),
            Err(ColumnOperationError::SignedCastingError { .. })
        ));
    }

    #[test]
    fn we_can_compare_integer_columns_across_remaining_upcast_pairs() {
        let uint8 = || OwnedColumn::<TestScalar>::Uint8(vec![1_u8, 2, 3]);
        let tinyint = || OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, -2, 4]);
        let smallint = || OwnedColumn::<TestScalar>::SmallInt(vec![1_i16, -2, 4]);
        let int = || OwnedColumn::<TestScalar>::Int(vec![1_i32, -2, 4]);
        let bigint = || OwnedColumn::<TestScalar>::BigInt(vec![1_i64, -2, 4]);
        let int128 = || OwnedColumn::<TestScalar>::Int128(vec![1_i128, -2, 4]);

        assert_eqs_to(uint8(), smallint(), vec![true, false, false]);
        assert_eqs_to(uint8(), int(), vec![true, false, false]);
        assert_eqs_to(uint8(), bigint(), vec![true, false, false]);
        assert_eqs_to(uint8(), int128(), vec![true, false, false]);
        assert_eqs_to(tinyint(), int128(), vec![true, true, true]);
        assert_eqs_to(smallint(), tinyint(), vec![true, true, true]);
        assert_eqs_to(smallint(), bigint(), vec![true, true, true]);
        assert_eqs_to(smallint(), int128(), vec![true, true, true]);
        assert_eqs_to(int(), tinyint(), vec![true, true, true]);
        assert_eqs_to(int(), bigint(), vec![true, true, true]);
        assert_eqs_to(int(), int128(), vec![true, true, true]);
        assert_eqs_to(bigint(), tinyint(), vec![true, true, true]);
        assert_eqs_to(bigint(), smallint(), vec![true, true, true]);
        assert_eqs_to(bigint(), int(), vec![true, true, true]);
        assert_eqs_to(int128(), tinyint(), vec![true, true, true]);
        assert_eqs_to(int128(), smallint(), vec![true, true, true]);
        assert_eqs_to(int128(), int(), vec![true, true, true]);
        assert_eqs_to(int128(), bigint(), vec![true, true, true]);
    }

    #[test]
    fn we_can_operate_on_decimal_columns_with_integer_columns_in_both_orders() {
        let decimal = || decimal_column(&[10, -20, 35], 5, 1);
        let tinyint = || OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, -3]);
        let smallint = || OwnedColumn::<TestScalar>::SmallInt(vec![1_i16, 2, -3]);
        let int = || OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, -3]);
        let bigint = || OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, -3]);
        let int128 = || OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, -3]);
        let expected_values = vec![
            TestScalar::from(20),
            TestScalar::from(0),
            TestScalar::from(5),
        ];

        for rhs in [tinyint(), smallint(), int(), bigint(), int128()] {
            match decimal().element_wise_add(&rhs).unwrap() {
                OwnedColumn::Decimal75(_, scale, values) => {
                    assert_eq!(scale, 1);
                    assert_eq!(values, expected_values);
                }
                other => panic!("expected decimal result, got {other:?}"),
            }
        }

        let decimal_rhs = || decimal_column(&[10, -20, 35], 5, 1);
        assert_eqs_to(tinyint(), decimal_rhs(), vec![true, false, false]);
        assert_eqs_to(smallint(), decimal_rhs(), vec![true, false, false]);
        assert_eqs_to(int(), decimal_rhs(), vec![true, false, false]);
        assert_eqs_to(bigint(), decimal_rhs(), vec![true, false, false]);
        assert_eqs_to(int128(), decimal_rhs(), vec![true, false, false]);
        assert_eqs_to(decimal_rhs(), tinyint(), vec![true, false, false]);
        assert_eqs_to(decimal_rhs(), smallint(), vec![true, false, false]);
        assert_eqs_to(decimal_rhs(), int(), vec![true, false, false]);
        assert_eqs_to(decimal_rhs(), bigint(), vec![true, false, false]);
        assert_eqs_to(decimal_rhs(), int128(), vec![true, false, false]);
    }

    #[test]
    fn we_can_do_lt_operation_on_numeric_and_boolean_columns() {
        // Booleans
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]);
        let result = lhs.element_wise_lt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, true, false]))
        );

        // Integers
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_lt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let result = lhs.element_wise_lt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 24, -3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_lt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, true, false]))
        );

        // Decimals and integers
        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_lt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]))
        );

        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_lt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]))
        );
    }

    #[test]
    fn we_can_do_ge_operation_on_numeric_and_boolean_columns() {
        // Booleans
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false]);
        let result = lhs.element_wise_gt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]))
        );

        // Integers
        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_gt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, true, false]))
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 3, 2]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3]);
        let result = lhs.element_wise_gt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, true, false]))
        );

        // Decimals
        let lhs_scalars = [10, 2, 30].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 24, -3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 3, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_gt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![false, false, true]))
        );

        // Decimals and integers
        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_gt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );

        let lhs_scalars = [10, -2, -30].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, -20, 3]);
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), -1, lhs_scalars);
        let result = lhs.element_wise_gt(&rhs);
        assert_eq!(
            result,
            Ok(OwnedColumn::<TestScalar>::Boolean(vec![true, false, false]))
        );
    }

    #[test]
    fn we_cannot_do_comparison_on_columns_with_incompatible_types() {
        // Strings can't be compared with other types
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_gt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Booleans can't be compared with other types
        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        // Strings can not be <= or >= to each other
        let lhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let result = lhs.element_wise_lt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_gt(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_cannot_do_arithmetic_on_nonnumeric_columns() {
        let lhs = OwnedColumn::<TestScalar>::VarChar(
            ["Space", "and", "Time"]
                .iter()
                .map(ToString::to_string)
                .collect(),
        );
        let rhs = OwnedColumn::<TestScalar>::Scalar(vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ]);
        let result = lhs.element_wise_add(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_sub(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_mul(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));

        let result = lhs.element_wise_div(&rhs);
        assert!(matches!(
            result,
            Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
        ));
    }

    #[test]
    fn we_can_add_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs.element_wise_add(&rhs).unwrap();
        assert_eq!(result, OwnedColumn::<TestScalar>::TinyInt(vec![2_i8, 4, 6]));

        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1_i16, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1_i16, 2, 3]);
        let result = lhs.element_wise_add(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::SmallInt(vec![2_i16, 4, 6])
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs.element_wise_add(&rhs).unwrap();
        assert_eq!(result, OwnedColumn::<TestScalar>::Int(vec![2_i32, 4, 6]));

        let lhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs.element_wise_add(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Int128(vec![2_i128, 4, 6])
        );
    }

    #[test]
    fn we_can_add_decimal_columns() {
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_add(&rhs).unwrap();
        let expected_scalars = [2, 4, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(51).unwrap(), 3, rhs_scalars);
        let result = lhs.element_wise_add(&rhs).unwrap();
        let expected_scalars = [11, 22, 33].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(52).unwrap(), 3, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_add(&rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_add(&rhs).unwrap();
        let expected_scalars = [101, 202, 303].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 2, expected_scalars)
        );
    }

    #[test]
    fn we_can_subtract_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, 2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 3, -1])
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![4_i32, 5, 2]);
        let rhs = OwnedColumn::<TestScalar>::Int(vec![1_i32, 2, 3]);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        assert_eq!(result, OwnedColumn::<TestScalar>::Int(vec![3_i32, 3, -1]));

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, 2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 5]);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::BigInt(vec![3_i64, 3, -3])
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 5]);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::BigInt(vec![2_i64, 0, -2])
        );
    }

    #[test]
    fn we_can_subtract_decimal_columns() {
        // lhs and rhs have the same precision and scale
        let lhs_scalars = [4, 5, 2].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        let expected_scalars = [3, 3, -1].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        // lhs and rhs have different precisions and scales
        let lhs_scalars = [4, 5, 2].iter().map(TestScalar::from).collect();
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(25).unwrap(), 2, lhs_scalars);
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(51).unwrap(), 3, rhs_scalars);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        let expected_scalars = [39, 48, 17].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(52).unwrap(), 3, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(6).unwrap(), 2, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_sub(&rhs).unwrap();
        let expected_scalars = [399, 498, 197].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 2, expected_scalars)
        );
    }

    #[test]
    fn we_can_multiply_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 10, -6])
        );

        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 3]);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 10, -6])
        );

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Int128(vec![3_i128, 4, 15])
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Int128(vec![3_i128, 4, 15])
        );
    }

    #[test]
    fn we_can_multiply_decimal_columns() {
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 2].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs_scalars = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        let expected_scalars = [-4, 10, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(11).unwrap(), 4, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(9).unwrap(), 2, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![4, 5, 2]);
        let rhs_scalars = [1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_mul(&rhs).unwrap();
        let expected_scalars = [4, 10, 6].iter().map(TestScalar::from).collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(16).unwrap(), 2, expected_scalars)
        );
    }

    #[test]
    fn we_can_divide_integer_columns() {
        // lhs and rhs have the same precision
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1_i8, 2, 3]);
        let result = lhs.element_wise_div(&rhs).unwrap();
        assert_eq!(result, OwnedColumn::<TestScalar>::TinyInt(vec![4_i8, 2, 0]));

        let lhs = OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 5, -2]);
        let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1_i64, 2, 3]);
        let result = lhs.element_wise_div(&rhs).unwrap();
        assert_eq!(result, OwnedColumn::<TestScalar>::BigInt(vec![4_i64, 2, 0]));

        // lhs and rhs have different precisions
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![3_i8, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs.element_wise_div(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Int128(vec![3_i128, 1, 0])
        );

        let lhs = OwnedColumn::<TestScalar>::Int(vec![3_i32, 2, 3]);
        let rhs = OwnedColumn::<TestScalar>::Int128(vec![1_i128, 2, 5]);
        let result = lhs.element_wise_div(&rhs).unwrap();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Int128(vec![3_i128, 1, 0])
        );
    }

    #[test]
    fn we_can_try_divide_decimal_columns() {
        // lhs and rhs are both decimals
        let lhs_scalars = [4, 5, 3].iter().map(TestScalar::from).collect();
        let lhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, lhs_scalars);
        let rhs_scalars = [-1, 2, 4].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(5).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_div(&rhs).unwrap();
        let expected_scalars = [-400_000_000_i128, 250_000_000, 75_000_000]
            .iter()
            .map(TestScalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 8, expected_scalars)
        );

        // lhs is integer and rhs is decimal
        let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![4, 5, 3]);
        let rhs_scalars = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(3).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_div(&rhs).unwrap();
        let expected_scalars = [-400_000_000, 250_000_000, 100_000_000]
            .iter()
            .map(TestScalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(11).unwrap(), 6, expected_scalars)
        );

        let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![4, 5, 3]);
        let rhs_scalars = [-1, 2, 3].iter().map(TestScalar::from).collect();
        let rhs = OwnedColumn::<TestScalar>::Decimal75(Precision::new(3).unwrap(), 2, rhs_scalars);
        let result = lhs.element_wise_div(&rhs).unwrap();
        let expected_scalars = [-400_000_000, 250_000_000, 100_000_000]
            .iter()
            .map(TestScalar::from)
            .collect();
        assert_eq!(
            result,
            OwnedColumn::<TestScalar>::Decimal75(Precision::new(13).unwrap(), 6, expected_scalars)
        );
    }
}
