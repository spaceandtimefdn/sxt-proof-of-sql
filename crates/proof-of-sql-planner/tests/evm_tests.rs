//! In this file we run end-to-end tests for the evm verifier.
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use datafusion::config::ConfigOptions;
use itertools::Itertools;
use proof_of_sql::{
    base::{
        database::{
            owned_table_utility::{
                bigint, boolean, decimal75, int, owned_table, smallint, timestamptz, tinyint,
                varbinary, varchar,
            },
            ColumnRef, ColumnType, CommitmentAccessor, LiteralValue, OwnedTableTestAccessor,
            SchemaAccessor, TableRef, TestAccessor,
        },
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        try_standard_binary_deserialization, try_standard_binary_serialization,
    },
    proof_primitive::hyperkzg::{
        load_small_setup_for_testing, HyperKZGCommitment, HyperKZGCommitmentEvaluationProof,
    },
    sql::{
        evm_proof_plan::EVMProofPlan,
        proof::{ProofPlan, VerifiableQueryResult},
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
        proof_plans::DynProofPlan,
    },
};
use proof_of_sql_planner::sql_to_proof_plans;
use sqlparser::{ast::Ident, dialect::GenericDialect, parser::Parser};

#[expect(clippy::missing_panics_doc)]
fn evm_verifier_with_extra_args(
    plan: &DynProofPlan,
    params: &str,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
    extra_args: &[&'static str],
) -> bool {
    let commitments = plan
        .get_column_references()
        .into_iter()
        .map(|c| accessor.get_commitment(&c.table_ref(), &c.column_id()))
        .flat_map(|c| {
            c.commitment
                .into_affine()
                .xy()
                .map_or(["0".to_string(), "0".to_string()], |(x, y)| {
                    [x.into_bigint().to_string(), y.into_bigint().to_string()]
                })
        })
        .join(",");
    let table_lengths = plan
        .get_table_references()
        .into_iter()
        .map(|t| accessor.get_length(&t).to_string())
        .join(",");

    let bincode_options = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_big_endian();
    let query_bytes =
        bincode::serde::encode_to_vec(EVMProofPlan::new(plan.clone()), bincode_options).unwrap();
    let proof_bytes =
        bincode::serde::encode_to_vec(&verifiable_result.proof, bincode_options).unwrap();
    let result_bytes =
        bincode::serde::encode_to_vec(&verifiable_result.result, bincode_options).unwrap();

    std::process::Command::new("../../solidity/scripts/pre_forge.sh")
        .arg("script")
        .arg("-vvvvv")
        .args(extra_args)
        .args(["--tc", "VerifierTest"])
        .args([
            "--sig",
            "verify(bytes,bytes,uint256[],bytes,uint256[],uint256[])",
        ])
        .arg("./test/verifier/Verifier.t.post.sol")
        .args([
            dbg!(hex::encode(&result_bytes)),
            dbg!(hex::encode(&query_bytes)),
            dbg!(params.to_string()),
            dbg!(hex::encode(&proof_bytes)),
        ])
        .arg(dbg!(format!("[{table_lengths}]")))
        .arg(dbg!(format!("[{commitments}]")))
        .output()
        .unwrap()
        .status
        .success()
}
fn evm_verifier_all(
    plan: &DynProofPlan,
    params: &str,
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
) -> bool {
    evm_verifier_with_extra_args(plan, params, verifiable_result, accessor, &[])
        && evm_verifier_with_extra_args(plan, params, verifiable_result, accessor, &["--via-ir"])
        && evm_verifier_with_extra_args(plan, params, verifiable_result, accessor, &["--optimize"])
        && evm_verifier_with_extra_args(
            plan,
            params,
            verifiable_result,
            accessor,
            &["--optimize", "--via-ir"],
        )
}

#[expect(clippy::missing_panics_doc)]
fn col_ref(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab, &name).unwrap();
    ColumnRef::new(tab.clone(), name, type_col)
}

fn col_expr_plan(
    tab: &TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab, name, accessor))),
        alias: name.into(),
    }
}

