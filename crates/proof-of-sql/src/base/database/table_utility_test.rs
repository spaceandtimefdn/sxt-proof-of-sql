use super::{table_utility::*, Column};
use crate::base::{
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn we_can_create_a_borrowed_uint8_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_uint8::<TestScalar>("a", [1_u8, 2, 3], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(column, Column::Uint8(&[1, 2, 3]));
}

#[test]
fn we_can_create_a_borrowed_tinyint_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_tinyint::<TestScalar>("a", [1_i8, -2, 3], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(column, Column::TinyInt(&[1, -2, 3]));
}

#[test]
fn we_can_create_a_borrowed_smallint_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_smallint::<TestScalar>("a", [1_i16, -2, 300], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(column, Column::SmallInt(&[1, -2, 300]));
}

#[test]
fn we_can_create_a_borrowed_int_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_int::<TestScalar>("a", [1, -2, 70000], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(column, Column::Int(&[1, -2, 70000]));
}

#[test]
fn we_can_create_a_borrowed_bigint_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_bigint::<TestScalar>("a", [1_i64, -2, 5_000_000_000], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(column, Column::BigInt(&[1, -2, 5_000_000_000]));
}

#[test]
fn we_can_create_a_borrowed_boolean_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_boolean::<TestScalar>("a", [true, false, true], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(column, Column::Boolean(&[true, false, true]));
}

#[test]
fn we_can_create_a_borrowed_int128_column() {
    let alloc = Bump::new();
    let (ident, column) =
        borrowed_int128::<TestScalar>("a", [1_i128, -2, 170_000_000_000_000_000_000], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(
        column,
        Column::Int128(&[1, -2, 170_000_000_000_000_000_000])
    );
}

#[test]
fn we_can_create_a_borrowed_scalar_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_scalar::<TestScalar>("a", [1, 2, 3], &alloc);
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(
        column,
        Column::Scalar(&[
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ])
    );
}

#[test]
fn we_can_create_a_borrowed_varchar_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_varchar::<TestScalar>("a", ["foo", "bar"], &alloc);
    assert_eq!(ident, Ident::new("a"));
    // VarChar stores both the string slices and their scalar hashes; assert on
    // both so the scalar-derivation path is exercised, not just the strings.
    match column {
        Column::VarChar((strings, scalars)) => {
            assert_eq!(strings, &["foo", "bar"]);
            assert_eq!(scalars, &[TestScalar::from("foo"), TestScalar::from("bar")]);
        }
        _ => panic!("expected a VarChar column"),
    }
}

#[test]
fn we_can_create_a_borrowed_decimal75_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_decimal75::<TestScalar>("a", 12, 2, [1, 2, 3], &alloc);
    assert_eq!(ident, Ident::new("a"));
    match column {
        Column::Decimal75(precision, scale, values) => {
            assert_eq!(precision, Precision::new(12).unwrap());
            assert_eq!(scale, 2);
            assert_eq!(
                values,
                &[
                    TestScalar::from(1),
                    TestScalar::from(2),
                    TestScalar::from(3),
                ]
            );
        }
        _ => panic!("expected a Decimal75 column"),
    }
}

#[test]
fn we_can_create_a_borrowed_timestamptz_column() {
    let alloc = Bump::new();
    let (ident, column) = borrowed_timestamptz::<TestScalar>(
        "a",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        [1_625_072_400_i64, 1_625_076_000],
        &alloc,
    );
    assert_eq!(ident, Ident::new("a"));
    assert_eq!(
        column,
        Column::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &[1_625_072_400, 1_625_076_000]
        )
    );
}

#[test]
fn we_can_build_a_multi_column_table() {
    let alloc = Bump::new();
    let table = table::<TestScalar>([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_boolean("b", [true, false, true], &alloc),
        borrowed_varchar("c", ["x", "y", "z"], &alloc),
    ]);
    assert_eq!(table.num_columns(), 3);
    assert_eq!(table.num_rows(), 3);

    let inner = table.inner_table();
    assert_eq!(
        inner.get(&Ident::new("a")),
        Some(&Column::BigInt(&[1, 2, 3]))
    );
    assert_eq!(
        inner.get(&Ident::new("b")),
        Some(&Column::Boolean(&[true, false, true]))
    );
    // Column ordering is preserved from the input iterator.
    let names: Vec<&Ident> = table.column_names().collect();
    assert_eq!(
        names,
        vec![&Ident::new("a"), &Ident::new("b"), &Ident::new("c")]
    );
}

#[test]
fn we_can_build_a_table_with_an_explicit_row_count_and_no_columns() {
    let alloc = Bump::new();
    // The whole point of `table_with_row_count` is supporting zero-column
    // tables that still carry a row count.
    let empty = table_with_row_count::<TestScalar>([], 5);
    assert_eq!(empty.num_columns(), 0);
    assert_eq!(empty.num_rows(), 5);
    assert!(empty.is_empty());

    // It must also agree with the column lengths when columns are present.
    let with_cols = table_with_row_count::<TestScalar>([borrowed_int("a", [10, 20], &alloc)], 2);
    assert_eq!(with_cols.num_columns(), 1);
    assert_eq!(with_cols.num_rows(), 2);
}

#[test]
#[should_panic(expected = "")]
fn we_panic_when_row_count_disagrees_with_column_length() {
    let alloc = Bump::new();
    // Declared row count (3) does not match the actual column length (2).
    let _ = table_with_row_count::<TestScalar>([borrowed_int("a", [10, 20], &alloc)], 3);
}
