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
    fn table_expr_round_trips_through_json() {
        let expr = TableExpr {
            table_ref: TableRef::new("sxt", "orders"),
        };

        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: TableExpr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, expr);
    }

    #[test]
    fn table_expr_clone_preserves_table_ref() {
        let expr = TableExpr {
            table_ref: TableRef::new("analytics", "events"),
        };

        let cloned = expr.clone();

        assert_eq!(cloned.table_ref, expr.table_ref);
    }
}
