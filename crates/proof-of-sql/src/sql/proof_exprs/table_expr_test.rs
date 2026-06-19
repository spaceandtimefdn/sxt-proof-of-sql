//! Tests for [`TableExpr`], the AST node representing a reference to a SQL table
//! in a provable query plan.
//!
//! These tests cover the previously-untested `sql/proof_exprs/table_expr.rs` module
//! as part of the repository-wide test coverage effort tracked in issue #560.

use crate::base::database::TableRef;
use crate::sql::proof_exprs::TableExpr;

#[test]
fn we_can_construct_a_table_expression_with_a_schema_and_table() {
    let table_ref = TableRef::new("public", "users");
    let expr = TableExpr {
        table_ref: table_ref.clone(),
    };

    assert_eq!(expr.table_ref, table_ref);
    assert_eq!(expr.table_ref.table_id().to_string(), "users");
    assert_eq!(
        expr.table_ref
            .schema_id()
            .map(ToString::to_string)
            .as_deref(),
        Some("public")
    );
}

#[test]
fn we_can_construct_a_table_expression_without_a_schema() {
    let table_ref = TableRef::new("", "blocks");
    let expr = TableExpr { table_ref };

    assert!(expr.table_ref.schema_id().is_none());
    assert_eq!(expr.table_ref.table_id().to_string(), "blocks");
}

#[test]
fn we_can_clone_a_table_expression() {
    let table_ref = TableRef::new("sxt", "transactions");
    let expr = TableExpr { table_ref };
    let cloned = expr.clone();

    assert_eq!(expr, cloned);
    // Cloned value is structurally identical, not just equal.
    assert_eq!(
        expr.table_ref.table_id().to_string(),
        cloned.table_ref.table_id().to_string()
    );
}

#[test]
fn we_can_compare_table_expressions_for_equality_and_inequality() {
    let a = TableExpr {
        table_ref: TableRef::new("public", "users"),
    };
    let b = TableExpr {
        table_ref: TableRef::new("public", "users"),
    };
    let c = TableExpr {
        table_ref: TableRef::new("public", "accounts"),
    };
    let d = TableExpr {
        table_ref: TableRef::new("private", "users"),
    };

    // PartialEq / Eq reflexivity and symmetry.
    assert_eq!(a, b);
    assert_eq!(b, a);
    assert_eq!(a, a);
    // Different table name -> not equal.
    assert_ne!(a, c);
    // Different schema name -> not equal.
    assert_ne!(a, d);
}

#[test]
fn we_can_format_a_table_expression_with_debug() {
    let expr = TableExpr {
        table_ref: TableRef::new("public", "orders"),
    };
    let rendered = format!("{expr:?}");

    // Debug output mentions both the wrapper type and the inner table.
    assert!(
        rendered.contains("TableExpr"),
        "Debug output missing type name: {rendered}"
    );
    assert!(
        rendered.contains("orders"),
        "Debug output missing table name: {rendered}"
    );
}

#[test]
fn we_can_serialize_and_deserialize_a_table_expression_via_json() {
    let original = TableExpr {
        table_ref: TableRef::new("public", "users"),
    };
    let json = serde_json::to_string(&original).expect("serialization must succeed");
    let decoded: TableExpr = serde_json::from_str(&json).expect("deserialization must succeed");

    assert_eq!(decoded, original);
    assert_eq!(decoded.table_ref.table_id().to_string(), "users");
}

#[test]
fn we_can_serialize_and_deserialize_a_table_expression_without_schema_via_json() {
    // `TableRef::new("", ...)` produces a schema-less reference; this is the
    // form that appears in queries with no default schema. It still must
    // roundtrip cleanly through serde.
    let original = TableExpr {
        table_ref: TableRef::new("", "events"),
    };
    let json = serde_json::to_string(&original).expect("serialization must succeed");
    let decoded: TableExpr = serde_json::from_str(&json).expect("deserialization must succeed");

    assert_eq!(decoded, original);
    assert!(decoded.table_ref.schema_id().is_none());
}
