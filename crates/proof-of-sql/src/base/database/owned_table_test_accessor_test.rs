use crate::base::{
    commitment::naive_commitment::NaiveCommitment,
    database::{
        owned_table_utility::*, OwnedTableTestAccessor, SchemaAccessor, TableRef, TestAccessor,
    },
    scalar::Curve25519Scalar,
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

#[test]
fn test_owned_table_test_accessor_schema() {
    let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
    let table_ref = TableRef::new("schema.table".parse().unwrap());
    let table = owned_table([
        bigint("a", [1_i64, 2, 3]),
        varchar("b", ["x", "y", "z"]),
        boolean("c", [true, false, true]),
    ]);
    accessor.add_table(table_ref, table, 0);

    let schema = accessor.lookup_schema(table_ref);
    assert_eq!(schema.len(), 3);
}

#[test]
fn test_owned_table_test_accessor_metadata() {
    let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
    let table_ref = TableRef::new("schema.table".parse().unwrap());
    let table = owned_table([bigint("a", [10_i64, 20, 30])]);
    accessor.add_table(table_ref, table, 0);

    assert_eq!(accessor.get_length(table_ref), 3);
    assert_eq!(accessor.get_offset(table_ref), 0);
}

#[test]
fn test_owned_table_test_accessor_timestamp() {
    let mut accessor = OwnedTableTestAccessor::<NaiveCommitment>::new_empty_with_setup(());
    let table_ref = TableRef::new("schema.ts_table".parse().unwrap());
    let table = owned_table([timestamptz(
        "ts",
        PoSQLTimeUnit::Second,
        PoSQLTimeZone::utc(),
        [1_i64, 2, 3],
    )]);
    accessor.add_table(table_ref, table, 0);

    let schema = accessor.lookup_schema(table_ref);
    assert_eq!(schema.len(), 1);
    assert_eq!(accessor.get_length(table_ref), 3);
}
