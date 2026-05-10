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
    fn we_round_trip_table_expr_through_json() {
        let expr = TableExpr {
            table_ref: TableRef::new("sxt", "orders"),
        };

        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: TableExpr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, expr);
    }
}
