use super::MultilinearExtension;
use crate::base::{
    database::Column,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
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

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_array() {
    let values = [2_i64, 3, 4, 5];
    let evaluation_vec: Vec<TestScalar> = vec![10.into(), 20.into(), 30.into(), 40.into()];
    assert_eq!(
        (&values).inner_product(&evaluation_vec),
        (2 * 10 + 3 * 20 + 4 * 30 + 5 * 40).into()
    );

    let mut res = evaluation_vec.clone();
    (&values).mul_add(&mut res, &2.into());
    assert_eq!(res, vec![14.into(), 26.into(), 38.into(), 50.into()]);

    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&&values, 3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            0.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    let zero_point = [TestScalar::from(0_u64), TestScalar::from(0_u64)];
    assert_eq!(
        (&values).evaluate_at_point(&zero_point),
        TestScalar::from(2_u64)
    );
}

fn assert_column_mle_methods(column: Column<'_, TestScalar>, values: [TestScalar; 2]) {
    let evaluation_vec: Vec<TestScalar> = vec![3.into(), 5.into()];
    assert_eq!(
        column.inner_product(&evaluation_vec),
        values[0] * evaluation_vec[0] + values[1] * evaluation_vec[1]
    );

    let mut res = vec![1.into(), 2.into()];
    let multiplier = TestScalar::from(7_u64);
    column.mul_add(&mut res, &multiplier);
    assert_eq!(
        res,
        vec![
            TestScalar::from(1_u64) + values[0] * multiplier,
            TestScalar::from(2_u64) + values[1] * multiplier
        ]
    );

    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&column, 2),
        vec![values[0], values[1], 0.into(), 0.into()]
    );

    let (_, len) = MultilinearExtension::<TestScalar>::id(&column);
    assert_eq!(len, 2);
}

#[test]
fn we_can_use_multilinear_extension_methods_for_column_variants() {
    assert_column_mle_methods(Column::Boolean(&[false, true]), [0.into(), 1.into()]);
    assert_column_mle_methods(Column::Uint8(&[2_u8, 7]), [2.into(), 7.into()]);
    assert_column_mle_methods(Column::TinyInt(&[-2_i8, 4]), [(-2).into(), 4.into()]);
    assert_column_mle_methods(Column::SmallInt(&[-3_i16, 5]), [(-3).into(), 5.into()]);
    assert_column_mle_methods(Column::Int(&[-4_i32, 6]), [(-4).into(), 6.into()]);
    assert_column_mle_methods(Column::BigInt(&[-5_i64, 8]), [(-5).into(), 8.into()]);
    assert_column_mle_methods(Column::Int128(&[-6_i128, 9]), [(-6).into(), 9.into()]);
    assert_column_mle_methods(
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[10_i64, 11]),
        [10.into(), 11.into()],
    );

    let scalar_values = [12.into(), 13.into()];
    assert_column_mle_methods(Column::Scalar(&scalar_values), scalar_values);

    let decimal_values = [14.into(), 15.into()];
    assert_column_mle_methods(
        Column::Decimal75(Precision::new(12).unwrap(), 3, &decimal_values),
        decimal_values,
    );

    let varchar_scalars = [16.into(), 17.into()];
    assert_column_mle_methods(
        Column::VarChar((&["left", "right"], &varchar_scalars)),
        varchar_scalars,
    );

    let varbinary_values: [&[u8]; 2] = [b"aa", b"bb"];
    let varbinary_scalars = [18.into(), 19.into()];
    assert_column_mle_methods(
        Column::VarBinary((&varbinary_values, &varbinary_scalars)),
        varbinary_scalars,
    );
}
