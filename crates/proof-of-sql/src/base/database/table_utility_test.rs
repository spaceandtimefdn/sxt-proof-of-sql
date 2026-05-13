use super::{
    table_utility::{
        borrowed_decimal75, borrowed_int, borrowed_smallint, borrowed_tinyint, borrowed_uint8,
        table,
    },
    Column,
};
use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};
use bumpalo::Bump;

#[test]
fn we_can_build_small_integer_helper_columns() {
    let alloc = Bump::new();

    let table = table::<TestScalar>([
        borrowed_uint8("uint8_col", [1_u8, 2, 3], &alloc),
        borrowed_tinyint("tinyint_col", [-1_i8, 0, 1], &alloc),
        borrowed_smallint("smallint_col", [-2_i16, 0, 2], &alloc),
        borrowed_int("int_col", [-3_i32, 0, 3], &alloc),
    ]);

    assert_eq!(table.num_rows(), 3);
    assert_eq!(table["uint8_col"], Column::Uint8(&[1, 2, 3]));
    assert_eq!(table["tinyint_col"], Column::TinyInt(&[-1, 0, 1]));
    assert_eq!(table["smallint_col"], Column::SmallInt(&[-2, 0, 2]));
    assert_eq!(table["int_col"], Column::Int(&[-3, 0, 3]));
}

#[test]
fn we_can_build_decimal75_helper_columns() {
    let alloc = Bump::new();
    let table = table::<TestScalar>([borrowed_decimal75(
        "decimal_col",
        12,
        -2,
        [10, -20, 30],
        &alloc,
    )]);
    let expected = [
        TestScalar::from(10),
        TestScalar::from(-20),
        TestScalar::from(30),
    ];

    assert_eq!(table.num_rows(), 3);
    assert_eq!(
        table["decimal_col"],
        Column::Decimal75(Precision::new(12).unwrap(), -2, &expected)
    );
}
