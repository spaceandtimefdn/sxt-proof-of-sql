use crate::base::database::{ParseError, TableRef};
use alloc::string::ToString;
use core::str::FromStr;
use sqlparser::ast::Ident;

#[test]
fn we_can_create_table_ref_with_schema() {
    let table_ref = TableRef::new("myschema", "mytable");
    assert_eq!(
        table_ref.schema_id(),
        Some(&Ident::new("myschema".to_string()))
    );
    assert_eq!(table_ref.table_id(), &Ident::new("mytable".to_string()));
}

#[test]
fn we_can_create_table_ref_without_schema() {
    let table_ref = TableRef::new("", "mytable");
    assert_eq!(table_ref.schema_id(), None);
    assert_eq!(table_ref.table_id(), &Ident::new("mytable".to_string()));
}

#[test]
fn we_can_create_table_ref_from_names() {
    let table_ref = TableRef::from_names(Some("public"), "users");
    assert_eq!(
        table_ref.schema_id(),
        Some(&Ident::new("public".to_string()))
    );
    assert_eq!(table_ref.table_id(), &Ident::new("users".to_string()));
}

#[test]
fn we_can_create_table_ref_from_names_without_schema() {
    let table_ref = TableRef::from_names(None, "users");
    assert_eq!(table_ref.schema_id(), None);
    assert_eq!(table_ref.table_id(), &Ident::new("users".to_string()));
}

#[test]
fn we_can_create_table_ref_from_idents() {
    let schema = Some(Ident::new("myschema".to_string()));
    let table = Ident::new("mytable".to_string());
    let table_ref = TableRef::from_idents(schema.clone(), table.clone());
    assert_eq!(table_ref.schema_id(), schema.as_ref());
    assert_eq!(table_ref.table_id(), &table);
}

#[test]
fn we_can_create_table_ref_from_single_component_strs() {
    let table_ref = TableRef::from_strs(&["mytable"]).unwrap();
    assert_eq!(table_ref.schema_id(), None);
    assert_eq!(table_ref.table_id(), &Ident::new("mytable".to_string()));
}

#[test]
fn we_can_create_table_ref_from_two_component_strs() {
    let table_ref = TableRef::from_strs(&["myschema", "mytable"]).unwrap();
    assert_eq!(
        table_ref.schema_id(),
        Some(&Ident::new("myschema".to_string()))
    );
    assert_eq!(table_ref.table_id(), &Ident::new("mytable".to_string()));
}

#[test]
fn we_cannot_create_table_ref_from_three_component_strs() {
    let result = TableRef::from_strs(&["a", "b", "c"]);
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { .. })
    ));
}

#[test]
fn we_cannot_create_table_ref_from_empty_strs() {
    let result = TableRef::from_strs::<&str>(&[]);
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { .. })
    ));
}

#[test]
fn we_can_try_from_dot_separated_str() {
    let table_ref = TableRef::try_from("myschema.mytable").unwrap();
    assert_eq!(
        table_ref.schema_id(),
        Some(&Ident::new("myschema".to_string()))
    );
    assert_eq!(table_ref.table_id(), &Ident::new("mytable".to_string()));
}

#[test]
fn we_can_try_from_single_str() {
    let table_ref = TableRef::try_from("mytable").unwrap();
    assert_eq!(table_ref.schema_id(), None);
    assert_eq!(table_ref.table_id(), &Ident::new("mytable".to_string()));
}

#[test]
fn we_cannot_try_from_triple_dot_str() {
    let result = TableRef::try_from("a.b.c");
    assert!(matches!(
        result,
        Err(ParseError::InvalidTableReference { .. })
    ));
}

#[test]
fn we_can_parse_table_ref_from_str() {
    let table_ref: TableRef = "myschema.mytable".parse().unwrap();
    assert_eq!(
        table_ref.schema_id(),
        Some(&Ident::new("myschema".to_string()))
    );
}

#[test]
fn table_ref_display_with_schema() {
    let table_ref = TableRef::new("myschema", "mytable");
    assert_eq!(table_ref.to_string(), "myschema.mytable");
}

#[test]
fn table_ref_display_without_schema() {
    let table_ref = TableRef::new("", "mytable");
    assert_eq!(table_ref.to_string(), "mytable");
}

#[test]
fn table_ref_serialize_deserialize_roundtrip() {
    let table_ref = TableRef::new("myschema", "mytable");
    let serialized = serde_json::to_string(&table_ref).unwrap();
    assert_eq!(serialized, "\"myschema.mytable\"");

    let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, table_ref);
}

#[test]
fn table_ref_serialize_deserialize_roundtrip_no_schema() {
    let table_ref = TableRef::new("", "mytable");
    let serialized = serde_json::to_string(&table_ref).unwrap();
    assert_eq!(serialized, "\"mytable\"");

    let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, table_ref);
}

#[test]
fn table_ref_deserialize_invalid_fails() {
    let result: Result<TableRef, _> = serde_json::from_str("\"a.b.c\"");
    assert!(result.is_err());
}

#[test]
fn table_ref_clone_eq() {
    let a = TableRef::new("schema", "table");
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn table_ref_equivalent_trait() {
    use indexmap::Equivalent;
    let a = TableRef::new("schema", "table");
    let b = TableRef::new("schema", "table");
    let a_ref = &a;
    assert!(a_ref.equivalent(&b));
}

#[test]
fn table_ref_equivalent_trait_different_schema() {
    use indexmap::Equivalent;
    let a = TableRef::new("schema1", "table");
    let b = TableRef::new("schema2", "table");
    let a_ref = &a;
    assert!(!a_ref.equivalent(&b));
}

#[test]
fn table_ref_equivalent_trait_different_table() {
    use indexmap::Equivalent;
    let a = TableRef::new("schema", "table1");
    let b = TableRef::new("schema", "table2");
    let a_ref = &a;
    assert!(!a_ref.equivalent(&b));
}
