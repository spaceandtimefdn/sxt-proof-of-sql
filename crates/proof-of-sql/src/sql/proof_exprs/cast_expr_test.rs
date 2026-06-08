use super::{
    test_utility::{aliased_plan, cast, column},
    LiteralExpr,
};
use crate::{
    base::{
        database::{
            owned_table_utility::{
                bigint, boolean, decimal75, int, int128, owned_table, smallint, timestamptz,
                tinyint, uint8,
            },
            table_utility::{borrowed_smallint, table},
            ColumnRef, ColumnType, LiteralValue, OwnedTableTestAccessor, TableRef,
            TableTestAccessor,
        },
        map::{indexmap, IndexSet},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    },
    sql::{
        proof::{
            exercise_verification, mock_verification_builder::MockVerificationBuilder,
            VerifiableQueryResult,
        },
        proof_exprs::{CastExpr, ColumnExpr, DynProofExpr, ProofExpr},
        proof_plans::test_utility::{column_field, filter, table_exec},
        AnalyzeError,
    },
};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;
use sqlparser::ast::Ident;

#[test]
fn we_can_prove_a_simple_cast_expr() {
    let data = owned_table([
        boolean("a", [false, true, false, true]),
        boolean("b", [true, true, false, true]),
        boolean("c", [false, false, false, true]),
        boolean("d", [false, true, false, false]),
        boolean("e", [false, true, true, false]),
        timestamptz(
            "f",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::new(1),
            [1i64, -500, i64::MAX, 0],
        ),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![
            aliased_plan(
                cast(column(&t, "a", &accessor), ColumnType::TinyInt),
                "a_cast",
            ),
            aliased_plan(
                cast(column(&t, "b", &accessor), ColumnType::SmallInt),
                "b_cast",
            ),
            aliased_plan(cast(column(&t, "c", &accessor), ColumnType::Int), "c_cast"),
            aliased_plan(
                cast(column(&t, "d", &accessor), ColumnType::BigInt),
                "d_cast",
            ),
            aliased_plan(
                cast(column(&t, "e", &accessor), ColumnType::Int128),
                "e_cast",
            ),
            aliased_plan(
                cast(column(&t, "f", &accessor), ColumnType::BigInt),
                "f_cast",
            ),
        ],
        table_exec(
            t.clone(),
            vec![
                column_field("a", ColumnType::Boolean),
                column_field("b", ColumnType::Boolean),
                column_field("c", ColumnType::Boolean),
                column_field("d", ColumnType::Boolean),
                column_field("e", ColumnType::Boolean),
                column_field(
                    "f",
                    ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::new(1)),
                ),
            ],
        ),
        super::DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        tinyint("a_cast", [0i8, 1, 0, 1]),
        smallint("b_cast", [1i16, 1, 0, 1]),
        int("c_cast", [0i32, 0, 0, 1]),
        bigint("d_cast", [0i64, 1, 0, 0]),
        int128("e_cast", [0i128, 1, 1, 0]),
        bigint("f_cast", [1i64, -500, i64::MAX, 0]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_a_simple_cast_expr_from_int_to_other_numeric_type() {
    let data = owned_table([
        tinyint("a", [1]),
        uint8("b", [1]),
        smallint("c", [1i16]),
        int("d", [1i32]),
        bigint("e", [1i64]),
        int128("f", [1i128]),
        decimal75("g", 2, 0, [1]),
    ]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = filter(
        vec![
            aliased_plan(
                cast(column(&t, "a", &accessor), ColumnType::SmallInt),
                "a_cast",
            ),
            aliased_plan(
                cast(column(&t, "b", &accessor), ColumnType::Uint8),
                "b_cast",
            ),
            aliased_plan(
                cast(column(&t, "c", &accessor), ColumnType::BigInt),
                "c_cast",
            ),
            aliased_plan(
                cast(column(&t, "d", &accessor), ColumnType::Int128),
                "d_cast",
            ),
            aliased_plan(
                cast(
                    column(&t, "e", &accessor),
                    ColumnType::Decimal75(Precision::new(42).unwrap(), 0),
                ),
                "e_cast",
            ),
            aliased_plan(
                cast(
                    column(&t, "f", &accessor),
                    ColumnType::Decimal75(Precision::new(42).unwrap(), 0),
                ),
                "f_cast",
            ),
            aliased_plan(
                cast(
                    column(&t, "g", &accessor),
                    ColumnType::Decimal75(Precision::new(42).unwrap(), 0),
                ),
                "g_cast",
            ),
        ],
        table_exec(
            t.clone(),
            vec![
                column_field("a", ColumnType::TinyInt),
                column_field("b", ColumnType::Uint8),
                column_field("c", ColumnType::SmallInt),
                column_field("d", ColumnType::Int),
                column_field("e", ColumnType::BigInt),
                column_field("f", ColumnType::Int128),
                column_field("g", ColumnType::Decimal75(Precision::new(2).unwrap(), 0)),
            ],
        ),
        super::DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
    );
    let verifiable_res = VerifiableQueryResult::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable_res, &ast, &accessor, &t);
    let res = verifiable_res
        .verify(&ast, &accessor, &(), &[])
        .unwrap()
        .table;
    let expected_res = owned_table([
        smallint("a_cast", [1i16]),
        uint8("b_cast", [1u8]),
        bigint("c_cast", [1i64]),
        int128("d_cast", [1i128]),
        decimal75("e_cast", 42, 0, [1]),
        decimal75("f_cast", 42, 0, [1]),
        decimal75("g_cast", 42, 0, [1]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_get_error_if_we_cast_uncastable_type() {
    let data = owned_table([decimal75("a", 57, 2, [1_i16, 2, 3, 4])]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    assert!(matches!(
        DynProofExpr::try_new_cast(column(&t, "a", &accessor), ColumnType::BigInt),
        Err(AnalyzeError::DataTypeMismatch { .. })
    ));
}

#[test]
fn we_can_inspect_cast_expr_and_verify_delegated_value() {
    let t = TableRef::new("sxt", "t");
    let column_ref = ColumnRef::new(t, Ident::from("a"), ColumnType::Boolean);
    let input_expr = DynProofExpr::Column(ColumnExpr::new(column_ref.clone()));
    let cast_expr = CastExpr::try_new(Box::new(input_expr.clone()), ColumnType::TinyInt).unwrap();

    assert_eq!(cast_expr.get_from_expr(), &input_expr);
    assert_eq!(cast_expr.to_type(), &ColumnType::TinyInt);
    assert_eq!(cast_expr.data_type(), ColumnType::TinyInt);

    let mut columns = IndexSet::default();
    cast_expr.get_column_references(&mut columns);
    assert_eq!(columns.len(), 1);
    assert!(columns.contains(&column_ref));

    let mut verifier = MockVerificationBuilder::<TestScalar>::new(
        Vec::new(),
        1,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let accessor = indexmap! {
        column_ref.column_id() => TestScalar::ONE,
    };
    let res = cast_expr
        .verifier_evaluate(&mut verifier, &accessor, TestScalar::ONE, &[])
        .unwrap();
    assert_eq!(res, TestScalar::ONE);
}

#[test]
fn we_cannot_cast_mismatching_types() {
    let alloc = Bump::new();
    let data = table([borrowed_smallint("a", [1_i16, 2, 3, 4], &alloc)]);
    let t = TableRef::new("sxt", "t");
    let accessor =
        TableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data.clone(), 0, ());
    let lhs = Box::new(column(&t, "a", &accessor));
    let cast_err = CastExpr::try_new(lhs.clone(), ColumnType::TinyInt).unwrap_err();
    assert!(matches!(
        cast_err,
        AnalyzeError::DataTypeMismatch {
            left_type: _,
            right_type: _
        }
    ));
}
