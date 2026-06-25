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
    use super::TableExpr;
    use crate::base::database::TableRef;

    #[test]
    fn table_expr_construction_stores_table_ref() {
        let table_ref = TableRef::new("my_schema", "my_table");
        let expr = TableExpr { table_ref: table_ref.clone() };
        assert_eq!(expr.table_ref, table_ref);
    }

    #[test]
    fn table_expr_implements_partial_eq_equal() {
        let tr = TableRef::new("schema", "table");
        let a = TableExpr { table_ref: tr.clone() };
        let b = TableExpr { table_ref: tr };
        assert_eq!(a, b);
    }

    #[test]
    fn table_expr_implements_partial_eq_not_equal() {
        let a = TableExpr { table_ref: TableRef::new("schema", "table_a") };
        let b = TableExpr { table_ref: TableRef::new("schema", "table_b") };
        assert_ne!(a, b);
    }

    #[test]
    fn table_expr_debug_output_contains_struct_name() {
        let expr = TableExpr { table_ref: TableRef::new("s", "t") };
        let debug = format!("{:?}", expr);
        assert!(debug.contains("TableExpr"));
    }

    #[test]
    fn table_expr_clone_equals_original() {
        let expr = TableExpr { table_ref: TableRef::new("schema", "table") };
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }
}
