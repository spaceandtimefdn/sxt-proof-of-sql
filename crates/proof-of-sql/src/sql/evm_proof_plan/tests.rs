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

#[test]
fn we_can_generate_serialized_proof_plan_for_simple_filter() {
    let table_ref: TableRef = "namespace.table".parse().unwrap();
    let identifier_a = "a".into();
    let identifier_b = "b".into();
    let identifier_alias = "alias".into();

    let column_ref_a = ColumnRef::new(table_ref.clone(), identifier_a, ColumnType::BigInt);
    let column_ref_b = ColumnRef::new(table_ref.clone(), identifier_b, ColumnType::BigInt);

    let table_exec = TableExec::new(
        table_ref.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
        ],
    );

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

    // Verify bytes are not empty and contain valid serialized data
    assert!(!bytes.is_empty());
}

#[test]
fn we_can_roundtrip_proof_plan_for_simple_filter() {
    let table_ref: TableRef = "namespace.table".parse().unwrap();
    let identifier_a = "a".into();
    let identifier_b = "b".into();
    let identifier_alias = "alias".into();

    let column_ref_a = ColumnRef::new(table_ref.clone(), identifier_a, ColumnType::BigInt);
    let column_ref_b = ColumnRef::new(table_ref.clone(), identifier_b, ColumnType::BigInt);

    let table_exec = TableExec::new(
        table_ref.clone(),
        vec![
            ColumnField::new("a".into(), ColumnType::BigInt),
            ColumnField::new("b".into(), ColumnType::BigInt),
        ],
    );

    let original_plan = DynProofPlan::Filter(FilterExec::new(
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

    let bytes = try_standard_binary_serialization(EVMProofPlan::new(original_plan.clone())).unwrap();
    let deserialized = try_standard_binary_deserialization::<EVMProofPlan>(&bytes).unwrap();
    let roundtripped_plan = deserialized.0.inner();

    assert_eq!(roundtripped_plan, &original_plan);
}
