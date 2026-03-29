use super::QueryContext;
use crate::base::database::ColumnType;

#[test]
fn test_query_context_new_is_empty() {
    let ctx = QueryContext::new();
    assert!(ctx.get_column_references().is_empty());
    assert!(ctx.get_table_references().is_empty());
}

#[test]
fn test_query_context_push_column() {
    let mut ctx = QueryContext::new();
    ctx.push_column_ref("col_a".into(), ColumnType::BigInt);
    assert_eq!(ctx.get_column_references().len(), 1);
}

#[test]
fn test_query_context_push_duplicate_column_deduplicates() {
    let mut ctx = QueryContext::new();
    ctx.push_column_ref("col_a".into(), ColumnType::BigInt);
    ctx.push_column_ref("col_a".into(), ColumnType::BigInt);
    // Should not contain duplicates
    assert_eq!(ctx.get_column_references().len(), 1);
}

#[test]
fn test_query_context_push_table_ref() {
    let mut ctx = QueryContext::new();
    ctx.push_table_ref("schema.table".parse().unwrap());
    assert_eq!(ctx.get_table_references().len(), 1);
}

#[test]
fn test_query_context_multiple_columns() {
    let mut ctx = QueryContext::new();
    ctx.push_column_ref("a".into(), ColumnType::BigInt);
    ctx.push_column_ref("b".into(), ColumnType::Int128);
    ctx.push_column_ref("c".into(), ColumnType::VarChar);
    assert_eq!(ctx.get_column_references().len(), 3);
}
