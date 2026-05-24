#[cfg(test)]
mod database_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::database::{
            Column, ColumnType, ColumnRef, ColumnField, ColumnarValue, ColumnOperationError,
            MetadataAccessor, SchemaAccessor, DataAccessor, OwnedTable, OwnedColumn,
            Table, TableOptions, TestAccessor, TableEvaluation,
        };
        assert!(true);
    }
}
