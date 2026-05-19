//! Tests for table_utility.rs
use crate::base::{
    database::{
        table_utility::*, Column, ColumnType, Table,
    },
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
use sqlparser::ast::Ident;

// === table() function tests ===

#[test]
fn we_can_create_a_table_with_all_borrowed_column_types() {
    let alloc = Bump::new();
    let t = table::<TestScalar>([
        borrowed_uint8("u8", [1u8, 2, 3], &alloc),
        borrowed_tinyint("i8", [1i8, 2, 3], &alloc),
        borrowed_smallint("i16", [1i16, 2, 3], &alloc),
        borrowed_int("i32", [1, 2, 3], &alloc),
        borrowed_bigint("i64", [1i64, 2, 3], &alloc),
        borrowed_boolean("bool", [true, false, true], &alloc),
        borrowed_int128("i128", [1i128, 2, 3], &alloc),
        borrowed_scalar("scalar", [TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)], &alloc),
        borrowed_varchar("vc", ["a", "b", "c"], &alloc),
        borrowed_decimal75("dec", 12, 1, [TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)], &alloc),
    ]);
    assert_eq!(t.num_columns(), 10);
    assert_eq!(t.num_rows(), 3);
}

#[test]
fn we_can_create_a_single_column_table() {
    let alloc = Bump::new();
    let t = table::<TestScalar>([borrowed_bigint("a", [10, 20, 30], &alloc)]);
    assert_eq!(t.num_columns(), 1);
    assert_eq!(t.num_rows(), 3);
}

#[test]
fn we_can_create_an_empty_column_table() {
    let alloc = Bump::new();
    let t = table::<TestScalar>([borrowed_bigint("a", vec![0i64; 0], &alloc)]);
    assert_eq!(t.num_columns(), 1);
    assert_eq!(t.num_rows(), 0);
}

// === borrowed_uint8 tests ===

#[test]
fn borrowed_uint8_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_uint8::<TestScalar>("test_u8", [1u8, 2, 3], &alloc);
    assert!(matches!(col, Column::Uint8(_)));
    assert_eq!(col.column_type(), ColumnType::Uint8);
}

// === borrowed_tinyint tests ===

#[test]
fn borrowed_tinyint_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_tinyint::<TestScalar>("test_i8", [-1i8, 0, 1], &alloc);
    assert!(matches!(col, Column::TinyInt(_)));
    assert_eq!(col.column_type(), ColumnType::TinyInt);
}

// === borrowed_smallint tests ===

#[test]
fn borrowed_smallint_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_smallint::<TestScalar>("test_i16", [100i16, 200, 300], &alloc);
    assert!(matches!(col, Column::SmallInt(_)));
    assert_eq!(col.column_type(), ColumnType::SmallInt);
}

// === borrowed_int tests ===

#[test]
fn borrowed_int_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_int::<TestScalar>("test_i32", [42], &alloc);
    assert!(matches!(col, Column::Int(_)));
    assert_eq!(col.column_type(), ColumnType::Int);
}

// === borrowed_bigint tests ===

#[test]
fn borrowed_bigint_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_bigint::<TestScalar>("test_i64", [i64::MIN, 0, i64::MAX], &alloc);
    assert!(matches!(col, Column::BigInt(_)));
    assert_eq!(col.column_type(), ColumnType::BigInt);
}

// === borrowed_boolean tests ===

#[test]
fn borrowed_boolean_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_boolean::<TestScalar>("flag", [true, false], &alloc);
    assert!(matches!(col, Column::Boolean(_)));
    assert_eq!(col.column_type(), ColumnType::Boolean);
}

// === borrowed_int128 tests ===

#[test]
fn borrowed_int128_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_int128::<TestScalar>("test_i128", [1i128, 2], &alloc);
    assert!(matches!(col, Column::Int128(_)));
    assert_eq!(col.column_type(), ColumnType::Int128);
}

// === borrowed_scalar tests ===

#[test]
fn borrowed_scalar_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_scalar::<TestScalar>("s", [TestScalar::from(42)], &alloc);
    assert!(matches!(col, Column::Scalar(_)));
    assert_eq!(col.column_type(), ColumnType::Scalar);
}

// === borrowed_varchar tests ===

#[test]
fn borrowed_varchar_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_varchar::<TestScalar>("name", ["hello", "world"], &alloc);
    assert!(matches!(col, Column::VarChar(_)));
    assert_eq!(col.column_type(), ColumnType::VarChar);
}

// === borrowed_decimal75 tests ===

#[test]
fn borrowed_decimal75_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_decimal75::<TestScalar>("price", 12, 1, [TestScalar::from(100)], &alloc);
    assert!(matches!(col, Column::Decimal75(_, _, _)));
}

// === borrowed_timestamptz tests ===

#[test]
fn borrowed_timestamptz_creates_correct_column() {
    let alloc = Bump::new();
    let (_ident, col) = borrowed_timestamptz::<TestScalar>(
        "ts",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        [1625072400i64, 1625076000],
        &alloc,
    );
    assert!(matches!(col, Column::TimestampTZ(_, _, _)));
}

// === table_with_row_count tests ===

#[test]
fn we_can_create_a_table_with_specified_row_count() {
    let alloc = Bump::new();
    let t = table_with_row_count::<TestScalar>(
        [borrowed_bigint("a", [1, 2, 3], &alloc)],
        3,
    );
    assert_eq!(t.num_rows(), 3);
    assert_eq!(t.num_columns(), 1);
}

#[test]
fn we_can_create_a_table_with_no_columns_but_specified_row_count() {
    let t = table_with_row_count::<TestScalar>([], 5);
    assert_eq!(t.num_columns(), 0);
    assert_eq!(t.num_rows(), 5);
}

// === Verify values through inner_table access ===

#[test]
fn borrowed_column_values_match_input() {
    let alloc = Bump::new();
    let t = table::<TestScalar>([
        borrowed_bigint("a", [10, 20, 30], &alloc),
        borrowed_boolean("b", [true, false, true], &alloc),
        borrowed_int("c", [1, 2, 3], &alloc),
    ]);
    let inner = t.inner_table();
    let key_a = Ident::new("a");
    let col_a = inner.get(&key_a).unwrap();
    assert_eq!(col_a.as_bigint().unwrap(), &[10, 20, 30]);

    let key_b = Ident::new("b");
    let col_b = inner.get(&key_b).unwrap();
    assert_eq!(col_b.as_boolean().unwrap(), &[true, false, true]);

    let key_c = Ident::new("c");
    let col_c = inner.get(&key_c).unwrap();
    assert_eq!(col_c.as_int().unwrap(), &[1, 2, 3]);
}
