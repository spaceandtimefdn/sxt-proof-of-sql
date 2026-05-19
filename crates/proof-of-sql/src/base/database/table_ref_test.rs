//! Tests for table_ref.rs and column_ref.rs
use crate::base::database::{ColumnRef, ColumnType, TableRef};
use crate::base::math::decimal::Precision;
use sqlparser::ast::Ident;

// === TableRef::new tests ===

#[test]
fn table_ref_new_with_schema() {
    let tr = TableRef::new("my_schema", "my_table");
    assert_eq!(tr.schema_id().unwrap().value, "my_schema");
    assert_eq!(tr.table_id().value, "my_table");
}

#[test]
fn table_ref_new_without_schema() {
    let tr = TableRef::new("", "my_table");
    assert!(tr.schema_id().is_none());
    assert_eq!(tr.table_id().value, "my_table");
}

#[test]
fn table_ref_from_names_with_none_schema() {
    let tr = TableRef::from_names(None, "tbl");
    assert!(tr.schema_id().is_none());
    assert_eq!(tr.table_id().value, "tbl");
}

#[test]
fn table_ref_from_names_with_schema() {
    let tr = TableRef::from_names(Some("sch"), "tbl");
    assert_eq!(tr.schema_id().unwrap().value, "sch");
    assert_eq!(tr.table_id().value, "tbl");
}

// === TableRef::from_strs tests ===

#[test]
fn table_ref_from_strs_single_component() {
    let tr = TableRef::from_strs(&["tbl"]).unwrap();
    assert!(tr.schema_id().is_none());
    assert_eq!(tr.table_id().value, "tbl");
}

#[test]
fn table_ref_from_strs_two_components() {
    let tr = TableRef::from_strs(&["sch", "tbl"]).unwrap();
    assert_eq!(tr.schema_id().unwrap().value, "sch");
    assert_eq!(tr.table_id().value, "tbl");
}

#[test]
fn table_ref_from_strs_too_many_components() {
    let result = TableRef::from_strs(&["a", "b", "c"]);
    assert!(result.is_err());
}

// === TableRef FromStr tests ===

#[test]
fn table_ref_from_str_table_only() {
    let tr: TableRef = "my_table".parse().unwrap();
    assert!(tr.schema_id().is_none());
    assert_eq!(tr.table_id().value, "my_table");
}

#[test]
fn table_ref_from_str_schema_and_table() {
    let tr: TableRef = "my_schema.my_table".parse().unwrap();
    assert_eq!(tr.schema_id().unwrap().value, "my_schema");
    assert_eq!(tr.table_id().value, "my_table");
}

#[test]
fn table_ref_from_str_too_many_dots() {
    let result: Result<TableRef, _> = "a.b.c".parse();
    assert!(result.is_err());
}

// === TableRef Display tests ===

#[test]
fn table_ref_display_with_schema() {
    let tr = TableRef::new("sch", "tbl");
    assert_eq!(format!("{tr}"), "sch.tbl");
}

#[test]
fn table_ref_display_without_schema() {
    let tr = TableRef::new("", "tbl");
    assert_eq!(format!("{tr}"), "tbl");
}

// === TableRef from_idents tests ===

#[test]
fn table_ref_from_idents() {
    let tr = TableRef::from_idents(Some(Ident::new("sch")), Ident::new("tbl"));
    assert_eq!(tr.schema_id().unwrap().value, "sch");
    assert_eq!(tr.table_id().value, "tbl");
}

// === TableRef equality tests ===

#[test]
fn table_ref_equality() {
    let tr1 = TableRef::new("sch", "tbl");
    let tr2 = TableRef::new("sch", "tbl");
    assert_eq!(tr1, tr2);
}

#[test]
fn table_ref_inequality() {
    let tr1 = TableRef::new("sch", "tbl1");
    let tr2 = TableRef::new("sch", "tbl2");
    assert_ne!(tr1, tr2);
}

// === TableRef serialization roundtrip ===

#[test]
fn table_ref_serde_roundtrip_with_schema() {
    let tr = TableRef::new("schema", "table");
    let json = serde_json::to_string(&tr).unwrap();
    let deserialized: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(tr, deserialized);
}

#[test]
fn table_ref_serde_roundtrip_without_schema() {
    let tr = TableRef::new("", "table");
    let json = serde_json::to_string(&tr).unwrap();
    let deserialized: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(tr, deserialized);
}

// === ColumnRef tests ===

#[test]
fn column_ref_new() {
    let table_ref = TableRef::new("sch", "tbl");
    let cr = ColumnRef::new(table_ref, Ident::new("col"), ColumnType::BigInt);
    assert_eq!(cr.column_id().value, "col");
    assert_eq!(cr.column_type(), &ColumnType::BigInt);
}

#[test]
fn column_ref_table_ref() {
    let table_ref = TableRef::new("sch", "tbl");
    let cr = ColumnRef::new(table_ref, Ident::new("col"), ColumnType::Int);
    let returned_tr = cr.table_ref();
    assert_eq!(returned_tr.schema_id().unwrap().value, "sch");
    assert_eq!(returned_tr.table_id().value, "tbl");
}

#[test]
fn column_ref_different_types() {
    let table_ref = TableRef::new("", "tbl");

    let cr_bool = ColumnRef::new(table_ref.clone(), Ident::new("b"), ColumnType::Boolean);
    assert_eq!(cr_bool.column_type(), &ColumnType::Boolean);

    let cr_varchar = ColumnRef::new(table_ref.clone(), Ident::new("v"), ColumnType::VarChar);
    assert_eq!(cr_varchar.column_type(), &ColumnType::VarChar);

    let cr_decimal = ColumnRef::new(table_ref, Ident::new("d"), ColumnType::Decimal75(Precision::new(12).unwrap(), 1));
    assert_eq!(cr_decimal.column_type(), &ColumnType::Decimal75(Precision::new(12).unwrap(), 1));
}

#[test]
fn column_ref_clone_and_equality() {
    let table_ref = TableRef::new("sch", "tbl");
    let cr1 = ColumnRef::new(table_ref, Ident::new("col"), ColumnType::Int);
    let cr2 = cr1.clone();
    assert_eq!(cr1, cr2);
}

// === TableRef Equivalent trait ===

#[test]
fn table_ref_equivalent() {
    let tr1 = TableRef::new("sch", "tbl");
    let tr2 = TableRef::new("sch", "tbl");
    assert!(<&TableRef as indexmap::Equivalent<TableRef>>::equivalent(&&tr1, &tr2));
}
