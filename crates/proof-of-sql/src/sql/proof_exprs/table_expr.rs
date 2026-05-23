use crate::base::database::TableRef;
use serde::{Deserialize, Serialize};

/// Expression for an SQL table
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TableExpr {
    /// The `TableRef` for the table
    pub table_ref: TableRef,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table_expr(schema: &str, table: &str) -> TableExpr {
        TableExpr {
            table_ref: TableRef::new(schema, table),
        }
    }

    #[test]
    fn we_can_create_a_table_expr() {
        let expr = make_table_expr("sxt", "blocks");
        assert_eq!(expr.table_ref, TableRef::new("sxt", "blocks"));
    }

    #[test]
    fn we_can_clone_a_table_expr() {
        let expr = make_table_expr("sxt", "blocks");
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn we_can_check_equality_of_table_exprs() {
        let a = make_table_expr("sxt", "blocks");
        let b = make_table_expr("sxt", "blocks");
        let c = make_table_expr("sxt", "transactions");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn we_can_debug_print_a_table_expr() {
        let expr = make_table_expr("sxt", "blocks");
        let debug_str = format!("{expr:?}");
        assert!(debug_str.contains("TableExpr"));
    }

    #[test]
    fn we_can_serialize_and_deserialize_a_table_expr() {
        let expr = make_table_expr("sxt", "blocks");
        let serialized = serde_json::to_string(&expr).expect("serialization failed");
        let deserialized: TableExpr =
            serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn we_can_round_trip_table_expr_with_schema() {
        let expr = make_table_expr("myschema", "mytable");
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: TableExpr = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }
}
