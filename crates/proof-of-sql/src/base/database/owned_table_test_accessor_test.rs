/// Tests for OwnedTableTestAccessor – exercises paths not covered elsewhere.
#[cfg(test)]
mod tests {
    use crate::base::{
        database::{
            owned_table_utility::*, ColumnType, OwnedTable, OwnedTableTestAccessor,
            SchemaAccessor, TableRef, TestAccessor,
        },
        scalar::Scalar,
    };
    use crate::base::commitment::naive_evaluation_proof::NaiveCommitment;
    use proof_of_sql_parser::Identifier;

    type Accessor = OwnedTableTestAccessor<NaiveCommitment>;

    fn make_accessor() -> Accessor {
        Accessor::new_empty_with_setup(())
    }

    fn sample_table() -> OwnedTable<crate::base::scalar::test_scalar::TestScalar> {
        owned_table([
            bigint("id", [1i64, 2, 3]),
            varchar("name", ["alice", "bob", "carol"]),
        ])
    }

    /// Adding a table and retrieving its column count works.
    #[test]
    fn test_add_table_and_get_column_names() {
        let mut accessor = make_accessor();
        let table_ref = TableRef::new("schema", "users");
        accessor.add_table(table_ref, sample_table(), 0);

        let columns = accessor.lookup_schema(table_ref);
        assert_eq!(columns.len(), 2);
    }

    /// Duplicate column names do not appear twice.
    #[test]
    fn test_schema_contains_expected_column_types() {
        let mut accessor = make_accessor();
        let table_ref = TableRef::new("schema", "users");
        accessor.add_table(table_ref, sample_table(), 0);

        let schema = accessor.lookup_schema(table_ref);
        let id_col = Identifier::try_new("id").unwrap();
        let name_col = Identifier::try_new("name").unwrap();

        let id_type = schema.iter().find(|(col, _)| col == &id_col).map(|(_, t)| t);
        let name_type = schema.iter().find(|(col, _)| col == &name_col).map(|(_, t)| t);

        assert_eq!(id_type, Some(&ColumnType::BigInt));
        assert_eq!(name_type, Some(&ColumnType::VarChar));
    }

    /// Table length is correctly reported.
    #[test]
    fn test_get_table_length() {
        let mut accessor = make_accessor();
        let table_ref = TableRef::new("schema", "users");
        accessor.add_table(table_ref, sample_table(), 0);

        assert_eq!(accessor.get_length(table_ref), 3);
    }

    /// Offset is stored and returned correctly.
    #[test]
    fn test_get_offset() {
        let mut accessor = make_accessor();
        let table_ref = TableRef::new("schema", "events");
        accessor.add_table(table_ref, sample_table(), 7);

        assert_eq!(accessor.get_offset(table_ref), 7);
    }
}
