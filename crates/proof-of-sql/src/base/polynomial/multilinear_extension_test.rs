use super::MultilinearExtension;
use crate::base::{
    database::Column,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{test_scalar::TestScalar, ScalarExt},
};
use bumpalo::Bump;

#[test]
fn allocated_slices_must_have_different_ids_even_when_one_is_empty() {
    let alloc = Bump::new();
    let foo = alloc.alloc_slice_fill_default(5) as &[TestScalar];
    let bar = alloc.alloc_slice_fill_default(0) as &[TestScalar];
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&foo),
        MultilinearExtension::<TestScalar>::id(&bar)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_slice() {
    let slice: &[i64] = &[2, 3, 4, 5, 6];
    let evaluation_vec: Vec<TestScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_column() {
    let slice = Column::BigInt(&[2, 3, 4, 5, 6]);
    let evaluation_vec: Vec<TestScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_vec() {
    let slice: &Vec<i64> = &vec![2, 3, 4, 5, 6];
    let evaluation_vec: Vec<TestScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

fn assert_column_mle(column: Column<'_, TestScalar>, expected_values: &[TestScalar]) {
    let evaluation_vec: Vec<TestScalar> = vec![2.into(), 3.into(), 5.into()];
    let expected_inner_product = expected_values
        .iter()
        .zip(evaluation_vec.iter())
        .map(|(value, weight)| *value * *weight)
        .sum();
    assert_eq!(
        column.inner_product(&evaluation_vec),
        expected_inner_product
    );

    let mut res: Vec<TestScalar> = vec![10.into(), 20.into(), 30.into()];
    column.mul_add(&mut res, &7.into());
    assert_eq!(
        res,
        expected_values
            .iter()
            .zip([10, 20, 30])
            .map(|(value, base)| TestScalar::from(base) + TestScalar::from(7) * *value)
            .collect::<Vec<_>>()
    );

    let mut expected_sumcheck_term = expected_values.to_vec();
    expected_sumcheck_term.push(0.into());
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&column, 2),
        expected_sumcheck_term
    );

    let (ptr, len) = column.id();
    assert!(!ptr.is_null());
    assert_eq!(len, expected_values.len());
}

#[test]
fn we_can_use_multilinear_extension_methods_for_each_column_variant() {
    let scalar_values: Vec<TestScalar> = vec![3.into(), 4.into(), 5.into()];
    assert_column_mle(
        Column::Boolean(&[true, false, true]),
        &[1.into(), 0.into(), 1.into()],
    );
    assert_column_mle(Column::Uint8(&[3, 4, 5]), &scalar_values);
    assert_column_mle(
        Column::TinyInt(&[-3, 4, -5]),
        &[-TestScalar::from(3), 4.into(), -TestScalar::from(5)],
    );
    assert_column_mle(
        Column::SmallInt(&[-13, 14, -15]),
        &[-TestScalar::from(13), 14.into(), -TestScalar::from(15)],
    );
    assert_column_mle(
        Column::Int(&[-23, 24, -25]),
        &[-TestScalar::from(23), 24.into(), -TestScalar::from(25)],
    );
    assert_column_mle(
        Column::BigInt(&[-33, 34, -35]),
        &[-TestScalar::from(33), 34.into(), -TestScalar::from(35)],
    );
    assert_column_mle(
        Column::Int128(&[-43, 44, -45]),
        &[-TestScalar::from(43), 44.into(), -TestScalar::from(45)],
    );
    assert_column_mle(Column::Scalar(&scalar_values), &scalar_values);
    assert_column_mle(
        Column::Decimal75(Precision::new(5).unwrap(), 2, &scalar_values),
        &scalar_values,
    );
    assert_column_mle(
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[53, 54, 55]),
        &[53.into(), 54.into(), 55.into()],
    );

    let varchar_scalars = [
        TestScalar::from("alpha"),
        TestScalar::from("beta"),
        TestScalar::from("gamma"),
    ];
    assert_column_mle(
        Column::VarChar((&["alpha", "beta", "gamma"], &varchar_scalars)),
        &varchar_scalars,
    );

    let raw_bytes: [&[u8]; 3] = [b"red", b"green", b"blue"];
    let binary_scalars = raw_bytes.map(TestScalar::from_byte_slice_via_hash);
    assert_column_mle(
        Column::VarBinary((&raw_bytes, &binary_scalars)),
        &binary_scalars,
    );
}
