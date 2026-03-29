/// Tests for ColumnRef to improve coverage of construction and accessors
#[cfg(test)]
mod tests {
    use crate::base::database::{ColumnRef, ColumnType, TableRef};

    fn make_table_ref(schema: &str, name: &str) -> TableRef {
        let resource_id = format!("{schema}.{name}").parse().unwrap();
        TableRef::new(resource_id)
    }

    #[test]
    fn test_column_ref_new_and_accessors() {
        let table = make_table_ref("sxt", "employees");
        let col_id: proof_of_sql_parser::Identifier = "salary".parse().unwrap();
        let col_ref = ColumnRef::new(table.clone(), col_id, ColumnType::BigInt);

        assert_eq!(col_ref.table_ref(), &table);
        assert_eq!(col_ref.column_id().name(), "salary");
        assert_eq!(col_ref.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_column_ref_equality() {
        let table = make_table_ref("sxt", "employees");
        let col_id: proof_of_sql_parser::Identifier = "id".parse().unwrap();
        let c1 = ColumnRef::new(table.clone(), col_id.clone(), ColumnType::Int);
        let c2 = ColumnRef::new(table.clone(), col_id, ColumnType::Int);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_column_ref_inequality_different_type() {
        let table = make_table_ref("sxt", "t");
        let col_id: proof_of_sql_parser::Identifier = "col".parse().unwrap();
        let c1 = ColumnRef::new(table.clone(), col_id.clone(), ColumnType::BigInt);
        let c2 = ColumnRef::new(table, col_id, ColumnType::VarChar);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_column_ref_inequality_different_column() {
        let table = make_table_ref("sxt", "t");
        let col_a: proof_of_sql_parser::Identifier = "a".parse().unwrap();
        let col_b: proof_of_sql_parser::Identifier = "b".parse().unwrap();
        let c1 = ColumnRef::new(table.clone(), col_a, ColumnType::BigInt);
        let c2 = ColumnRef::new(table, col_b, ColumnType::BigInt);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_column_ref_clone() {
        let table = make_table_ref("sxt", "t");
        let col_id: proof_of_sql_parser::Identifier = "x".parse().unwrap();
        let c1 = ColumnRef::new(table, col_id, ColumnType::Boolean);
        let c2 = c1.clone();
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_column_ref_hash() {
        use std::collections::HashSet;
        let table = make_table_ref("sxt", "t");
        let col_id: proof_of_sql_parser::Identifier = "col".parse().unwrap();
        let c1 = ColumnRef::new(table.clone(), col_id.clone(), ColumnType::Int);
        let c2 = ColumnRef::new(table, col_id, ColumnType::Int);
        let mut set = HashSet::new();
        set.insert(c1);
        assert!(set.contains(&c2));
    }

    #[test]
    fn test_column_ref_ordering() {
        let table = make_table_ref("sxt", "t");
        let col_a: proof_of_sql_parser::Identifier = "a".parse().unwrap();
        let col_b: proof_of_sql_parser::Identifier = "b".parse().unwrap();
        let c1 = ColumnRef::new(table.clone(), col_a, ColumnType::BigInt);
        let c2 = ColumnRef::new(table, col_b, ColumnType::BigInt);
        // "a" < "b" lexicographically
        assert!(c1 < c2);
    }
}
