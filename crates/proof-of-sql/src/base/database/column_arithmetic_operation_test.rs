use crate::base::{
    database::{ColumnOperationError, ColumnType, OwnedColumn},
    scalar::test_scalar::TestScalar,
};

// ── AddOp ────────────────────────────────────────────────────────────────────

#[test]
fn we_can_add_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![4u8, 5, 6]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::Uint8(vec![5, 7, 9])
    );
}

#[test]
fn we_can_add_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![-1i8, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![4i8, -5, 6]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::TinyInt(vec![3, -3, 9])
    );
}

#[test]
fn we_can_add_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![10i16, 20, 30]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![5i16, 5, 5]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::SmallInt(vec![15, 25, 35])
    );
}

#[test]
fn we_can_add_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![100i32, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![1i32, 2, 3]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::Int(vec![101, 202, 303])
    );
}

#[test]
fn we_can_add_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1i64, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![10i64, 20, 30]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![11, 22, 33])
    );
}

#[test]
fn we_can_add_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![1i128, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![10i128, 20, 30]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::Int128(vec![11, 22, 33])
    );
}

#[test]
fn we_can_add_mixed_integer_columns() {
    // Uint8 + SmallInt → SmallInt (left upcast)
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![10i16, 20, 30]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::SmallInt(vec![11, 22, 33])
    );

    // TinyInt + BigInt → BigInt (left upcast)
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8, -2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![100i64, 200, 300]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![101, 198, 303])
    );

    // SmallInt + Int → Int (left upcast)
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![5i16, 10, 15]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![100i32, 200, 300]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::Int(vec![105, 210, 315])
    );

    // BigInt + SmallInt → BigInt (right upcast)
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1000i64, 2000, 3000]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![1i16, 2, 3]);
    assert_eq!(
        lhs.element_wise_add(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![1001, 2002, 3003])
    );
}

#[test]
fn we_cannot_add_columns_with_different_lengths() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
    assert_eq!(
        lhs.element_wise_add(&rhs),
        Err(ColumnOperationError::DifferentColumnLength {
            len_a: 3,
            len_b: 2,
        })
    );
}

#[test]
fn we_cannot_add_uint8_with_tinyint() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8]);
    assert_eq!(
        lhs.element_wise_add(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Uint8,
            right_type: ColumnType::TinyInt,
        })
    );
}

#[test]
fn we_cannot_add_incompatible_column_types() {
    let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
    assert!(matches!(
        lhs.element_wise_add(&rhs),
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn we_cannot_add_bigint_overflow() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![i64::MAX]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1i64]);
    assert!(matches!(
        lhs.element_wise_add(&rhs),
        Err(ColumnOperationError::IntegerOverflow { .. })
    ));
}

// ── SubOp ────────────────────────────────────────────────────────────────────

#[test]
fn we_can_sub_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![10u8, 5, 3]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![1u8, 2, 3]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::Uint8(vec![9, 3, 0])
    );
}

#[test]
fn we_can_sub_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![5i8, -2, 10]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![3i8, -5, 10]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::TinyInt(vec![2, 3, 0])
    );
}

#[test]
fn we_can_sub_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![20i16, 10, 5]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![5i16, 5, 5]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::SmallInt(vec![15, 5, 0])
    );
}

#[test]
fn we_can_sub_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![100i32, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![10i32, 20, 300]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::Int(vec![90, 180, 0])
    );
}

#[test]
fn we_can_sub_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![100i64, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1i64, 2, 3]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![99, 198, 297])
    );
}

#[test]
fn we_can_sub_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![100i128, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![1i128, 2, 3]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::Int128(vec![99, 198, 297])
    );
}

#[test]
fn we_can_sub_mixed_integer_columns() {
    // SmallInt - Int → Int (left upcast)
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![5i16, 10, 15]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![1i32, 2, 15]);
    assert_eq!(
        lhs.element_wise_sub(&rhs).unwrap(),
        OwnedColumn::Int(vec![4, 8, 0])
    );
}

#[test]
fn we_cannot_sub_columns_with_different_lengths() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
    assert_eq!(
        lhs.element_wise_sub(&rhs),
        Err(ColumnOperationError::DifferentColumnLength {
            len_a: 2,
            len_b: 1,
        })
    );
}

#[test]
fn we_cannot_sub_uint8_with_tinyint() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![5u8]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![1i8]);
    assert_eq!(
        lhs.element_wise_sub(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Uint8,
            right_type: ColumnType::TinyInt,
        })
    );
}

#[test]
fn we_cannot_sub_bigint_overflow() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![i64::MIN]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1i64]);
    assert!(matches!(
        lhs.element_wise_sub(&rhs),
        Err(ColumnOperationError::IntegerOverflow { .. })
    ));
}

// ── MulOp ────────────────────────────────────────────────────────────────────

#[test]
fn we_can_mul_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![2u8, 3, 4]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![3u8, 3, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::Uint8(vec![6, 9, 0])
    );
}

#[test]
fn we_can_mul_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![2i8, -3, 4]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![3i8, 2, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::TinyInt(vec![6, -6, 0])
    );
}

#[test]
fn we_can_mul_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![10i16, 20, 5]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![2i16, 3, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::SmallInt(vec![20, 60, 0])
    );
}

