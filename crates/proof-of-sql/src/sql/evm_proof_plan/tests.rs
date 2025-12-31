use crate::{
    base::{
        database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
        try_standard_binary_deserialization, try_standard_binary_serialization,
    },
    sql::{
        evm_proof_plan::EVMProofPlan,
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr},
        proof_plans::{DynProofPlan, FilterExec, TableExec},
    },
};
use alloc::boxed::Box;
use core::iter;
use sqlparser::ast::Ident;
use std::sync::LazyLock;

static SIMPLE_FILTER_SERIALIZED_BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    iter::empty()
        // Header: tables
        .chain(&1_usize.to_be_bytes()) //   tables.len()
        .chain(&15_usize.to_be_bytes()) //  table name length
        .chain("namespace.table".as_bytes())
        // Header: columns (from TableExec schema, in order a, b)
        .chain(&2_usize.to_be_bytes()) //   columns.len()
        .chain(&0_usize.to_be_bytes()) //   column[0] table_index
        .chain(&1_usize.to_be_bytes()) //   column[0] name length
        .chain("a".as_bytes()) //           column[0] name
        .chain(&5_u32.to_be_bytes()) //     column[0] type (BigInt)
        .chain(&0_usize.to_be_bytes()) //   column[1] table_index
        .chain(&1_usize.to_be_bytes()) //   column[1] name length
        .chain("b".as_bytes()) //           column[1] name
        .chain(&5_u32.to_be_bytes()) //     column[1] type (BigInt)
        // Header: output column names
        .chain(&1_usize.to_be_bytes()) //   output_column_names.len()
        .chain(&5_usize.to_be_bytes()) //   output_column_names[0] length
        .chain("alias".as_bytes()) //       output_column_names[0]
        // Plan: FilterExec (variant 8)
        .chain(&8_u32.to_be_bytes()) //   FilterExec
        // Input plan: TableExec (variant 2)
        .chain(&2_u32.to_be_bytes()) //     TableExec
        .chain(&0_usize.to_be_bytes()) //     table_number
        .chain(&2_usize.to_be_bytes()) //     column_numbers.len()
        .chain(&0_usize.to_be_bytes()) //     column_numbers[0] (column "a" at index 0)
        .chain(&1_usize.to_be_bytes()) //     column_numbers[1] (column "b" at index 1)
        // where_clause: EqualsExpr
        .chain(&2_u32.to_be_bytes()) //     EqualsExpr
        .chain(&0_u32.to_be_bytes()) //       lhs - ColumnExpr
        .chain(&0_usize.to_be_bytes()) //       column_number (column "a" at index 0 in input schema)
        .chain(&1_u32.to_be_bytes()) //       rhs - LiteralExpr
        .chain(&5_u32.to_be_bytes()) //         type (BigInt)
        .chain(&5_i64.to_be_bytes()) //         value
        // results
        .chain(&1_usize.to_be_bytes()) //   results.len()
        .chain(&0_u32.to_be_bytes()) //     results[0] - ColumnExpr
        .chain(&1_usize.to_be_bytes()) //     column_number (column "b" at index 1 in input schema)
        .copied()
        .collect()
});

#[test]
fn we_can_generate_serialized_proof_plan_for_simple_filter() {
    let table_ref: TableRef = "namespace.table".parse().unwrap();
    let identifier_a: Ident = "a".into();
    let identifier_b: Ident = "b".into();
    let identifier_alias: Ident = "alias".into();

    let column_ref_a = ColumnRef::new(table_ref.clone(), identifier_a.clone(), ColumnType::BigInt);
    let column_ref_b = ColumnRef::new(table_ref.clone(), identifier_b.clone(), ColumnType::BigInt);

    // Create a table exec to use as input
    let column_fields = vec![
        ColumnField::new(identifier_a, ColumnType::BigInt),
        ColumnField::new(identifier_b, ColumnType::BigInt),
    ];
    let table_exec = TableExec::new(table_ref, column_fields);

    let plan = DynProofPlan::Filter(FilterExec::new(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b)),
            alias: identifier_alias,
        }],
        Box::new(DynProofPlan::Table(table_exec)),
        DynProofExpr::Equals(
            EqualsExpr::try_new(
                Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a))),
                Box::new(DynProofExpr::Literal(LiteralExpr::new(
                    LiteralValue::BigInt(5),
                ))),
            )
            .unwrap(),
        ),
    ));

    let bytes = try_standard_binary_serialization(EVMProofPlan::new(plan)).unwrap();
    assert_eq!(bytes, *SIMPLE_FILTER_SERIALIZED_BYTES);
}

#[test]
fn we_can_deserialize_proof_plan_for_simple_filter() {
    let table_ref: TableRef = "namespace.table".parse().unwrap();
    let identifier_a: Ident = "a".into();
    let identifier_b: Ident = "b".into();

    // For deserialized plans, column expressions reference the input plan's result columns,
    // which use empty table refs (since they're intermediate results, not actual table columns)
    let empty_table_ref = TableRef::from_names(None, "");
    let column_ref_a = ColumnRef::new(
        empty_table_ref.clone(),
        identifier_a.clone(),
        ColumnType::BigInt,
    );
    let column_ref_b = ColumnRef::new(empty_table_ref, identifier_b.clone(), ColumnType::BigInt);

    // Create the expected plan with TableExec as input
    let column_fields = vec![
        ColumnField::new(identifier_a, ColumnType::BigInt),
        ColumnField::new(identifier_b, ColumnType::BigInt),
    ];
    let table_exec = TableExec::new(table_ref, column_fields);

    let expected_plan = DynProofPlan::Filter(FilterExec::new(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b)),
            alias: Ident::new("alias"),
        }],
        Box::new(DynProofPlan::Table(table_exec)),
        DynProofExpr::Equals(
            EqualsExpr::try_new(
                Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a))),
                Box::new(DynProofExpr::Literal(LiteralExpr::new(
                    LiteralValue::BigInt(5),
                ))),
            )
            .unwrap(),
        ),
    ));

    let deserialized =
        try_standard_binary_deserialization::<EVMProofPlan>(&SIMPLE_FILTER_SERIALIZED_BYTES)
            .unwrap();
    let plan = deserialized.0.inner();
    assert_eq!(plan, &expected_plan);
}
