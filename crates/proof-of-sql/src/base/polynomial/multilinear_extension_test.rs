use super::MultilinearExtension;
use crate::base::{
    database::{owned_table_utility::*, Column},
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
fn we_can_use_multilinear_extension_methods_for_all_column_variants() {
    let alloc = Bump::new();
    let (_, boolean) = boolean::<TestScalar>("boolean", [true, false, true]);
    let (_, uint8) = uint8::<TestScalar>("uint8", [2_u8, 4, 8]);
    let (_, tinyint) = tinyint::<TestScalar>("tinyint", [-2_i8, 0, 2]);
    let (_, smallint) = smallint::<TestScalar>("smallint", [-3_i16, 0, 3]);
    let (_, int) = int::<TestScalar>("int", [-4_i32, 0, 4]);
    let (_, int128) = int128::<TestScalar>("int128", [-5_i128, 0, 5]);
    let (_, scalar) = scalar::<TestScalar>("scalar", [7_i64, 11, 13]);
    let (_, varchar) = varchar::<TestScalar>("varchar", ["ab", "cd", "ef"]);
    let (_, varbinary) =
        varbinary::<TestScalar>("varbinary", [vec![1_u8], vec![2_u8], vec![3_u8]]);
    let (_, decimal) = decimal75::<TestScalar>("decimal", 10, 2, [17_i64, 19, 23]);
    let (_, timestamptz) = timestamptz::<TestScalar>(
        "ts",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        [29_i64, 31, 37],
    );

    let cases = [
        Column::from_owned_column(&boolean, &alloc),
        Column::from_owned_column(&uint8, &alloc),
        Column::from_owned_column(&tinyint, &alloc),
        Column::from_owned_column(&smallint, &alloc),
        Column::from_owned_column(&int, &alloc),
        Column::from_owned_column(&int128, &alloc),
        Column::from_owned_column(&scalar, &alloc),
        Column::from_owned_column(&varchar, &alloc),
        Column::from_owned_column(&varbinary, &alloc),
        Column::from_owned_column(&decimal, &alloc),
        Column::from_owned_column(&timestamptz, &alloc),
    ];

    for column in cases {
        let padded = MultilinearExtension::<TestScalar>::to_sumcheck_term(&column, 2);
        assert_eq!(padded.len(), 4);
        assert_eq!(padded[3], 0.into());

        let mut acc = vec![0.into(); 3];
        column.mul_add(&mut acc, &2.into());
        assert_ne!(acc, vec![0.into(); 3]);

        let (ptr, len) = MultilinearExtension::<TestScalar>::id(&column);
        assert_eq!(len, 3);
        assert!(!ptr.is_null());
    }
}
