use crate::base::database::{error::ParseError, table_ref::TableRef};
use alloc::string::ToString;
use core::str::FromStr;
use indexmap::Equivalent;
use sqlparser::ast::Ident;

// TableRef::new
#[test]
fn we_can_create_table_ref_with_schema_and_table() {
    let table_ref = TableRef::new("my_schema", "my_table");
    assert_eq!(table_ref.schema_id().unwrap().value, "my_schema");
    assert_eq!(table_ref.table_id().value, "my_table");
}

#[test]
fn we_can_create_table_ref_with_empty_schema() {
    let table_ref = TableRef::new("", "my_table");
    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id().value, "my_table");
}

// TableRef::from_names
#[test]
fn we_can_create_table_ref_from_names_with_schema() {
    let table_ref = TableRef::from_names(Some("schema"), "table");
    assert_eq!(table_ref.schema_id().unwrap().value, "schema");
    assert_eq!(table_ref.table_id().value, "table");
}

#[test]
fn we_can_create_table_ref_from_names_without_schema() {
    let table_ref = TableRef::from_names(None, "table");
    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id().value, "table");
}

// TableRef::from_idents
#[test]
fn we_can_create_table_ref_from_idents() {
    let schema = Ident::new("ident_schema");
    let table = Ident::new("ident_table");
    let table_ref = TableRef::from_idents(Some(schema.clone()), table.clone());
    assert_eq!(table_ref.schema_id().unwrap(), &schema);
    assert_eq!(table_ref.table_id(), &table);
}

#[test]
fn we_can_create_table_ref_from_idents_without_schema() {
    let table = Ident::new("ident_table");
    let table_ref = TableRef::from_idents(None, table.clone());
    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id(), &table);
}

// TableRef::from_strs
#[test]
fn we_can_create_table_ref_from_single_component() {
    let table_ref = TableRef::from_strs(&["only_table"]).unwrap();
    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id().value, "only_table");
}

#[test]
fn we_can_create_table_ref_from_two_components() {
    let table_ref = TableRef::from_strs(&["sch", "tbl"]).unwrap();
    assert_eq!(table_ref.schema_id().unwrap().value, "sch");
    assert_eq!(table_ref.table_id().value, "tbl");
}

#[test]
fn we_cannot_create_table_ref_from_empty_components() {
    let result = TableRef::from_strs::<&str>(&[]);
    assert!(result.is_err());
}

#[test]
fn we_cannot_create_table_ref_from_three_components() {
    let result = TableRef::from_strs(&["a", "b", "c"]);
    assert!(result.is_err());
}

// TryFrom<&str>
#[test]
fn we_can_parse_table_ref_from_dot_separated_string_with_schema() {
    let table_ref = TableRef::try_from("schema.table").unwrap();
    assert_eq!(table_ref.schema_id().unwrap().value, "schema");
    assert_eq!(table_ref.table_id().value, "table");
}

#[test]
fn we_can_parse_table_ref_from_simple_string() {
    let table_ref = TableRef::try_from("table_only").unwrap();
    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id().value, "table_only");
}

#[test]
fn we_cannot_parse_table_ref_with_too_many_dots() {
    let result = TableRef::try_from("a.b.c");
    assert!(result.is_err());
}

// FromStr
#[test]
fn we_can_use_from_str_to_parse_table_ref() {
    let table_ref = TableRef::from_str("ns.tbl").unwrap();
    assert_eq!(table_ref.schema_id().unwrap().value, "ns");
    assert_eq!(table_ref.table_id().value, "tbl");
}

// Display
#[test]
fn we_can_display_table_ref_with_schema() {
    let table_ref = TableRef::new("schema", "table");
    assert_eq!(table_ref.to_string(), "schema.table");
}

#[test]
fn we_can_display_table_ref_without_schema() {
    let table_ref = TableRef::from_names(None, "just_table");
    assert_eq!(table_ref.to_string(), "just_table");
}

// Serialize / Deserialize round-trip
#[test]
fn we_can_serialize_and_deserialize_table_ref_with_schema() {
    let table_ref = TableRef::new("schema", "table");
    let json = serde_json::to_string(&table_ref).unwrap();
    assert_eq!(json, "\"schema.table\"");
    let deserialized: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, table_ref);
}

#[test]
fn we_can_serialize_and_deserialize_table_ref_without_schema() {
    let table_ref = TableRef::from_names(None, "table");
    let json = serde_json::to_string(&table_ref).unwrap();
    assert_eq!(json, "\"table\"");
    let deserialized: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, table_ref);
}

// Equivalent trait
#[test]
fn we_can_use_equivalent_trait_for_table_ref() {
    let table_ref1 = TableRef::new("ns", "tbl");
    let table_ref2 = TableRef::new("ns", "tbl");
    let ref1 = &table_ref1;
    assert!(Equivalent::equivalent(&ref1, &table_ref2));
}

#[test]
fn equivalent_returns_false_for_different_table_refs() {
    let table_ref1 = TableRef::new("ns", "tbl1");
    let table_ref2 = TableRef::new("ns", "tbl2");
    let ref1 = &table_ref1;
    assert!(!Equivalent::equivalent(&ref1, &table_ref2));
}

// PartialEq
#[test]
fn table_refs_with_same_parts_are_equal() {
    let a = TableRef::new("s", "t");
    let b = TableRef::new("s", "t");
    assert_eq!(a, b);
}

#[test]
fn table_refs_with_different_schemas_are_not_equal() {
    let a = TableRef::new("s1", "t");
    let b = TableRef::new("s2", "t");
    assert_ne!(a, b);
}

#[test]
fn table_refs_with_different_tables_are_not_equal() {
    let a = TableRef::new("s", "t1");
    let b = TableRef::new("s", "t2");
    assert_ne!(a, b);
}

#[test]
fn table_ref_with_schema_is_not_equal_to_one_without() {
    let a = TableRef::new("schema", "table");
    let b = TableRef::from_names(None, "table");
    assert_ne!(a, b);
}
