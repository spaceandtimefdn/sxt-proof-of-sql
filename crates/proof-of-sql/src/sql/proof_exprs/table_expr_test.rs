//! Tests for TableExpr.

#[cfg(test)]
mod table_expr_test {
    use crate::sql::proof_exprs::table_expr::TableExpr;
    use crate::base::database::TableRef;

    #[test]
    fn test_table_expr_new() {
        let table_ref: TableRef = "test.table".parse().unwrap();
        let expr = TableExpr { table_ref };
        assert_eq!(expr.table_ref, table_ref);
    }

    #[test]
    fn test_table_expr_type_exists() {
        let _: Option<TableExpr> = None;
    }

    #[test]
    fn test_table_expr_debug() {
        let table_ref: TableRef = "test.table".parse().unwrap();
        let expr = TableExpr { table_ref };
        let debug_str = format!("{:?}", expr);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_table_expr_partial_eq() {
        let table_ref1: TableRef = "test.table1".parse().unwrap();
        let table_ref2: TableRef = "test.table1".parse().unwrap();
        let table_ref3: TableRef = "test.table2".parse().unwrap();
        let expr1 = TableExpr { table_ref: table_ref1 };
        let expr2 = TableExpr { table_ref: table_ref2 };
        let expr3 = TableExpr { table_ref: table_ref3 };
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }
}
