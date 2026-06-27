use crate::base::database::{ParseError, TableRef};
use alloc::{format, string::ToString, vec::Vec};
use core::str::FromStr;
use indexmap::Equivalent;
use sqlparser::ast::Ident;

#[test]
fn we_can_create_a_table_ref_with_schema_and_table() {
    let table_ref = TableRef::new("sch", "tab");
    assert_eq!(table_ref.schema_id(), Some(&Ident::new("sch")));
    assert_eq!(table_ref.table_id(), &Ident::new("tab"));
    assert_eq!(format!("{table_ref}"), "sch.tab");
}

#[test]
fn we_can_create_a_table_ref_without_schema_using_new() {
    let table_ref = TableRef::new("", "tab");
    assert_eq!(table_ref.schema_id(), None);
    assert_eq!(table_ref.table_id(), &Ident::new("tab"));
    assert_eq!(format!("{table_ref}"), "tab");
    assert_eq!(table_ref, TableRef::from_names(None, "tab"));
}

#[test]
fn we_can_create_a_table_ref_from_names() {
    let with_schema = TableRef::from_names(Some("sch"), "tab");
    assert_eq!(with_schema.schema_id(), Some(&Ident::new("sch")));
    assert_eq!(with_schema.table_id(), &Ident::new("tab"));

    let without_schema = TableRef::from_names(None, "tab");
    assert_eq!(without_schema.schema_id(), None);
    assert_eq!(without_schema.table_id(), &Ident::new("tab"));
}

#[test]
fn we_can_create_a_table_ref_from_idents() {
    let with_schema = TableRef::from_idents(Some(Ident::new("sch")), Ident::new("tab"));
    assert_eq!(with_schema.schema_id(), Some(&Ident::new("sch")));
    assert_eq!(with_schema.table_id(), &Ident::new("tab"));
    assert_eq!(format!("{with_schema}"), "sch.tab");

    let without_schema = TableRef::from_idents(None, Ident::new("tab"));
    assert_eq!(without_schema.schema_id(), None);
    assert_eq!(format!("{without_schema}"), "tab");
}

#[test]
fn we_can_create_a_table_ref_from_one_or_two_strs() {
    let table_only = TableRef::from_strs(&["tab"]).unwrap();
    assert_eq!(table_only, TableRef::from_names(None, "tab"));

    let with_schema = TableRef::from_strs(&["sch", "tab"]).unwrap();
    assert_eq!(with_schema, TableRef::from_names(Some("sch"), "tab"));
}

#[test]
fn we_cannot_create_a_table_ref_from_zero_or_three_strs() {
    let empty: &[&str] = &[];
    let empty_err = TableRef::from_strs(empty).unwrap_err();
    assert!(matches!(
        empty_err,
        ParseError::InvalidTableReference { ref table_reference } if table_reference.is_empty()
    ));

    let three_err = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();
    assert!(matches!(
        three_err,
        ParseError::InvalidTableReference { ref table_reference } if table_reference == "a,b,c"
    ));
}

#[test]
fn we_can_parse_a_table_ref_from_a_dotted_string() {
    let table_only = TableRef::try_from("tab").unwrap();
    assert_eq!(table_only.schema_id(), None);
    assert_eq!(table_only.table_id(), &Ident::new("tab"));

    let with_schema = TableRef::try_from("sch.tab").unwrap();
    assert_eq!(with_schema.schema_id(), Some(&Ident::new("sch")));
    assert_eq!(with_schema.table_id(), &Ident::new("tab"));

    let from_str = TableRef::from_str("sch.tab").unwrap();
    assert_eq!(from_str, with_schema);
}

#[test]
fn we_cannot_parse_a_table_ref_with_too_many_components() {
    let err = TableRef::try_from("a.b.c").unwrap_err();
    assert!(matches!(
        err,
        ParseError::InvalidTableReference { ref table_reference } if table_reference == "a.b.c"
    ));
    assert_eq!(format!("{err}"), "Invalid table reference: a.b.c");
}

#[test]
fn we_can_compare_table_refs_with_equivalent() {
    let table_ref = TableRef::new("sch", "tab");
    let same = TableRef::new("sch", "tab");
    let different_schema = TableRef::new("other", "tab");
    let different_table = TableRef::new("sch", "other");

    assert!((&table_ref).equivalent(&same));
    assert!(!(&table_ref).equivalent(&different_schema));
    assert!(!(&table_ref).equivalent(&different_table));
}

#[test]
fn we_can_serialize_and_deserialize_a_table_ref() {
    let with_schema = TableRef::new("sch", "tab");
    let serialized = serde_json::to_string(&with_schema).unwrap();
    assert_eq!(serialized, r#""sch.tab""#);
    let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, with_schema);

    let without_schema: TableRef = serde_json::from_str(r#""tab""#).unwrap();
    assert_eq!(without_schema, TableRef::from_names(None, "tab"));
}

#[test]
fn we_cannot_deserialize_an_invalid_table_ref() {
    let result: Result<TableRef, _> = serde_json::from_str(r#""a.b.c""#);
    assert!(result.is_err());
}

#[test]
fn we_can_round_trip_a_table_ref_through_display_and_parse() {
    let originals: Vec<TableRef> = alloc::vec![
        TableRef::new("sch", "tab"),
        TableRef::from_names(None, "tab"),
    ];
    for original in originals {
        let displayed = original.to_string();
        let reparsed = TableRef::from_str(&displayed).unwrap();
        assert_eq!(reparsed, original);
    }
}
