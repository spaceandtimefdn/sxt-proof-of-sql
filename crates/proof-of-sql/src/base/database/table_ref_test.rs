use super::{ParseError, TableRef};
use alloc::string::ToString;
use sqlparser::ast::Ident;

#[test]
fn table_ref_new_treats_empty_schema_as_unqualified() {
    let table_ref = TableRef::new("", "orders");

    assert!(table_ref.schema_id().is_none());
    assert_eq!(table_ref.table_id().value, "orders");
    assert_eq!(table_ref.to_string(), "orders");
}

#[test]
fn table_ref_constructors_preserve_schema_and_table_idents() {
    let from_names = TableRef::from_names(Some("analytics"), "events");
    let from_idents = TableRef::from_idents(Some(Ident::new("analytics")), Ident::new("events"));

    assert_eq!(from_names, from_idents);
    assert_eq!(from_names.schema_id().unwrap().value, "analytics");
    assert_eq!(from_names.table_id().value, "events");
    assert_eq!(from_names.to_string(), "analytics.events");
}

#[test]
fn table_ref_parses_component_slices_and_dot_separated_strings() {
    assert_eq!(
        TableRef::from_strs(&["orders"]).unwrap(),
        TableRef::from_names(None, "orders")
    );
    assert_eq!(
        TableRef::from_strs(&["analytics", "events"]).unwrap(),
        TableRef::from_names(Some("analytics"), "events")
    );
    assert_eq!(
        TableRef::try_from("analytics.events").unwrap(),
        TableRef::from_names(Some("analytics"), "events")
    );
    assert_eq!(
        "orders".parse::<TableRef>().unwrap(),
        TableRef::from_names(None, "orders")
    );
}

#[test]
fn table_ref_rejects_references_with_too_many_components() {
    assert_eq!(
        TableRef::from_strs(&["catalog", "schema", "table"]),
        Err(ParseError::InvalidTableReference {
            table_reference: "catalog,schema,table".to_string(),
        })
    );
    assert_eq!(
        TableRef::try_from("catalog.schema.table"),
        Err(ParseError::InvalidTableReference {
            table_reference: "catalog.schema.table".to_string(),
        })
    );
}

#[test]
fn table_ref_round_trips_through_serde_as_display_string() {
    let qualified = TableRef::new("analytics", "events");
    let unqualified = TableRef::new("", "orders");

    assert_eq!(
        serde_json::to_string(&qualified).unwrap(),
        "\"analytics.events\""
    );
    assert_eq!(serde_json::to_string(&unqualified).unwrap(), "\"orders\"");
    assert_eq!(
        serde_json::from_str::<TableRef>("\"analytics.events\"").unwrap(),
        qualified
    );
    assert_eq!(
        serde_json::from_str::<TableRef>("\"orders\"").unwrap(),
        unqualified
    );
    assert!(serde_json::from_str::<TableRef>("\"catalog.schema.table\"").is_err());
}

#[test]
fn borrowed_table_ref_is_equivalent_to_matching_owned_key() {
    use indexmap::Equivalent;

    let owned = TableRef::new("analytics", "events");
    let borrowed = &owned;
    let matching = TableRef::new("analytics", "events");
    let different_schema = TableRef::new("warehouse", "events");
    let different_table = TableRef::new("analytics", "sessions");

    assert!(borrowed.equivalent(&matching));
    assert!(!borrowed.equivalent(&different_schema));
    assert!(!borrowed.equivalent(&different_table));
}
