use super::{fold_columns, fold_vals};
use crate::{
    base::scalar::test_scalar::TestScalar,
    base::{database::Column, math::decimal::Precision},
};
use bumpalo::Bump;
use num_traits::Zero;

#[test]
fn we_can_fold_columns_with_scalars() {
    let expected = vec![
        TestScalar::from(77 + 1602 * 33) + TestScalar::from(10 * 33) * TestScalar::from("1"),
        TestScalar::from(77 + 2703 * 33) + TestScalar::from(10 * 33) * TestScalar::from("2"),
        TestScalar::from(77 + 3805 * 33) + TestScalar::from(10 * 33) * TestScalar::from("3"),
        TestScalar::from(77 + 4907 * 33) + TestScalar::from(10 * 33) * TestScalar::from("4"),
        TestScalar::from(77 + 5001 * 33) + TestScalar::from(10 * 33) * TestScalar::from("5"),
    ];

    let str_scalars: [TestScalar; 5] = ["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    let scalars = [2.into(), 3.into(), 5.into(), 7.into(), 1.into()];
    let mut columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[6, 7, 8, 9, 0]),
        Column::VarChar((&["1", "2", "3", "4", "5"], &str_scalars)),
        Column::Scalar(&scalars),
    ];

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(5, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);

    columns.pop();
    columns.push(Column::Decimal75(Precision::new(75).unwrap(), -1, &scalars));

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(5, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);
}

#[test]
fn we_can_fold_columns_with_that_get_padded() {
    let expected = vec![
        TestScalar::from(77 + 1602 * 33) + TestScalar::from(10 * 33) * TestScalar::from("1"),
        TestScalar::from(77 + 2703 * 33) + TestScalar::from(10 * 33) * TestScalar::from("2"),
        TestScalar::from(77 + 3800 * 33) + TestScalar::from(10 * 33) * TestScalar::from("3"),
        TestScalar::from(77 + 4900 * 33),
        TestScalar::from(77 + 5000 * 33),
        TestScalar::from(77),
        TestScalar::from(77),
        TestScalar::from(77),
        TestScalar::from(77),
        TestScalar::from(77),
        TestScalar::from(77),
    ];

    let str_scalars: [TestScalar; 3] = ["1".into(), "2".into(), "3".into()];
    let scalars = [2.into(), 3.into()];
    let mut columns = vec![
        Column::BigInt(&[1, 2, 3, 4, 5]),
        Column::Int128(&[6, 7, 8, 9]),
        Column::VarChar((&["1", "2", "3"], &str_scalars)),
        Column::Scalar(&scalars),
    ];
    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(11, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);

    columns.pop();
    columns.push(Column::Decimal75(Precision::new(75).unwrap(), -1, &scalars));

    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(11, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);

    assert_eq!(result, expected);
}

#[test]
fn we_can_fold_empty_columns() {
    let columns = vec![
        Column::BigInt::<TestScalar>(&[]),
        Column::Int128(&[]),
        Column::VarChar((&[], &[])),
        Column::Scalar(&[]),
        Column::Decimal75(Precision::new(75).unwrap(), -1, &[]),
    ];
    let alloc = Bump::new();
    let result = alloc.alloc_slice_fill_copy(0, 77.into());
    fold_columns(result, 33.into(), 10.into(), &columns);
    assert_eq!(result, vec![]);
}

#[test]
fn we_can_fold_vals() {
    assert_eq!(fold_vals(TestScalar::from(10), &[]), Zero::zero());
    assert_eq!(
        fold_vals(
            10.into(),
            &[TestScalar::from(1), 2.into(), 3.into(), 4.into(), 5.into()]
        ),
        (12345).into()
    );
}
