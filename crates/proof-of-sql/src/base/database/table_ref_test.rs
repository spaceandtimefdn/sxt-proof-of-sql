use super::{ParseError, TableRef};
use alloc::string::ToString;
use indexmap::Equivalent;
use sqlparser::ast::Ident;

#[test]
fn we_can_create_table_refs_from_names_and_components() {
    let table_without_schema = TableRef::new("", "orders");
    assert_eq!(table_without_schema.schema_id(), None);
    assert_eq!(table_without_schema.table_id().value, "orders");
    assert_eq!(table_without_schema.to_string(), "orders");

    let table_with_schema = TableRef::from_names(Some("sales"), "orders");
    assert_eq!(table_with_schema.schema_id().unwrap().value, "sales");
    assert_eq!(table_with_schema.table_id().value, "orders");
    assert_eq!(table_with_schema.to_string(), "sales.orders");

    let table_from_idents =
        TableRef::from_idents(Some(Ident::new("warehouse")), Ident::new("items"));
    assert_eq!(table_from_idents.schema_id().unwrap().value, "warehouse");
    assert_eq!(table_from_idents.table_id().value, "items");
    assert_eq!(table_from_idents.to_string(), "warehouse.items");
}

#[test]
fn we_can_parse_and_reject_table_ref_components() {
    assert_eq!(
        TableRef::from_strs(&["sales", "orders"]).unwrap(),
        TableRef::new("sales", "orders")
    );
    assert_eq!(
        TableRef::from_strs(&["orders"]).unwrap(),
        TableRef::new("", "orders")
    );
    assert_eq!(
        "sales.orders".parse::<TableRef>().unwrap(),
        TableRef::new("sales", "orders")
    );

    let no_components: [&str; 0] = [];
    assert!(matches!(
        TableRef::from_strs(&no_components),
        Err(ParseError::InvalidTableReference {
            table_reference
        }) if table_reference.is_empty()
    ));
    assert!(matches!(
        TableRef::from_strs(&["too", "many", "parts"]),
        Err(ParseError::InvalidTableReference {
            table_reference
        }) if table_reference == "too,many,parts"
    ));
    assert!(matches!(
        "too.many.parts".parse::<TableRef>(),
        Err(ParseError::InvalidTableReference {
            table_reference
        }) if table_reference == "too.many.parts"
    ));
}

#[test]
fn we_can_serialize_and_deserialize_table_refs() {
    let table = TableRef::new("sales", "orders");

    assert_eq!(serde_json::to_string(&table).unwrap(), "\"sales.orders\"");
    assert_eq!(
        serde_json::from_str::<TableRef>("\"sales.orders\"").unwrap(),
        table
    );
    assert!(serde_json::from_str::<TableRef>("\"too.many.parts\"").is_err());
}

#[test]
fn we_can_compare_table_refs_as_equivalent_keys() {
    let table = TableRef::new("sales", "orders");
    let matching_table = TableRef::new("sales", "orders");
    let different_table = TableRef::new("finance", "orders");
    let key = &table;

    assert!(key.equivalent(&matching_table));
    assert!(!key.equivalent(&different_table));
}
