use super::{ColumnOperationError, ColumnType, OwnedColumn};
use crate::base::{
    math::{decimal::Precision, permutation::Permutation},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use alloc::{string::ToString, vec, vec::Vec};

fn precision() -> Precision {
    Precision::new(5).unwrap()
}

fn test_scalars(values: [i64; 3]) -> Vec<TestScalar> {
    values.into_iter().map(TestScalar::from).collect()
}

fn decimal_column(values: [i64; 3]) -> OwnedColumn<TestScalar> {
    OwnedColumn::Decimal75(precision(), 0, test_scalars(values))
}

fn scalar_column(values: [i64; 3]) -> OwnedColumn<TestScalar> {
    OwnedColumn::Scalar(test_scalars(values))
}

fn timestamp_column(values: [i64; 3]) -> OwnedColumn<TestScalar> {
    OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), values.to_vec())
}

fn assert_eq_case(lhs: OwnedColumn<TestScalar>, rhs: OwnedColumn<TestScalar>, expected: [bool; 2]) {
    assert_eq!(
        lhs.element_wise_eq(&rhs).unwrap(),
        OwnedColumn::Boolean(expected.to_vec())
    );
}

fn assert_signed_cast_error(lhs: OwnedColumn<TestScalar>, rhs: OwnedColumn<TestScalar>) {
    assert!(matches!(
        lhs.element_wise_eq(&rhs),
        Err(ColumnOperationError::SignedCastingError { .. })
    ));
}

fn assert_add_case(
    lhs: OwnedColumn<TestScalar>,
    rhs: OwnedColumn<TestScalar>,
    expected: OwnedColumn<TestScalar>,
) {
    assert_eq!(lhs.element_wise_add(&rhs).unwrap(), expected);
}

fn assert_decimal_add_case(
    lhs: OwnedColumn<TestScalar>,
    rhs: OwnedColumn<TestScalar>,
    expected_values: [i64; 2],
) {
    match lhs.element_wise_add(&rhs).unwrap() {
        OwnedColumn::Decimal75(_, scale, values) => {
            assert_eq!(scale, 0);
            assert_eq!(
                values,
                expected_values
                    .into_iter()
                    .map(TestScalar::from)
                    .collect::<Vec<_>>()
            );
        }
        other => panic!("expected decimal result, got {other:?}"),
    }
}

