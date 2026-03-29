#[cfg(test)]
mod tests {
    use crate::base::{
        commitment::naive_commitment::NaiveCommitment,
        database::{
            owned_table_utility::*, OwnedTableTestAccessor, TestAccessor,
        },
        scalar::test_scalar::TestScalar,
    };
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn test_owned_table_test_accessor_schema() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "sxt.table1".parse().unwrap();
        let table = owned_table([
            bigint("a", [1_i64, 2, 3]),
            varchar("b", ["x", "y", "z"]),
        ]);
        accessor.add_table(table_ref, table, 0);

        let columns = accessor.lookup_schema(table_ref);
        assert_eq!(columns.len(), 2);
    }

    #[test]
    fn test_owned_table_test_accessor_metadata() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "sxt.table2".parse().unwrap();
        let table = owned_table([bigint("x", [10_i64, 20, 30])]);
        accessor.add_table(table_ref, table, 42);

        assert_eq!(accessor.get_length(table_ref), 3);
        assert_eq!(accessor.get_offset(table_ref), 42);
    }

    #[test]
    fn test_owned_table_test_accessor_timestamp_column() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "sxt.table3".parse().unwrap();
        let table = owned_table([timestamptz(
            "ts",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [1_i64, 2, 3],
        )]);
        accessor.add_table(table_ref, table, 0);

        let columns = accessor.lookup_schema(table_ref);
        assert_eq!(columns.len(), 1);
        assert_eq!(accessor.get_length(table_ref), 3);
    }

    #[test]
    fn test_owned_table_test_accessor_boolean_column() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref = "sxt.table4".parse().unwrap();
        let table = owned_table([boolean("flag", [true, false, true])]);
        accessor.add_table(table_ref, table, 0);

        let columns = accessor.lookup_schema(table_ref);
        assert_eq!(columns.len(), 1);
        assert_eq!(accessor.get_length(table_ref), 3);
    }

    #[test]
    fn test_owned_table_test_accessor_multiple_tables() {
        let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
        let table_ref_a = "sxt.alpha".parse().unwrap();
        let table_ref_b = "sxt.beta".parse().unwrap();

        accessor.add_table(
            table_ref_a,
            owned_table([bigint("id", [1_i64, 2])]),
            0,
        );
        accessor.add_table(
            table_ref_b,
            owned_table([bigint("val", [100_i64, 200, 300])]),
            5,
        );

        assert_eq!(accessor.get_length(table_ref_a), 2);
        assert_eq!(accessor.get_length(table_ref_b), 3);
        assert_eq!(accessor.get_offset(table_ref_b), 5);
    }
}
