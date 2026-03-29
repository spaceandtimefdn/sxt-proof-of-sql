#[cfg(test)]
mod tests {
    use crate::base::{
        commitment::naive_commitment::NaiveCommitment,
        database::{
            owned_table_utility::*, OwnedTable, OwnedTableTestAccessor, SchemaAccessor,
            TableAccessor,
        },
        scalar::Curve25519Scalar,
    };
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    fn example_table() -> OwnedTable<Curve25519Scalar> {
        owned_table([
            bigint("a", [1_i64, 2, 3]),
            varchar("b", ["x", "y", "z"]),
        ])
    }

    #[test]
    fn test_table_accessor_returns_correct_table() {
        let accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "namespace.test_table".parse().unwrap();
        let table = example_table();
        let mut accessor = accessor;
        accessor.add_table(table_ref, table.clone(), 0);
        let retrieved = accessor.get_table(table_ref);
        assert_eq!(retrieved, &table);
    }

    #[test]
    fn test_schema_accessor_returns_correct_column_type() {
        use crate::base::database::ColumnType;
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "namespace.schema_test".parse().unwrap();
        let table = owned_table([bigint("col_a", [10_i64, 20]), boolean("col_b", [true, false])]);
        accessor.add_table(table_ref, table, 0);
        let col_a_ref = "namespace.schema_test.col_a".parse().unwrap();
        let col_b_ref = "namespace.schema_test.col_b".parse().unwrap();
        assert_eq!(accessor.lookup_column(table_ref, "col_a".into()), Some(ColumnType::BigInt));
        assert_eq!(accessor.lookup_column(table_ref, "col_b".into()), Some(ColumnType::Boolean));
        let _ = (col_a_ref, col_b_ref);
    }

    #[test]
    fn test_schema_accessor_returns_none_for_missing_column() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "namespace.missing_col_test".parse().unwrap();
        let table = owned_table([bigint("x", [1_i64])]);
        accessor.add_table(table_ref, table, 0);
        assert_eq!(accessor.lookup_column(table_ref, "nonexistent".into()), None);
    }

    #[test]
    fn test_offset_is_stored_correctly() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "namespace.offset_test".parse().unwrap();
        let table = example_table();
        accessor.add_table(table_ref, table, 42);
        assert_eq!(accessor.get_offset(table_ref), 42);
    }

    #[test]
    fn test_table_with_timestamp_column() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "namespace.ts_test".parse().unwrap();
        let table = owned_table([timestamptz(
            "ts",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0_i64, 1, 2],
        )]);
        accessor.add_table(table_ref, table.clone(), 0);
        let retrieved = accessor.get_table(table_ref);
        assert_eq!(retrieved, &table);
    }
}
