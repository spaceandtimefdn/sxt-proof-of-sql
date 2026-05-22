use super::{ColumnField, ColumnType};
use sqlparser::ast::Ident;

#[test]
fn we_can_create_a_column_field_and_read_back_its_name_and_type() {
    let field = ColumnField::new(Ident::new("amount"), ColumnType::BigInt);
    assert_eq!(field.name(), Ident::new("amount"));
    assert_eq!(field.data_type(), ColumnType::BigInt);
}

#[test]
fn column_fields_are_equal_only_when_both_name_and_type_match() {
    let base = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
    let same = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
    let other_name = ColumnField::new(Ident::new("b"), ColumnType::BigInt);
    let other_type = ColumnField::new(Ident::new("a"), ColumnType::Boolean);

    assert_eq!(base, same);
    assert_ne!(base, other_name);
    assert_ne!(base, other_type);
}

#[test]
fn a_cloned_column_field_is_equal_to_the_original() {
    let field = ColumnField::new(Ident::new("flag"), ColumnType::Boolean);
    assert_eq!(field.clone(), field);
}

#[test]
fn a_column_field_round_trips_through_serde_json() {
    let field = ColumnField::new(Ident::new("label"), ColumnType::Boolean);
    let json = serde_json::to_string(&field).expect("ColumnField should serialize");
    let decoded: ColumnField = serde_json::from_str(&json).expect("ColumnField should deserialize");
    assert_eq!(field, decoded);
}