#[test]
fn we_can_mul_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![10i32, 20, 30]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![2i32, 3, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::Int(vec![20, 60, 0])
    );
}

#[test]
fn we_can_mul_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![10i64, 20, 30]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2i64, 3, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![20, 60, 0])
    );
}

#[test]
fn we_can_mul_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![10i128, 20, 30]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![2i128, 3, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::Int128(vec![20, 60, 0])
    );
}

#[test]
fn we_can_mul_mixed_integer_columns() {
    // Uint8 * BigInt → BigInt (left upcast)
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![2u8, 3, 4]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![10i64, 20, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![20, 60, 0])
    );

    // Int128 * TinyInt → Int128 (right upcast)
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![100i128, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![2i8, 3, 0]);
    assert_eq!(
        lhs.element_wise_mul(&rhs).unwrap(),
        OwnedColumn::Int128(vec![200, 600, 0])
    );
}

#[test]
fn we_cannot_mul_columns_with_different_lengths() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1, 2]);
    assert_eq!(
        lhs.element_wise_mul(&rhs),
        Err(ColumnOperationError::DifferentColumnLength {
            len_a: 3,
            len_b: 2,
        })
    );
}

#[test]
fn we_cannot_mul_uint8_with_tinyint() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![2u8]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![3i8]);
    assert_eq!(
        lhs.element_wise_mul(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Uint8,
            right_type: ColumnType::TinyInt,
        })
    );
}

#[test]
fn we_cannot_mul_incompatible_column_types() {
    let lhs = OwnedColumn::<TestScalar>::VarChar(vec!["a".into()]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
    assert!(matches!(
        lhs.element_wise_mul(&rhs),
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}

#[test]
fn we_cannot_mul_bigint_overflow() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![i64::MAX]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2i64]);
    assert!(matches!(
        lhs.element_wise_mul(&rhs),
        Err(ColumnOperationError::IntegerOverflow { .. })
    ));
}

// ── DivOp ────────────────────────────────────────────────────────────────────

#[test]
fn we_can_div_uint8_columns() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![10u8, 9, 6]);
    let rhs = OwnedColumn::<TestScalar>::Uint8(vec![2u8, 3, 6]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::Uint8(vec![5, 3, 1])
    );
}

#[test]
fn we_can_div_tinyint_columns() {
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![10i8, -9, 6]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![2i8, 3, 6]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::TinyInt(vec![5, -3, 1])
    );
}

#[test]
fn we_can_div_smallint_columns() {
    let lhs = OwnedColumn::<TestScalar>::SmallInt(vec![20i16, 15, 0]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![4i16, 5, 7]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::SmallInt(vec![5, 3, 0])
    );
}

#[test]
fn we_can_div_int_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int(vec![100i32, 200, 0]);
    let rhs = OwnedColumn::<TestScalar>::Int(vec![10i32, 20, 5]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::Int(vec![10, 10, 0])
    );
}

#[test]
fn we_can_div_bigint_columns() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![100i64, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![10i64, 10, 10]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::BigInt(vec![10, 20, 30])
    );
}

#[test]
fn we_can_div_int128_columns() {
    let lhs = OwnedColumn::<TestScalar>::Int128(vec![100i128, 200, 300]);
    let rhs = OwnedColumn::<TestScalar>::Int128(vec![10i128, 10, 10]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::Int128(vec![10, 20, 30])
    );
}

#[test]
fn we_can_div_mixed_integer_columns() {
    // TinyInt / SmallInt → SmallInt (left upcast)
    let lhs = OwnedColumn::<TestScalar>::TinyInt(vec![10i8, 9, 6]);
    let rhs = OwnedColumn::<TestScalar>::SmallInt(vec![2i16, 3, 6]);
    assert_eq!(
        lhs.element_wise_div(&rhs).unwrap(),
        OwnedColumn::SmallInt(vec![5, 3, 1])
    );
}

#[test]
fn we_cannot_div_columns_with_different_lengths() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![10, 20]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2]);
    assert_eq!(
        lhs.element_wise_div(&rhs),
        Err(ColumnOperationError::DifferentColumnLength {
            len_a: 2,
            len_b: 1,
        })
    );
}

#[test]
fn we_cannot_div_by_zero() {
    let lhs = OwnedColumn::<TestScalar>::BigInt(vec![10i64, 20]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![2i64, 0]);
    assert_eq!(
        lhs.element_wise_div(&rhs),
        Err(ColumnOperationError::DivisionByZero)
    );
}

#[test]
fn we_cannot_div_uint8_with_tinyint() {
    let lhs = OwnedColumn::<TestScalar>::Uint8(vec![10u8]);
    let rhs = OwnedColumn::<TestScalar>::TinyInt(vec![2i8]);
    assert_eq!(
        lhs.element_wise_div(&rhs),
        Err(ColumnOperationError::SignedCastingError {
            left_type: ColumnType::Uint8,
            right_type: ColumnType::TinyInt,
        })
    );
}

#[test]
fn we_cannot_div_incompatible_column_types() {
    let lhs = OwnedColumn::<TestScalar>::Boolean(vec![true]);
    let rhs = OwnedColumn::<TestScalar>::BigInt(vec![1]);
    assert!(matches!(
        lhs.element_wise_div(&rhs),
        Err(ColumnOperationError::BinaryOperationInvalidColumnType { .. })
    ));
}