fn aliased_plan(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr,
        alias: alias.into(),
    }
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_query_with_all_supported_types_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            boolean("b", [true, false, true, false, true]),
            tinyint("i8", [0, i8::MIN, i8::MAX, -1, 1]),
            smallint("i16", [0, i16::MIN, i16::MAX, -1, 1]),
            int("i32", [0, i32::MIN, i32::MAX, -1, 1]),
            bigint("i64", [0, i64::MIN, i64::MAX, -1, 1]),
            decimal75("d", 5, 0, [0, -2, -1, 1, 2]),
            timestamptz(
                "t",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [
                    1_746_627_936,
                    1_746_627_937,
                    1_746_627_938,
                    1_746_627_939,
                    1_746_627_940,
                ],
            ),
            varchar("lang", ["en", "he", "hu", "ru", "hy"]),
            varchar(
                "sxt",
                [
                    "Space and Time",
                    "מרחב וזמן",
                    "Tér és idő",
                    "Пространство и время",
                    "Տարածություն և ժամանակ",
                ],
            ),
            varbinary(
                "bin",
                [
                    &b""[..],
                    &b"\x00\x01\x02\x03\x04"[..],
                    &b"\xFF\xFE\xFD\xFC\xFB"[..],
                    &b"\xFF\xFE\xFD\xFC\xFB"[..],
                    &b"\xFF\xFE\xFD\xFC\xFB"[..],
                ],
            ),
        ]),
        0,
        &ps[..],
    );

    let sql_list = "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where bin = 0x;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where bin = 0x0001020304;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where lang = 'en';
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where sxt = 'מרחב וזמן';
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where b;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where i8 = 0;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where i16 = 0;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where i32 = 1;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where i64 = 0;
        SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from namespace.table where d = 1;";

    let statements = Parser::parse_sql(&GenericDialect {}, sql_list).unwrap();
    let plans = sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap();
    for plan in plans {
        let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
            &EVMProofPlan::new(plan.clone()),
            &accessor,
            &&ps[..],
            &[],
        )
        .unwrap();

        assert!(evm_verifier_all(&plan, "[]", &verifiable_result, &accessor));

        verifiable_result
            .clone()
            .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
            .unwrap();
    }
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_simple_filter_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT b FROM namespace.table WHERE a = 5",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));
}
#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_simple_filter_with_a_parameter_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT b FROM namespace.table WHERE a = $1",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[LiteralValue::BigInt(5)],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(
            &EVMProofPlan::new(plan.clone()),
            &accessor,
            &&vk,
            &[LiteralValue::BigInt(5)],
        )
        .unwrap();

    assert!(evm_verifier_all(plan, "[5]", &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_simple_filter_with_negative_literal_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, -2, 5, 3, -2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT b FROM namespace.table WHERE a = -2",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_filter_with_arithmetic_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT a, b FROM namespace.table WHERE a + b = a - b",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_filter_with_arithmetic_with_scaling_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            decimal75("b", 20, 2, [250, 150, 200, 300, 400, 500]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT a, b FROM namespace.table WHERE a = b + b",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_filter_with_cast_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2, 4]),
            boolean("b", [true, false, true, false, true, false, true]),
        ]),
        0,
        &ps[..],
    );
    let t = TableRef::from_names(Some("namespace"), "table");
    let plan = DynProofPlan::new_filter(
        vec![
            col_expr_plan(&t, "a", &accessor),
            aliased_plan(
                DynProofExpr::try_new_cast(
                    DynProofExpr::new_column(col_ref(&t, "b", &accessor)),
                    ColumnType::BigInt,
                )
                .unwrap(),
                "b",
            ),
        ],
        TableExpr {
            table_ref: t.clone(),
        },
        DynProofExpr::try_new_equals(
            DynProofExpr::new_column(col_ref(&t, "a", &accessor)),
            DynProofExpr::new_literal(LiteralValue::BigInt(4_i64)),
        )
        .unwrap(),
    );

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(&plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_filter_with_int_to_decimal_cast_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2, 4]),
            boolean("b", [true, false, true, false, true, false, true]),
        ]),
        0,
        &ps[..],
    );
    let t = TableRef::from_names(Some("namespace"), "table");
    let plan = DynProofPlan::new_filter(
        vec![
            aliased_plan(
                DynProofExpr::try_new_cast(
                    DynProofExpr::new_column(col_ref(&t, "a", &accessor)),
                    ColumnType::Decimal75(Precision::new(25).unwrap(), 0),
                )
                .unwrap(),
                "a",
            ),
            col_expr_plan(&t, "b", &accessor),
        ],
        TableExpr {
            table_ref: t.clone(),
        },
        DynProofExpr::try_new_equals(
            DynProofExpr::new_column(col_ref(&t, "a", &accessor)),
            DynProofExpr::new_literal(LiteralValue::BigInt(4_i64)),
        )
        .unwrap(),
    );

    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(&plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_complex_filter_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2, 102, 104, 107, 108]),
            bigint("b", [0, 1, 2, 3, 4, 5, 33, 44, 55, 6]),
            bigint("c", [0, 7, 8, 9, 10, 11, 14, 15, 73, 23]),
            bigint("d", [5, 7, 2, 5, 4, 1, 12, 22, 22, 22]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT b,c FROM namespace.table WHERE (a + b = d) and (b = a * c)",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_simple_projection_exec_and_table_exec_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let statements =
        Parser::parse_sql(&GenericDialect {}, "SELECT a, b, a+b FROM namespace.table").unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_simple_inequality_filter_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([bigint("a", [0, 0]), bigint("b", [0, 1])]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT a FROM namespace.table WHERE a<b",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_empty_exec_using_the_evm() {
    let (ps, _vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::default();
    let plan = &DynProofPlan::new_empty();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_groupby_query_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3]),
            bigint("b", [0, 1, 2, 3, 4]),
            bigint("c", [0, 2, 2, 1, 2]),
        ]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT a, sum(b) as sum_b, count(1) as count_0 FROM namespace.table WHERE c = 2 GROUP BY a",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_simple_slice_exec_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([bigint("a", [0, 3, -87, 6]), bigint("b", [3, -50, 1, 0])]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT a, b FROM namespace.table WHERE a > 2 LIMIT 2 OFFSET 1",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_slice_exec_using_the_evm() {
    let (ps, _) = load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        TableRef::new("namespace", "table"),
        owned_table([bigint("a", [5, 3, 2, 5, 3, 2, 102, 104, 107, 108])]),
        0,
        &ps[..],
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "SELECT COUNT(a) as a_count FROM namespace.table GROUP BY a",
    )
    .unwrap();
    let inner_plan =
        &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let inner_plan = DynProofPlan::new_slice(inner_plan.clone(), 1, Some(2));
    let plan = EVMProofPlan::new(inner_plan.clone());
    let serialized = try_standard_binary_serialization(plan).unwrap();
    let deserialized: EVMProofPlan = try_standard_binary_deserialization(&serialized).unwrap().0;
    assert_eq!(inner_plan, deserialized.into_inner());
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
#[expect(clippy::missing_panics_doc)]
fn we_can_verify_a_union_exec_using_the_evm() {
    let (ps, vk) = load_small_setup_for_testing();

    let mut accessor =
        OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_empty_with_setup(&ps[..]);
    accessor.add_table(
        TableRef::from_names(None, "table1"),
        owned_table([
            varchar("column1", ["Chloe", "Margaret", "Katy", "Lucy", "Prudence"]),
            bigint("column3", [1, 2, 3, 4, 5]),
        ]),
        0,
    );
    accessor.add_table(
        TableRef::from_names(None, "table2"),
        owned_table([
            varchar("column2", ["Test", "Some", "Creamy", "Chocolate"]),
            bigint("column4", [1, 2, 6, 4]),
        ]),
        0,
    );
    let statements = Parser::parse_sql(
        &GenericDialect {},
        "(SELECT column1, column3 FROM table1 where column3 > 1 limit 2 offset 1) UNION ALL SELECT column2, column4 FROM table2",
    )
    .unwrap();
    let plan = &sql_to_proof_plans(&statements, &accessor, &ConfigOptions::default()).unwrap()[0];
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, "[]", &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}
