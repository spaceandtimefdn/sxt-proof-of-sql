use super::{ColumnField, ColumnType};
use sqlparser::ast::Ident;

#[test]
fn column_field_preserves_name_and_type() {
    let name = Ident::new("total");
    let field = ColumnField::new(name.clone(), ColumnType::BigInt);

    assert_eq!(field.name(), name);
    assert_eq!(field.data_type(), ColumnType::BigInt);
}

#[test]
fn column_field_json_round_trip_preserves_quoted_name() {
    let field = ColumnField::new(Ident::with_quote('"', "Mixed Name"), ColumnType::VarChar);

    let json = serde_json::to_string(&field).unwrap();
    let deserialized: ColumnField = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized, field);
}
