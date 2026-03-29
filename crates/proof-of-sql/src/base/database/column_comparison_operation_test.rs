use crate::base::{
    database::{ColumnOperationError, ColumnType, OwnedColumn},
    scalar::test_scalar::TestScalar,
};

// ── EqualOp ──────────────────────────────────────────────────────────────────

#[test]
fn we_can_eq_boolean_columns() {
    let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, false]);
    let rhs = OwnedColumn::<TestScalar>::Boolean(vec![true, true, false, false]);
    let result = lhs.element_wise_eq(&rhs).unwrap();
    assert_eq!(
        result,
        OwnedColumn::Boolean(vec![true, false, false, true])
    );
}

#[test]
fn we_can_eq_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![1, 3, 3]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );
}

#[test]
fn we_can_eq_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![-1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![-1, 3, 3]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );
}

#[test]
fn we_can_eq_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![100, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![100, 300, 300]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );
}

#[test]
fn we_can_eq_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![1000, 2000, 3000]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![1000, 3000, 3000]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );
}

#[test]
fn we_can_eq_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 4]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, true, false])
    );
}

#[test]
fn we_can_eq_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![1, 2, 4]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, true, false])
    );
}

#[test]
fn we_can_eq_varchar_columns() {
    let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".into(), "b".into(), "c".into()]);
    let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".into(), "x".into(), "c".into()]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );
}

#[test]
fn we_can_eq_mixed_integer_columns() {
    // Uint8 == BigInt (left upcast)
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1i64, 3, 3]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );

    // TinyInt == SmallInt (left upcast)
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1i16, 2, 4]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, true, false])
    );

    // SmallInt == Int (left upcast)
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![10i16, 20, 30]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![10i32, 99, 30]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );

    // BigInt == TinyInt (right upcast)
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![5i64, 6, 7]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![5i8, 7, 7]);
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, true])
    );
}

#[test]
fn we_cannot_eq_columns_with_different_lengths() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
    assert_eq!(
        lhs.element_wise_eq(&rhs),
        Err(ColumnOperationError::DifferentColumnLength {
            len_a: 3,
            len_b: 2,
        })
    );
}

#[test]
fn we_cannot_eq_incompatible_column_types() {
    let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
    assert!(matches!(
        lhs.element_wise_eq(&rhs),
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn we_cannot_eq_uint8_with_tinyint() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8]);
    assert_eq!(
        lhs.element_wise_eq(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Uint8,
            right_type: ColumnType::TinyInt,
        })
    );
}

// ── LessThanOp ───────────────────────────────────────────────────────────────

#[test]
fn we_can_lt_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2, 5, 1]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_lt_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![1i32, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![2i32, 5, 1]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_lt_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1i16, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![2i16, 5, 1]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_lt_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![-5i8, 0, 5]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![0i8, 0, 3]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_lt_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![2u8, 5, 1]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_lt_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![1, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![2, 5, 1]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_lt_mixed_integer_columns() {
    // SmallInt < BigInt
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![1i16, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2i64, 5, 1]);
    assert_eq!(
        lhs.element_wise_lt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_cannot_lt_varchar_columns() {
    let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".into()]);
    let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["b".into()]);
    assert!(matches!(
        lhs.element_wise_lt(&rhs),
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn we_cannot_lt_uint8_with_tinyint() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8]);
    assert_eq!(
        lhs.element_wise_lt(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Uint8,
            right_type: ColumnType::TinyInt,
        })
    );
}

// ── GreaterThanOp ─────────────────────────────────────────────────────────────

#[test]
fn we_can_gt_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![5, 1, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_gt_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![5i32, 1, 3]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![2i32, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_gt_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![5i16, 1, 3]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![2i16, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_gt_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![5i8, -1, 3]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![2i8, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_gt_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![5u8, 1, 3]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![2u8, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_gt_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![5, 1, 3]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![2, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_can_gt_mixed_integer_columns() {
    // BigInt > SmallInt (right upcast)
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![5i64, 1, 3]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![2i16, 5, 3]);
    assert_eq!(
        lhs.element_wise_gt(&rhs).unwrap(),
        OwnedColumn::Boolean(vec![true, false, false])
    );
}

#[test]
fn we_cannot_gt_varchar_columns() {
    let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["b".into()]);
    let rhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".into()]);
    assert!(matches!(
        lhs.element_wise_gt(&rhs),
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn we_cannot_gt_tinyint_with_uint8() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8]);
    assert_eq!(
        lhs.element_wise_gt(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::TinyInt,
            right_type: ColumnType::Uint8,
        })
    );
}

#[test]
fn we_cannot_gt_columns_with_different_lengths() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
    assert_eq!(
        lhs.element_wise_gt(&rhs),
        Err(ColumnOperationError::DifferentColumnLength {
            len_a: 2,
            len_b: 1,
        })
    );
}
