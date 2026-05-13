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
fn we_can_evaluate_multilinear_extensions_at_a_point() {
    let slice: &[i64] = &[8, 13];

    assert_eq!(slice.evaluate_at_point(&[TestScalar::from(0)]), 8.into());
    assert_eq!(slice.evaluate_at_point(&[TestScalar::from(1)]), 13.into());
}

#[test]
fn we_can_use_multilinear_extension_methods_for_array_refs() {
    let array = &[2_i64, 3, 4, 5];
    let evaluation_vec: Vec<TestScalar> = vec![11.into(), 13.into(), 17.into(), 19.into()];

    assert_eq!(
        array.inner_product(&evaluation_vec),
        (2 * 11 + 3 * 13 + 4 * 17 + 5 * 19).into()
    );

    let mut res: Vec<TestScalar> = vec![1.into(), 2.into(), 3.into(), 4.into()];
    array.mul_add(&mut res, &10.into());
    assert_eq!(res, vec![21.into(), 32.into(), 43.into(), 54.into()]);
    assert_eq!(
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&array, 2),
        vec![2.into(), 3.into(), 4.into(), 5.into()]
    );
    assert_eq!(MultilinearExtension::<TestScalar>::id(&array).1, 4);
}

fn assert_column_mle_methods(column: Column<'_, TestScalar>, expected: &[TestScalar]) {
    let evaluation_vec: Vec<TestScalar> = vec![11.into(), 13.into()];
    assert_eq!(
        column.inner_product(&evaluation_vec),
        expected[0] * evaluation_vec[0] + expected[1] * evaluation_vec[1]
    );

    let mut res: Vec<TestScalar> = vec![17.into(), 19.into()];
    let multiplier = TestScalar::from(2);
    column.mul_add(&mut res, &multiplier);
    assert_eq!(
        res,
        vec![
            TestScalar::from(17) + expected[0] * multiplier,
            TestScalar::from(19) + expected[1] * multiplier
        ]
    );

    assert_eq!(
        column.to_sumcheck_term(2),
        vec![expected[0], expected[1], 0.into(), 0.into()]
    );
    assert_eq!(column.id().1, 2);
}

#[test]
fn we_can_use_multilinear_extension_methods_for_all_column_variants() {
    let bools = [true, false];
    assert_column_mle_methods(Column::Boolean(&bools), &[1.into(), 0.into()]);

    let uint8s = [2_u8, 3];
    assert_column_mle_methods(Column::Uint8(&uint8s), &[2.into(), 3.into()]);

    let tinyints = [2_i8, 3];
    assert_column_mle_methods(Column::TinyInt(&tinyints), &[2.into(), 3.into()]);

    let smallints = [2_i16, 3];
    assert_column_mle_methods(Column::SmallInt(&smallints), &[2.into(), 3.into()]);

    let ints = [2_i32, 3];
    assert_column_mle_methods(Column::Int(&ints), &[2.into(), 3.into()]);

    let bigints = [2_i64, 3];
    assert_column_mle_methods(Column::BigInt(&bigints), &[2.into(), 3.into()]);

    let int128s = [2_i128, 3];
    assert_column_mle_methods(Column::Int128(&int128s), &[2.into(), 3.into()]);

    let scalars = [2.into(), 3.into()];
    assert_column_mle_methods(Column::Scalar(&scalars), &[2.into(), 3.into()]);

    let decimal_scalars = [2.into(), 3.into()];
    assert_column_mle_methods(
        Column::Decimal75(Precision::new(10).unwrap(), 2, &decimal_scalars),
        &[2.into(), 3.into()],
    );

    let timestamps = [2_i64, 3];
    assert_column_mle_methods(
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps),
        &[2.into(), 3.into()],
    );

    let strings = ["alpha", "beta"];
    let string_scalars = [5.into(), 7.into()];
    assert_column_mle_methods(
        Column::VarChar((&strings, &string_scalars)),
        &[5.into(), 7.into()],
    );

    let binaries: [&[u8]; 2] = [b"alpha", b"beta"];
    let binary_scalars = [5.into(), 7.into()];
    assert_column_mle_methods(
        Column::VarBinary((&binaries, &binary_scalars)),
        &[5.into(), 7.into()],
    );
}
