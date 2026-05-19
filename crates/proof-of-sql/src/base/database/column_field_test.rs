//! Tests for column_field.rs
use crate::base::database::{ColumnField, ColumnType};
use sqlparser::ast::Ident;

#[test]
fn column_field_new_and_accessors() {
    let cf = ColumnField::new(Ident::new("my_col"), ColumnType::BigInt);
    assert_eq!(cf.name(), Ident::new("my_col"));
    assert_eq!(cf.data_type(), ColumnType::BigInt);
}

#[test]
fn column_field_with_varchar_type() {
    let cf = ColumnField::new(Ident::new("name"), ColumnType::VarChar);
    assert_eq!(cf.name(), Ident::new("name"));
    assert_eq!(cf.data_type(), ColumnType::VarChar);
}

#[test]
fn column_field_with_decimal75_type() {
    use crate::base::math::decimal::Precision;
    let cf = ColumnField::new(Ident::new("price"), ColumnType::Decimal75(Precision::new(12).unwrap(), 2));
    assert_eq!(cf.data_type(), ColumnType::Decimal75(Precision::new(12).unwrap(), 2));
}

#[test]
fn column_field_equality() {
    let cf1 = ColumnField::new(Ident::new("a"), ColumnType::Int);
    let cf2 = ColumnField::new(Ident::new("a"), ColumnType::Int);
    assert_eq!(cf1, cf2);
}

#[test]
fn column_field_inequality_name() {
    let cf1 = ColumnField::new(Ident::new("a"), ColumnType::Int);
    let cf2 = ColumnField::new(Ident::new("b"), ColumnType::Int);
    assert_ne!(cf1, cf2);
}

#[test]
fn column_field_inequality_type() {
    let cf1 = ColumnField::new(Ident::new("a"), ColumnType::Int);
    let cf2 = ColumnField::new(Ident::new("a"), ColumnType::BigInt);
    assert_ne!(cf1, cf2);
}

#[test]
fn column_field_clone() {
    let cf = ColumnField::new(Ident::new("col"), ColumnType::Boolean);
    let cloned = cf.clone();
    assert_eq!(cf, cloned);
}

#[test]
fn column_field_serde_roundtrip() {
    let cf = ColumnField::new(Ident::new("col"), ColumnType::Int128);
    let json = serde_json::to_string(&cf).unwrap();
    let deserialized: ColumnField = serde_json::from_str(&json).unwrap();
    assert_eq!(cf, deserialized);
}

#[test]
fn column_field_hash_consistency() {
    use core::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let cf1 = ColumnField::new(Ident::new("x"), ColumnType::SmallInt);
    let cf2 = ColumnField::new(Ident::new("x"), ColumnType::SmallInt);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    cf1.hash(&mut hasher1);
    cf2.hash(&mut hasher2);
    assert_eq!(hasher1.finish(), hasher2.finish());
}
