use super::TableExpr;
use crate::base::database::TableRef;

#[test]
fn table_expr_preserves_table_ref_when_cloned() {
    let table_expr = TableExpr {
        table_ref: TableRef::new("public", "orders"),
    };

    let cloned = table_expr.clone();

    assert_eq!(cloned, table_expr);
    assert_eq!(cloned.table_ref.to_string(), "public.orders");
}

#[test]
fn table_expr_serializes_through_table_ref() {
    let table_expr = TableExpr {
        table_ref: TableRef::new("", "orders"),
    };

    let serialized = serde_json::to_string(&table_expr).unwrap();
    let deserialized: TableExpr = serde_json::from_str(&serialized).unwrap();

    assert_eq!(serialized, r#"{"table_ref":"orders"}"#);
    assert_eq!(deserialized, table_expr);
}
