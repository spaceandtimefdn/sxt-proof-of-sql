use crate::base::database::{ParseError, TableRef};
use indexmap::Equivalent;
use sqlparser::ast::Ident;

#[test]
fn new_omits_empty_schema_name() {
    let table = TableRef::new("", "people");

    assert_eq!(table.schema_id(), None);
    assert_eq!(table.table_id().value, "people");
    assert_eq!(table.to_string(), "people");
}

#[test]
fn from_strs_supports_one_or_two_components() {
    let without_schema = TableRef::from_strs(&["people"]).unwrap();
    let with_schema = TableRef::from_strs(&["public", "people"]).unwrap();

    assert_eq!(without_schema.schema_id(), None);
    assert_eq!(without_schema.table_id().value, "people");
    assert_eq!(
        with_schema.schema_id().map(|ident| ident.value.as_str()),
        Some("public")
    );
    assert_eq!(with_schema.table_id().value, "people");
}

#[test]
fn from_strs_rejects_invalid_component_counts() {
    let err = TableRef::from_strs(&["db", "public", "people"]).unwrap_err();

    assert_eq!(
        err,
        ParseError::InvalidTableReference {
            table_reference: "db,public,people".to_string(),
        }
    );
}

#[test]
fn try_from_str_round_trips_through_display_and_serde() {
    let table = TableRef::try_from("public.people").unwrap();

    assert_eq!(table.to_string(), "public.people");
    let serialized = serde_json::to_string(&table).unwrap();
    assert_eq!(serialized, "\"public.people\"");

    let decoded: TableRef = serde_json::from_str(&serialized).unwrap();
    assert_eq!(decoded, table);
}

#[test]
fn try_from_str_rejects_too_many_components() {
    let err = TableRef::try_from("db.public.people").unwrap_err();

    assert_eq!(
        err,
        ParseError::InvalidTableReference {
            table_reference: "db.public.people".to_string(),
        }
    );
}

#[test]
fn from_idents_and_equivalent_consider_schema_and_table_name() {
    let left = TableRef::from_idents(Some(Ident::new("public")), Ident::new("people"));
    let same = TableRef::from_names(Some("public"), "people");
    let different_schema = TableRef::from_names(Some("private"), "people");
    let different_table = TableRef::from_names(Some("public"), "orders");

    assert!((&left).equivalent(&same));
    assert!(!(&left).equivalent(&different_schema));
    assert!(!(&left).equivalent(&different_table));
}
