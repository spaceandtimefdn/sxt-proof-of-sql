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
    fn table_expr_preserves_table_ref_through_clone_and_serde() {
        let table_ref = TableRef::new("public", "orders");
        let expr = TableExpr {
            table_ref: table_ref.clone(),
        };

        assert_eq!(expr.clone(), expr);
        assert_eq!(expr.table_ref, table_ref);

        let json = serde_json::to_string(&expr).unwrap();
        let round_trip: TableExpr = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, expr);
    }
}
