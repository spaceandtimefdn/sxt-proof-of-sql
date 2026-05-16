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

    #[test]
    fn we_can_round_trip_a_table_expr_through_json() {
        let table_expr = TableExpr {
            table_ref: TableRef::new("sxt", "orders"),
        };

        let serialized = serde_json::to_string(&table_expr).unwrap();
        assert_eq!(serialized, r#"{"table_ref":"sxt.orders"}"#);

        let deserialized: TableExpr = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, table_expr);
        assert_eq!(deserialized.table_ref.to_string(), "sxt.orders");
    }

    #[test]
    fn we_can_clone_a_table_expr_without_losing_the_table_ref() {
        let table_expr = TableExpr {
            table_ref: TableRef::new("", "line_items"),
        };

        let cloned = table_expr.clone();

        assert_eq!(cloned, table_expr);
        assert_eq!(cloned.table_ref.to_string(), "line_items");
    }
}