#[test]
fn owned_column_variant_helpers_cover_all_runtime_variants() {
    let permutation = Permutation::try_new(vec![2, 0, 1]).unwrap();
    let timestamp_type = ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc());

    let cases = vec![
        (
            OwnedColumn::Boolean(vec![true, false, true]),
            ColumnType::Boolean,
            OwnedColumn::Boolean(vec![false, true]),
            OwnedColumn::Boolean(vec![true, true, false]),
        ),
        (
            OwnedColumn::Uint8(vec![1, 2, 3]),
            ColumnType::Uint8,
            OwnedColumn::Uint8(vec![2, 3]),
            OwnedColumn::Uint8(vec![3, 1, 2]),
        ),
        (
            OwnedColumn::TinyInt(vec![1, 2, 3]),
            ColumnType::TinyInt,
            OwnedColumn::TinyInt(vec![2, 3]),
            OwnedColumn::TinyInt(vec![3, 1, 2]),
        ),
        (
            OwnedColumn::SmallInt(vec![1, 2, 3]),
            ColumnType::SmallInt,
            OwnedColumn::SmallInt(vec![2, 3]),
            OwnedColumn::SmallInt(vec![3, 1, 2]),
        ),
        (
            OwnedColumn::Int(vec![1, 2, 3]),
            ColumnType::Int,
            OwnedColumn::Int(vec![2, 3]),
            OwnedColumn::Int(vec![3, 1, 2]),
        ),
        (
            OwnedColumn::BigInt(vec![1, 2, 3]),
            ColumnType::BigInt,
            OwnedColumn::BigInt(vec![2, 3]),
            OwnedColumn::BigInt(vec![3, 1, 2]),
        ),
        (
            OwnedColumn::Int128(vec![1, 2, 3]),
            ColumnType::Int128,
            OwnedColumn::Int128(vec![2, 3]),
            OwnedColumn::Int128(vec![3, 1, 2]),
        ),
        (
            OwnedColumn::VarChar(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            ColumnType::VarChar,
            OwnedColumn::VarChar(vec!["b".to_string(), "c".to_string()]),
            OwnedColumn::VarChar(vec!["c".to_string(), "a".to_string(), "b".to_string()]),
        ),
        (
            decimal_column([1, 2, 3]),
            ColumnType::Decimal75(precision(), 0),
            decimal_column([2, 3, 0]).slice(0, 2),
            decimal_column([3, 1, 2]),
        ),
        (
            scalar_column([1, 2, 3]),
            ColumnType::Scalar,
            scalar_column([2, 3, 0]).slice(0, 2),
            scalar_column([3, 1, 2]),
        ),
        (
            timestamp_column([1, 2, 3]),
            timestamp_type,
            timestamp_column([2, 3, 0]).slice(0, 2),
            timestamp_column([3, 1, 2]),
        ),
        (
            OwnedColumn::VarBinary(vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]),
            ColumnType::VarBinary,
            OwnedColumn::VarBinary(vec![b"b".to_vec(), b"c".to_vec()]),
            OwnedColumn::VarBinary(vec![b"c".to_vec(), b"a".to_vec(), b"b".to_vec()]),
        ),
    ];

    for (column, expected_type, expected_slice, expected_permutation) in cases {
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());
        assert_eq!(column.column_type(), expected_type);
        assert_eq!(column.slice(1, 3), expected_slice);
        assert_eq!(
            column.try_permute(&permutation).unwrap(),
            expected_permutation
        );
    }
}

#[test]
fn owned_column_scalar_conversions_cover_numeric_scalar_and_timestamp_targets() {
    let scalars = test_scalars([1, 2, 3]);
    let precision = precision();
    let timezone = PoSQLTimeZone::utc();

    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::Uint8).unwrap(),
        OwnedColumn::Uint8(vec![1, 2, 3])
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::TinyInt).unwrap(),
        OwnedColumn::TinyInt(vec![1, 2, 3])
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::SmallInt).unwrap(),
        OwnedColumn::SmallInt(vec![1, 2, 3])
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::Int).unwrap(),
        OwnedColumn::Int(vec![1, 2, 3])
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::BigInt).unwrap(),
        OwnedColumn::BigInt(vec![1, 2, 3])
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::Int128).unwrap(),
        OwnedColumn::Int128(vec![1, 2, 3])
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::Scalar).unwrap(),
        OwnedColumn::Scalar(scalars.clone())
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(&scalars, ColumnType::Decimal75(precision, 0)).unwrap(),
        OwnedColumn::Decimal75(precision, 0, scalars.clone())
    );
    assert_eq!(
        OwnedColumn::try_from_scalars(
            &scalars,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, timezone),
        )
        .unwrap(),
        OwnedColumn::TimestampTZ(PoSQLTimeUnit::Second, timezone, vec![1, 2, 3])
    );
}

#[test]
fn owned_column_test_iterators_and_uint8_coercion_cover_remaining_helpers() {
    let scalars = test_scalars([1, 2, 3]);

    assert_eq!(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2, 3])
            .u8_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2, 3])
            .i8_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2, 3])
            .i16_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::Int(vec![1, 2, 3])
            .i32_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        timestamp_column([1, 2, 3])
            .i64_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2, 3])
            .i128_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::Boolean(vec![true, false, true])
            .bool_iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![true, false, true]
    );
    assert_eq!(
        scalar_column([1, 2, 3])
            .scalar_iter()
            .cloned()
            .collect::<Vec<_>>(),
        scalars.clone()
    );
    assert_eq!(
        decimal_column([1, 2, 3])
            .scalar_iter()
            .cloned()
            .collect::<Vec<_>>(),
        scalars.clone()
    );
    assert_eq!(
        OwnedColumn::<TestScalar>::VarChar(vec!["a".to_string(), "b".to_string()])
            .string_iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(
        OwnedColumn::Scalar(scalars)
            .try_coerce_scalar_to_numeric(ColumnType::Uint8)
            .unwrap(),
        OwnedColumn::Uint8(vec![1, 2, 3])
    );
}

