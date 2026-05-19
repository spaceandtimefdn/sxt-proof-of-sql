//! Tests for owned_table_utility.rs
use crate::base::{
    database::{owned_table_utility::*, OwnedColumn, OwnedTable, ColumnType},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use sqlparser::ast::Ident;

// === owned_table() function tests ===

#[test]
fn we_can_create_an_owned_table_with_all_column_types() {
    let t = owned_table::<TestScalar>([
        uint8("u8", [1u8, 2, 3]),
        tinyint("i8", [1i8, 2, 3]),
        smallint("i16", [1i16, 2, 3]),
        int("i32", [1, 2, 3]),
        bigint("i64", [1i64, 2, 3]),
        boolean("bool", [true, false, true]),
        int128("i128", [1i128, 2, 3]),
        scalar("scalar", [TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)]),
        varchar("vc", ["a", "b", "c"]),
        decimal75("dec", 12, 1, [TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)]),
    ]);
    assert_eq!(t.num_columns(), 10);
    assert_eq!(t.num_rows(), 3);
}

#[test]
fn we_can_create_a_single_column_owned_table() {
    let t = owned_table::<TestScalar>([bigint("a", [10, 20, 30])]);
    assert_eq!(t.num_columns(), 1);
    assert_eq!(t.num_rows(), 3);
}

#[test]
fn we_can_create_an_empty_owned_table_with_columns() {
    let t = owned_table::<TestScalar>([bigint("a", vec![0i64; 0])]);
    assert_eq!(t.num_columns(), 1);
    assert_eq!(t.num_rows(), 0);
}

// === uint8 tests ===

#[test]
fn uint8_creates_correct_column() {
    let (_, col) = uint8::<TestScalar>("u8", [1u8, 2, 3]);
    assert!(matches!(col, OwnedColumn::Uint8(_)));
    assert_eq!(col.column_type(), ColumnType::Uint8);
}

// === tinyint tests ===

#[test]
fn tinyint_creates_correct_column() {
    let (_, col) = tinyint::<TestScalar>("i8", [-1i8, 0, 1]);
    assert!(matches!(col, OwnedColumn::TinyInt(_)));
    assert_eq!(col.column_type(), ColumnType::TinyInt);
}

// === smallint tests ===

#[test]
fn smallint_creates_correct_column() {
    let (_, col) = smallint::<TestScalar>("i16", [100i16, 200]);
    assert!(matches!(col, OwnedColumn::SmallInt(_)));
    assert_eq!(col.column_type(), ColumnType::SmallInt);
}

// === int tests ===

#[test]
fn int_creates_correct_column() {
    let (_, col) = int::<TestScalar>("i32", [42]);
    assert!(matches!(col, OwnedColumn::Int(_)));
    assert_eq!(col.column_type(), ColumnType::Int);
}

// === bigint tests ===

#[test]
fn bigint_creates_correct_column() {
    let (_, col) = bigint::<TestScalar>("i64", [i64::MIN, 0, i64::MAX]);
    assert!(matches!(col, OwnedColumn::BigInt(_)));
    assert_eq!(col.column_type(), ColumnType::BigInt);
}

// === boolean tests ===

#[test]
fn boolean_creates_correct_column() {
    let (_, col) = boolean::<TestScalar>("flag", [true, false]);
    assert!(matches!(col, OwnedColumn::Boolean(_)));
    assert_eq!(col.column_type(), ColumnType::Boolean);
}

// === int128 tests ===

#[test]
fn int128_creates_correct_column() {
    let (_, col) = int128::<TestScalar>("i128", [1i128, 2]);
    assert!(matches!(col, OwnedColumn::Int128(_)));
    assert_eq!(col.column_type(), ColumnType::Int128);
}

// === scalar tests ===

#[test]
fn scalar_creates_correct_column() {
    let (_, col) = scalar::<TestScalar>("s", [TestScalar::from(42)]);
    assert!(matches!(col, OwnedColumn::Scalar(_)));
    assert_eq!(col.column_type(), ColumnType::Scalar);
}

// === varchar tests ===

#[test]
fn varchar_creates_correct_column() {
    let (_, col) = varchar::<TestScalar>("name", ["hello", "world"]);
    assert!(matches!(col, OwnedColumn::VarChar(_)));
    assert_eq!(col.column_type(), ColumnType::VarChar);
}

// === varbinary tests ===

#[test]
fn varbinary_creates_correct_column() {
    let (_, col) = varbinary::<TestScalar>("data", [vec![1u8, 2], vec![3u8, 4]]);
    assert!(matches!(col, OwnedColumn::VarBinary(_)));
    assert_eq!(col.column_type(), ColumnType::VarBinary);
}

// === decimal75 tests ===

#[test]
fn decimal75_creates_correct_column() {
    let (_, col) = decimal75::<TestScalar>("price", 12, 1, [TestScalar::from(100)]);
    assert!(matches!(col, OwnedColumn::Decimal75(_, _, _)));
}

// === timestamptz tests ===

#[test]
fn timestamptz_creates_correct_column() {
    let (_, col) = timestamptz::<TestScalar>(
        "ts",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        vec![1625072400i64, 1625076000],
    );
    assert!(matches!(col, OwnedColumn::TimestampTZ(_, _, _)));
}

// === Verify owned column values through into_inner ===

#[test]
fn owned_column_bigint_values_match_input() {
    let t = owned_table::<TestScalar>([bigint("a", [10, 20, 30])]);
    let inner = t.into_inner();
    let key = Ident::new("a");
    let col_a = inner.get(&key).unwrap();
    let values: Vec<i64> = col_a.i64_iter().copied().collect();
    assert_eq!(values, &[10, 20, 30]);
}

#[test]
fn owned_column_boolean_values_match_input() {
    let t = owned_table::<TestScalar>([boolean("b", [true, false, true])]);
    let inner = t.into_inner();
    let key = Ident::new("b");
    let col_b = inner.get(&key).unwrap();
    let values: Vec<bool> = col_b.bool_iter().copied().collect();
    assert_eq!(values, &[true, false, true]);
}

#[test]
fn owned_column_int_values_match_input() {
    let t = owned_table::<TestScalar>([int("c", [1, 2, 3])]);
    let inner = t.into_inner();
    let key = Ident::new("c");
    let col_c = inner.get(&key).unwrap();
    let values: Vec<i32> = col_c.i32_iter().copied().collect();
    assert_eq!(values, &[1, 2, 3]);
}

// === Edge cases ===

#[test]
fn owned_table_with_single_row() {
    let t = owned_table::<TestScalar>([
        bigint("a", [42]),
        varchar("b", ["hello"]),
    ]);
    assert_eq!(t.num_rows(), 1);
}

#[test]
fn owned_table_with_large_values() {
    let t = owned_table::<TestScalar>([
        bigint("a", [i64::MAX, i64::MIN]),
        int128("b", [i128::MAX, i128::MIN]),
    ]);
    assert_eq!(t.num_rows(), 2);
}
