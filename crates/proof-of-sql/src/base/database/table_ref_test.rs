use super::{ParseError, TableRef};
use alloc::string::ToString;
use sqlparser::ast::Ident;

#[test]
fn table_ref_new_handles_schema_and_no_schema() {
    let with_schema = TableRef::new("analytics", "events");
    assert_eq!(with_schema.schema_id(), Some(&Ident::new("analytics")));
    assert_eq!(with_schema.table_id(), &Ident::new("events"));
    assert_eq!(with_schema.to_string(), "analytics.events");

    let without_schema = TableRef::new("", "events");
    assert_eq!(without_schema.schema_id(), None);
    assert_eq!(without_schema.table_id(), &Ident::new("events"));
    assert_eq!(without_schema.to_string(), "events");
}

#[test]
fn table_ref_parse_helpers_cover_success_and_errors() {
    let one = TableRef::from_strs(&["events"]).unwrap();
    assert_eq!(one.schema_id(), None);
    assert_eq!(one.table_id(), &Ident::new("events"));

    let two = TableRef::from_strs(&["analytics", "events"]).unwrap();
    assert_eq!(two.schema_id(), Some(&Ident::new("analytics")));
    assert_eq!(two.table_id(), &Ident::new("events"));

    let err = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();
    assert_eq!(
        err,
        ParseError::InvalidTableReference {
            table_reference: "a,b,c".to_string(),
        }
    );

    let parsed = TableRef::try_from("analytics.events").unwrap();
    assert_eq!(parsed, two);

    let bad = TableRef::try_from("a.b.c").unwrap_err();
    assert_eq!(
        bad,
        ParseError::InvalidTableReference {
            table_reference: "a.b.c".to_string(),
        }
    );
}

#[test]
fn table_ref_serde_roundtrip_and_invalid_input() {
    let table = TableRef::from_names(Some("analytics"), "events");
    let json = serde_json::to_string(&table).unwrap();
    assert_eq!(json, "\"analytics.events\"");

    let decoded: TableRef = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, table);

    let decode_err = serde_json::from_str::<TableRef>("\"a.b.c\"").unwrap_err();
    assert!(decode_err.to_string().contains("Invalid table reference"));
}