#[test]
fn equality_operations_cover_unhit_numeric_promotion_paths() {
    assert_signed_cast_error(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 3]),
    );
    assert_signed_cast_error(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Uint8(vec![1, 3]),
    );

    assert_eq_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::Uint8(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(3)],
        ),
        [true, false],
    );

    assert_eq_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(3)],
        ),
        [true, false],
    );

    assert_eq_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(3)],
        ),
        [true, false],
    );

    assert_eq_case(
        OwnedColumn::<TestScalar>::Int(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(3)],
        ),
        [true, false],
    );

    assert_eq_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(3)],
        ),
        [true, false],
    );

    assert_eq_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(3)],
        ),
        [true, false],
    );

    assert_eq_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 3]),
        [true, false],
    );
    assert_eq_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::Int128(vec![1, 3]),
        [true, false],
    );
}

#[test]
fn addition_operations_cover_unhit_numeric_promotion_paths() {
    assert!(matches!(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2])
            .element_wise_add(&OwnedColumn::<TestScalar>::TinyInt(vec![10, 20])),
        Err(ColumnOperationError::SignedCastingError { .. })
    ));
    assert!(matches!(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2])
            .element_wise_add(&OwnedColumn::<TestScalar>::Uint8(vec![10, 20])),
        Err(ColumnOperationError::SignedCastingError { .. })
    ));

    assert_add_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::Uint8(vec![10, 20]),
        OwnedColumn::<TestScalar>::Uint8(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::SmallInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::BigInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_decimal_add_case(
        OwnedColumn::<TestScalar>::Uint8(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(10), TestScalar::from(20)],
        ),
        [11, 22],
    );

    assert_add_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::TinyInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::SmallInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::BigInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_decimal_add_case(
        OwnedColumn::<TestScalar>::TinyInt(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(10), TestScalar::from(20)],
        ),
        [11, 22],
    );

    assert_add_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::SmallInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::BigInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::SmallInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );

    assert_add_case(
        OwnedColumn::<TestScalar>::Int(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Int(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int(vec![11, 22]),
    );

    assert_add_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::BigInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::BigInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![10, 20]),
        OwnedColumn::<TestScalar>::BigInt(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_decimal_add_case(
        OwnedColumn::<TestScalar>::BigInt(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(10), TestScalar::from(20)],
        ),
        [11, 22],
    );

    assert_add_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::TinyInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::SmallInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::BigInt(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_add_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::<TestScalar>::Int128(vec![10, 20]),
        OwnedColumn::<TestScalar>::Int128(vec![11, 22]),
    );
    assert_decimal_add_case(
        OwnedColumn::<TestScalar>::Int128(vec![1, 2]),
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(10), TestScalar::from(20)],
        ),
        [11, 22],
    );

    assert_decimal_add_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::TinyInt(vec![10, 20]),
        [11, 22],
    );
    assert_decimal_add_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::SmallInt(vec![10, 20]),
        [11, 22],
    );
    assert_decimal_add_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::Int(vec![10, 20]),
        [11, 22],
    );
    assert_decimal_add_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::BigInt(vec![10, 20]),
        [11, 22],
    );
    assert_decimal_add_case(
        OwnedColumn::Decimal75(
            precision(),
            0,
            vec![TestScalar::from(1), TestScalar::from(2)],
        ),
        OwnedColumn::<TestScalar>::Int128(vec![10, 20]),
        [11, 22],
    );
}
