use super::{ParseError, TableRef};
use alloc::string::ToString;
use indexmap::Equivalent;
use sqlparser::ast::Ident;

#[test]
fn we_can_create_table_refs_without_schema() {
    let table_ref = TableRef::new("", "sessions");

    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id().value, "sessions");
    assert_eq!(table_ref.to_string(), "sessions");
}

#[test]
fn we_can_create_table_refs_from_idents() {
    let table_ref = TableRef::from_idents(Some(Ident::new("analytics")), Ident::new("events"));

    assert_eq!(table_ref.schema_id().unwrap().value, "analytics");
    assert_eq!(table_ref.table_id().value, "events");
    assert_eq!(table_ref.to_string(), "analytics.events");
}

#[test]
fn we_can_parse_table_refs_from_component_slices() {
    let table_only = TableRef::from_strs(&["events"]).unwrap();
    let schema_and_table = TableRef::from_strs(&["analytics", "events"]).unwrap();

    assert_eq!(table_only, TableRef::from_names(None, "events"));
    assert_eq!(
        schema_and_table,
        TableRef::from_names(Some("analytics"), "events")
    );
}

#[test]
fn we_cannot_parse_table_refs_with_too_many_components() {
    let result = TableRef::from_strs(&["warehouse", "analytics", "events"]);

    assert_eq!(
        result,
        Err(ParseError::InvalidTableReference {
            table_reference: "warehouse,analytics,events".to_string()
        })
    );
}

#[test]
fn we_can_try_from_table_ref_strings() {
    assert_eq!(
        TableRef::try_from("events").unwrap(),
        TableRef::from_names(None, "events")
    );
    assert_eq!(
        TableRef::try_from("analytics.events").unwrap(),
        TableRef::from_names(Some("analytics"), "events")
    );
}

#[test]
fn we_cannot_try_from_table_ref_strings_with_too_many_components() {
    assert_eq!(
        TableRef::try_from("warehouse.analytics.events"),
        Err(ParseError::InvalidTableReference {
            table_reference: "warehouse.analytics.events".to_string()
        })
    );
}

#[test]
fn table_ref_equivalence_matches_schema_and_table() {
    let table_ref = TableRef::from_names(Some("analytics"), "events");
    let same = TableRef::from_names(Some("analytics"), "events");
    let different_schema = TableRef::from_names(Some("public"), "events");
    let different_table = TableRef::from_names(Some("analytics"), "sessions");

    assert!((&table_ref).equivalent(&same));
    assert!(!(&table_ref).equivalent(&different_schema));
    assert!(!(&table_ref).equivalent(&different_table));
}

#[test]
fn table_refs_serialize_as_dot_separated_strings() {
    let schema_and_table = TableRef::from_names(Some("analytics"), "events");
    let table_only = TableRef::from_names(None, "events");

    assert_eq!(
        serde_json::to_string(&schema_and_table).unwrap(),
        "\"analytics.events\""
    );
    assert_eq!(serde_json::to_string(&table_only).unwrap(), "\"events\"");
}

#[test]
fn table_refs_deserialize_from_dot_separated_strings() {
    let schema_and_table: TableRef = serde_json::from_str("\"analytics.events\"").unwrap();
    let table_only: TableRef = serde_json::from_str("\"events\"").unwrap();

    assert_eq!(
        schema_and_table,
        TableRef::from_names(Some("analytics"), "events")
    );
    assert_eq!(table_only, TableRef::from_names(None, "events"));
}

#[test]
fn table_ref_deserialization_rejects_invalid_table_references() {
    let result = serde_json::from_str::<TableRef>("\"warehouse.analytics.events\"");

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid table reference: warehouse.analytics.events"));
}
