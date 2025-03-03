use crate::{
    base::database::{ColumnRef, ColumnType, LiteralValue, TableRef},
    sql::{
        evm_proof_plan::EVMProofPlan,
        proof_exprs::{
            AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr, TableExpr,
        },
        proof_plans::{DynProofPlan, EmptyExec, FilterExec},
    },
};
use core::iter;

#[test]
fn we_cannot_generate_serialized_proof_plan_for_unsupported_plan() {
    let plan = DynProofPlan::Empty(EmptyExec::new());

    bincode::serde::encode_to_vec(
        EVMProofPlan::new(plan),
        bincode::config::legacy()
            .with_fixed_int_encoding()
            .with_big_endian(),
    )
    .unwrap_err();
}

#[test]
fn we_can_generate_serialized_proof_plan_for_simple_filter() {
    let table_ref: TableRef = "namespace.table".parse().unwrap();
    let identifier_a = "a".into();
    let identifier_b = "b".into();
    let identifier_alias = "alias".into();

    let column_ref_a = ColumnRef::new(table_ref.clone(), identifier_a, ColumnType::BigInt);
    let column_ref_b = ColumnRef::new(table_ref.clone(), identifier_b, ColumnType::BigInt);

    let plan = DynProofPlan::Filter(FilterExec::new(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b)),
            alias: identifier_alias,
        }],
        TableExpr { table_ref },
        DynProofExpr::Equals(EqualsExpr::new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a))),
            Box::new(DynProofExpr::Literal(LiteralExpr::new(
                LiteralValue::BigInt(5),
            ))),
        )),
    ));

    let bytes = bincode::serde::encode_to_vec(
        EVMProofPlan::new(plan),
        bincode::config::legacy()
            .with_fixed_int_encoding()
            .with_big_endian(),
    )
    .unwrap();

    let expected_bytes: Vec<_> = iter::empty()
        .chain(&1_usize.to_be_bytes())
        .chain(&15_usize.to_be_bytes())
        .chain("namespace.table".as_bytes())
        .chain(&2_usize.to_be_bytes())
        .chain(&0_usize.to_be_bytes())
        .chain(&1_usize.to_be_bytes())
        .chain("b".as_bytes())
        .chain(&0_usize.to_be_bytes())
        .chain(&1_usize.to_be_bytes())
        .chain("a".as_bytes())
        .chain([])
        .chain(&0_u32.to_be_bytes()) //   FilterExec
        .chain(&0_usize.to_be_bytes()) //   table_number
        .chain(&1_u32.to_be_bytes()) //     where_clause - EqualsExpr
        .chain(&0_u32.to_be_bytes()) //       lhs - ColumnExpr
        .chain(&1_usize.to_be_bytes()) //       column_number
        .chain(&2_u32.to_be_bytes()) //       rhs - LiteralExpr
        .chain(&0_u32.to_be_bytes()) //         type
        .chain(&5_i64.to_be_bytes()) //         value
        .chain(&1_usize.to_be_bytes()) //   results.len()
        .chain(&0_u32.to_be_bytes()) //     results[0] - ColumnExpr
        .chain(&0_usize.to_be_bytes()) //     column_number
        .copied()
        .collect();
    assert_eq!(bytes, expected_bytes);
}
