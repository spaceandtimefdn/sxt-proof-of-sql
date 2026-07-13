use crate::{
    base::{
        database::{owned_table_utility::*, OwnedColumn, OwnedTable, OwnedTableError},
        map::IndexMap,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use sqlparser::ast::Ident;
#[test]
fn we_can_create_an_owned_table_with_no_columns() {
    let table = OwnedTable::<TestScalar>::try_new(IndexMap::default()).unwrap();
    assert_eq!(table.num_columns(), 0);
}
#[test]
fn we_can_create_an_empty_owned_table() {
    let owned_table = owned_table::<DoryScalar>([
        bigint("bigint", [0; 0]),
        int128("decimal", [0; 0]),
        varchar("varchar", ["0"; 0]),
        scalar("scalar", [0; 0]),
        boolean("boolean", [true; 0]),
    ]);
    let mut table = IndexMap::default();
    table.insert(Ident::new("bigint"), OwnedColumn::BigInt(vec![]));
    table.insert(Ident::new("decimal"), OwnedColumn::Int128(vec![]));
    table.insert(Ident::new("varchar"), OwnedColumn::VarChar(vec![]));
    table.insert(Ident::new("scalar"), OwnedColumn::Scalar(vec![]));
    table.insert(Ident::new("boolean"), OwnedColumn::Boolean(vec![]));
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_can_create_an_owned_table_with_data() {
    let owned_table = owned_table([
        bigint("bigint", [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
        int128("decimal", [0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
        varchar("varchar", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
        scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        boolean(
            "boolean",
            [true, false, true, false, true, false, true, false, true],
        ),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
        ),
    ]);
    let mut table = IndexMap::default();
    table.insert(
        Ident::new("time_stamp"),
        OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX].into(),
        ),
    );
    table.insert(
        Ident::new("bigint"),
        OwnedColumn::BigInt(vec![0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
    );
    table.insert(
        Ident::new("decimal"),
        OwnedColumn::Int128(vec![0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
    );
    table.insert(
        Ident::new("varchar"),
        OwnedColumn::VarChar(vec![
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
        ]),
    );
    table.insert(
        Ident::new("scalar"),
        OwnedColumn::Scalar(vec![
            DoryScalar::from(0),
            1.into(),
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            7.into(),
            8.into(),
        ]),
    );
    table.insert(
        Ident::new("boolean"),
        OwnedColumn::Boolean(vec![
            true, false, true, false, true, false, true, false, true,
        ]),
    );
    assert_eq!(owned_table.into_inner(), table);
}

#[test]
fn we_can_create_an_owned_table_with_all_column_helpers() {
    let owned_table = owned_table::<TestScalar>([
        uint8("uint8", [1_u8, 2]),
        tinyint("tinyint", [-1_i8, 2]),
        smallint("smallint", [-3_i16, 4]),
        int("int", [-5_i32, 6]),
        decimal75("decimal75", 12, -2, [7, 8]),
        varbinary("varbinary", [vec![1_u8, 2], vec![3, 4, 5]]),
    ]);

    assert_eq!(owned_table.num_columns(), 6);
    assert_eq!(owned_table.num_rows(), 2);
    assert!(!owned_table.is_empty());

    let column_names = owned_table
        .column_names()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        column_names,
        [
            "uint8",
            "tinyint",
            "smallint",
            "int",
            "decimal75",
            "varbinary"
        ]
    );

    assert_eq!(
        owned_table.column_by_index(0),
        Some(&OwnedColumn::Uint8(vec![1, 2]))
    );
    assert_eq!(
        owned_table.column_by_index(3),
        Some(&OwnedColumn::Int(vec![-5, 6]))
    );
    assert_eq!(
        owned_table.column_by_index(4),
        Some(&OwnedColumn::Decimal75(
            Precision::new(12).unwrap(),
            -2,
            vec![TestScalar::from(7), TestScalar::from(8)]
        ))
    );
    assert_eq!(
        owned_table.inner_table().get(&Ident::new("varbinary")),
        Some(&OwnedColumn::VarBinary(vec![vec![1, 2], vec![3, 4, 5]]))
    );
    assert_eq!(owned_table.column_by_index(6), None);
}
#[test]
fn we_get_inequality_between_tables_with_differing_column_order() {
    let owned_table_a: OwnedTable<TestScalar> = owned_table([
        bigint("a", [0; 0]),
        int128("b", [0; 0]),
        varchar("c", ["0"; 0]),
        boolean("d", [false; 0]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0; 0],
        ),
    ]);
    let owned_table_b: OwnedTable<TestScalar> = owned_table([
        boolean("d", [false; 0]),
        int128("b", [0; 0]),
        bigint("a", [0; 0]),
        varchar("c", ["0"; 0]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0; 0],
        ),
    ]);
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_get_inequality_between_tables_with_differing_data() {
    let owned_table_a: OwnedTable<DoryScalar> = owned_table([
        bigint("a", [0]),
        int128("b", [0]),
        varchar("c", ["0"]),
        boolean("d", [true]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [1_625_072_400],
        ),
    ]);
    let owned_table_b: OwnedTable<DoryScalar> = owned_table([
        bigint("a", [1]),
        int128("b", [0]),
        varchar("c", ["0"]),
        boolean("d", [true]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [1_625_076_000],
        ),
    ]);
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_cannot_create_an_owned_table_with_differing_column_lengths() {
    assert!(matches!(
        OwnedTable::<TestScalar>::try_from_iter([
            ("a".into(), OwnedColumn::BigInt(vec![0])),
            ("b".into(), OwnedColumn::BigInt(vec![])),
        ]),
        Err(OwnedTableError::ColumnLengthMismatch)
    ));
}
