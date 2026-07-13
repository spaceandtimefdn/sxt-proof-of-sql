use crate::base::database::{error::ParseError, TableRef};
use core::str::FromStr;
use sqlparser::ast::Ident;

// --- new() ---

#[test]
fn we_can_create_table_ref_with_schema() {
    let t = TableRef::new("myschema", "mytable");
    assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("myschema"));
    assert_eq!(t.table_id().value, "mytable");
}

#[test]
fn we_can_create_table_ref_without_schema_via_empty_string() {
    let t = TableRef::new("", "mytable");
    assert!(t.schema_id().is_none());
    assert_eq!(t.table_id().value, "mytable");
}

// --- from_names() ---

#[test]
fn we_can_create_table_ref_from_names_with_schema() {
    let t = TableRef::from_names(Some("sxt"), "blocks");
    assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("sxt"));
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn we_can_create_table_ref_from_names_without_schema() {
    let t = TableRef::from_names(None, "blocks");
    assert!(t.schema_id().is_none());
    assert_eq!(t.table_id().value, "blocks");
}

// --- from_idents() ---

#[test]
fn we_can_create_table_ref_from_idents_with_schema() {
    let t = TableRef::from_idents(Some(Ident::new("sxt")), Ident::new("blocks"));
    assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("sxt"));
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn we_can_create_table_ref_from_idents_without_schema() {
    let t = TableRef::from_idents(None, Ident::new("blocks"));
    assert!(t.schema_id().is_none());
    assert_eq!(t.table_id().value, "blocks");
}

// --- from_strs() ---

#[test]
fn we_can_create_table_ref_from_strs_with_one_component() {
    let t = TableRef::from_strs(&["blocks"]).unwrap();
    assert!(t.schema_id().is_none());
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn we_can_create_table_ref_from_strs_with_two_components() {
    let t = TableRef::from_strs(&["sxt", "blocks"]).unwrap();
    assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("sxt"));
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn we_get_error_from_strs_with_zero_components() {
    let result = TableRef::from_strs::<&str>(&[]);
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { .. })
    ));
}

#[test]
fn we_get_error_from_strs_with_three_or_more_components() {
    let result = TableRef::from_strs(&["a", "b", "c"]);
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { table_reference }) if table_reference == "a,b,c"
    ));
}

// --- TryFrom<&str> ---

#[test]
fn we_can_parse_table_only_from_str() {
    let t = TableRef::try_from("blocks").unwrap();
    assert!(t.schema_id().is_none());
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn we_can_parse_schema_dot_table_from_str() {
    let t = TableRef::try_from("sxt.blocks").unwrap();
    assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("sxt"));
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn we_get_error_parsing_three_part_dotted_str() {
    let result = TableRef::try_from("a.b.c");
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { table_reference }) if table_reference == "a.b.c"
    ));
}

#[test]
fn we_get_error_parsing_schema_dot_empty_table_from_str() {
    let result = TableRef::try_from("schema.");
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { table_reference }) if table_reference == "schema."
    ));
}

#[test]
fn we_get_error_parsing_empty_schema_dot_table_from_str() {
    let result = TableRef::try_from(".table");
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { table_reference }) if table_reference == ".table"
    ));
}

// --- FromStr ---

#[test]
fn we_can_parse_via_from_str() {
    let t: TableRef = "sxt.blocks".parse().unwrap();
    assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("sxt"));
    assert_eq!(t.table_id().value, "blocks");
}

#[test]
fn from_str_error_matches_try_from_error() {
    let result: Result<TableRef, _> = "a.b.c".parse();
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { .. })
    ));
}

// --- Display ---

#[test]
fn display_with_schema_uses_dot_notation() {
    let t = TableRef::new("sxt", "blocks");
    assert_eq!(t.to_string(), "sxt.blocks");
}

#[test]
fn display_without_schema_uses_table_name_only() {
    let t = TableRef::new("", "blocks");
    assert_eq!(t.to_string(), "blocks");
}

// --- Serde round-trip ---

#[test]
fn serde_round_trip_with_schema() {
    let original = TableRef::new("sxt", "blocks");
    let json = serde_json::to_string(&original).unwrap();
    let decoded: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn serde_round_trip_without_schema() {
    let original = TableRef::new("", "blocks");
    let json = serde_json::to_string(&original).unwrap();
    let decoded: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn serde_serializes_as_display_string() {
    let t = TableRef::new("sxt", "blocks");
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(json, "\"sxt.blocks\"");
}

#[test]
fn serde_deserialize_invalid_string_returns_error() {
    let result: Result<TableRef, _> = serde_json::from_str("\"a.b.c\"");
    assert!(result.is_err());
}
