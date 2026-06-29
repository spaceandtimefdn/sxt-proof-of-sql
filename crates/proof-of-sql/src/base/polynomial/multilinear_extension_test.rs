use super::MultilinearExtension;
use crate::base::{database::Column, scalar::test_scalar::TestScalar};
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
fn we_can_get_sumcheck_terms_for_remaining_column_variants() {
    let bools = [true, false];
    assert_eq!(
        Column::<TestScalar>::Boolean(&bools).to_sumcheck_term(2),
        vec![true.into(), false.into(), 0.into(), 0.into()]
    );

    let scalars = [TestScalar::from(7), TestScalar::from(8)];
    assert_eq!(
        Column::Scalar(&scalars).to_sumcheck_term(2),
        vec![7.into(), 8.into(), 0.into(), 0.into()]
    );

    let strings = ["alpha", "beta"];
    let string_scalars = [TestScalar::from("alpha"), TestScalar::from("beta")];
    assert_eq!(
        Column::VarChar((&strings, &string_scalars)).to_sumcheck_term(2),
        vec![
            TestScalar::from("alpha"),
            TestScalar::from("beta"),
            0.into(),
            0.into()
        ]
    );

    let alpha_bytes = b"alpha".as_slice();
    let beta_bytes = b"beta".as_slice();
    let binaries = [alpha_bytes, beta_bytes];
    let binary_scalars = [TestScalar::from(11), TestScalar::from(12)];
    assert_eq!(
        Column::VarBinary((&binaries, &binary_scalars)).to_sumcheck_term(2),
        vec![11.into(), 12.into(), 0.into(), 0.into()]
    );

    let uints = [2_u8, 3];
    assert_eq!(
        Column::<TestScalar>::Uint8(&uints).to_sumcheck_term(2),
        vec![2.into(), 3.into(), 0.into(), 0.into()]
    );

    let tiny_ints = [-2_i8, 3];
    assert_eq!(
        Column::<TestScalar>::TinyInt(&tiny_ints).to_sumcheck_term(2),
        vec![(-2).into(), 3.into(), 0.into(), 0.into()]
    );
}

#[test]
fn we_can_get_ids_for_remaining_column_variants() {
    let bools = [true, false];
    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&Column::Boolean(&bools)).1,
        bools.len()
    );

    let scalars = [TestScalar::from(7), TestScalar::from(8)];
    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&Column::Scalar(&scalars)).1,
        scalars.len()
    );

    let strings = ["alpha", "beta"];
    let string_scalars = [TestScalar::from("alpha"), TestScalar::from("beta")];
    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&Column::VarChar((&strings, &string_scalars))).1,
        string_scalars.len()
    );

    let alpha_bytes = b"alpha".as_slice();
    let beta_bytes = b"beta".as_slice();
    let binaries = [alpha_bytes, beta_bytes];
    let binary_scalars = [TestScalar::from(11), TestScalar::from(12)];
    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&Column::VarBinary((&binaries, &binary_scalars))).1,
        binary_scalars.len()
    );

    let uints = [2_u8, 3];
    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&Column::Uint8(&uints)).1,
        uints.len()
    );

    let tiny_ints = [-2_i8, 3];
    assert_eq!(
        MultilinearExtension::<TestScalar>::id(&Column::TinyInt(&tiny_ints)).1,
        tiny_ints.len()
    );
}
