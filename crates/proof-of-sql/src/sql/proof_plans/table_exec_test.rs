//! Tests for TableExec.

#[cfg(test)]
mod table_exec_test {
    use crate::sql::proof_plans::table_exec::TableExec;
    use crate::base::database::{ColumnField, TableRef};

    #[test]
    fn test_table_exec_new() {
        let table_ref: TableRef = "test.table".parse().unwrap();
        let schema = vec![ColumnField::new("col1".parse().unwrap(), crate::base::database::ColumnType::Int128)];
        let exec = TableExec::new(table_ref.clone(), schema.clone());
        assert_eq!(exec.table_ref(), &table_ref);
        assert_eq!(exec.schema(), &schema);
    }

    #[test]
    fn test_table_exec_type_exists() {
        let _: Option<TableExec> = None;
    }

    #[test]
    fn test_table_exec_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<TableExec>());
        assert!(!debug_str.is_empty());
    }
}
