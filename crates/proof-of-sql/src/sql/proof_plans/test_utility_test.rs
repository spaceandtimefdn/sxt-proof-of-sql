#[cfg(test)]
mod test_utility_test {
    use crate::sql::proof_plans::test_utility::{
        column_field, empty_exec, table_exec, projection,
    };
    use crate::base::database::{ColumnType, TableRef};
    
    #[test]
    fn test_column_field() {
        let field = column_field("test", ColumnType::Int);
        assert_eq!(field.name().as_ref(), "test");
    }
    
    #[test]
    fn test_empty_exec() {
        let exec = empty_exec();
        // Just verify it can be created
        assert!(true);
    }
}
