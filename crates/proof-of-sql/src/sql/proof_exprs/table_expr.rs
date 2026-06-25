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

    fn make_expr() -> TableExpr {
        TableExpr { table_ref: TableRef::new("schema", "table") }
    }

    #[test]
    fn table_expr_stores_table_ref() {
        let e = make_expr();
        assert_eq!(e.table_ref, TableRef::new("schema", "table"));
    }

    #[test]
    fn table_expr_equality() {
        assert_eq!(make_expr(), make_expr());
    }

    #[test]
    fn table_expr_inequality() {
        let a = TableExpr { table_ref: TableRef::new("s", "t1") };
        let b = TableExpr { table_ref: TableRef::new("s", "t2") };
        assert_ne!(a, b);
    }

    #[test]
    fn table_expr_clone_equals_original() {
        let e = make_expr();
        assert_eq!(e.clone(), e);
    }

    #[test]
    fn table_expr_is_debug_formattable() {
        let e = make_expr();
        let s = alloc::format!("{e:?}");
        assert!(s.contains("TableExpr"));
    }

    #[test]
    fn table_expr_without_schema() {
        let e = TableExpr { table_ref: TableRef::new("", "mytable") };
        assert_eq!(alloc::format!("{}", e.table_ref), "mytable");
    }
}
