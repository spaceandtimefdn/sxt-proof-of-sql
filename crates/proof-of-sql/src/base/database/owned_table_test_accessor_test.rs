use super::OwnedTableTestAccessor;
use crate::base::{
    commitment::naive_commitment::NaiveCommitment,
    database::{OwnedTable, TableRef},
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::{ColumnType, OwnedColumn},
        scalar::test_scalar::TestScalar,
    };
    use indexmap::IndexMap;
    use sqlparser::ast::Ident;

    fn make_simple_table() -> OwnedTable<TestScalar> {
        let mut columns = IndexMap::new();
        columns.insert(
            Ident::new("id"),
            OwnedColumn::BigInt(vec![1i64, 2, 3]),
        );
        columns.insert(
            Ident::new("val"),
            OwnedColumn::Int128(vec![10i128, 20, 30]),
        );
        OwnedTable::try_new(columns).unwrap()
    }

    #[test]
    fn test_accessor_schema_reflects_table_columns() {
        let table = make_simple_table();
        let table_ref = TableRef::new("schema", "test_table");
        let accessor =
            OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, table, 0, ());
        let schema = accessor.lookup_schema(table_ref);
        // Should have exactly 2 columns
        assert_eq!(schema.len(), 2);
        let col_names: Vec<&str> = schema.iter().map(|(id, _)| id.value.as_str()).collect();
        assert!(col_names.contains(&"id"));
        assert!(col_names.contains(&"val"));
    }

    #[test]
    fn test_accessor_schema_column_types() {
        let table = make_simple_table();
        let table_ref = TableRef::new("schema", "test_table");
        let accessor =
            OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, table, 0, ());
        let schema = accessor.lookup_schema(table_ref);
        let type_map: IndexMap<String, ColumnType> = schema
            .iter()
            .map(|(id, ct)| (id.value.clone(), *ct))
            .collect();
        assert_eq!(type_map["id"], ColumnType::BigInt);
        assert_eq!(type_map["val"], ColumnType::Int128);
    }

    #[test]
    fn test_accessor_empty_table() {
        let empty_table: OwnedTable<TestScalar> =
            OwnedTable::try_new(IndexMap::new()).unwrap();
        let table_ref = TableRef::new("schema", "empty_table");
        let accessor =
            OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, empty_table, 0, ());
        let schema = accessor.lookup_schema(table_ref);
        assert!(schema.is_empty());
    }

    #[test]
    fn test_accessor_offset_is_stored() {
        let table = make_simple_table();
        let table_ref = TableRef::new("schema", "offset_table");
        let offset = 42_usize;
        let accessor =
            OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, table, offset, ());
        // The accessor should record the table length + offset for row-count purposes
        // Just verify construction succeeds without panic
        let schema = accessor.lookup_schema(table_ref);
        assert_eq!(schema.len(), 2);
    }

    #[test]
    fn test_accessor_timestamp_column_type() {
        let mut columns = IndexMap::new();
        columns.insert(
            Ident::new("ts"),
            OwnedColumn::<TestScalar>::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                vec![0i64, 1, 2],
            ),
        );
        let table = OwnedTable::try_new(columns).unwrap();
        let table_ref = TableRef::new("schema", "ts_table");
        let accessor =
            OwnedTableTestAccessor::<NaiveCommitment>::new_from_table(table_ref, table, 0, ());
        let schema = accessor.lookup_schema(table_ref);
        assert_eq!(schema.len(), 1);
        let (_, col_type) = schema.first().unwrap();
        assert!(matches!(
            col_type,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, _)
        ));
    }
}
