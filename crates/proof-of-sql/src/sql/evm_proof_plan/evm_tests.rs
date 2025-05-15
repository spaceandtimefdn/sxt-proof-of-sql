use crate::{
    base::{
        database::{
            owned_table_utility::*, ColumnType, CommitmentAccessor, LiteralValue,
            OwnedTableTestAccessor, TableRef,
        },
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    },
    proof_primitive::hyperkzg::{self, HyperKZGCommitment, HyperKZGCommitmentEvaluationProof},
    sql::{
        evm_proof_plan::EVMProofPlan,
        parse::QueryExpr,
        proof::{ProofPlan, VerifiableQueryResult},
        proof_exprs::{test_utility::*, DynProofExpr, TableExpr},
        proof_plans::DynProofPlan,
    },
};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use itertools::Itertools;

fn evm_verifier_with_extra_args(
    plan: &DynProofPlan,
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
        .args(["--sig", "verify(bytes,bytes,bytes,uint256[],uint256[])"])
        .arg("./test/verifier/Verifier.t.post.sol")
        .args([
            dbg!(hex::encode(&result_bytes)),
            dbg!(hex::encode(&query_bytes)),
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
    verifiable_result: &VerifiableQueryResult<HyperKZGCommitmentEvaluationProof>,
    accessor: &impl CommitmentAccessor<HyperKZGCommitment>,
) -> bool {
    evm_verifier_with_extra_args(plan, verifiable_result, accessor, &[])
        && evm_verifier_with_extra_args(plan, verifiable_result, accessor, &["--via-ir"])
        && evm_verifier_with_extra_args(plan, verifiable_result, accessor, &["--optimize"])
        && evm_verifier_with_extra_args(
            plan,
            verifiable_result,
            accessor,
            &["--optimize", "--via-ir"],
        )
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_query_with_all_supported_types_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
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
                    b"\x00\x01\x02\x03\x04",
                    b"\x00\x01\x02\x03\x04",
                    b"\xFF\xFE\xFD\xFC\xFB",
                    b"\xFF\xFE\xFD\xFC\xFB",
                    b"\xFF\xFE\xFD\xFC\xFB",
                ],
            ),
        ]),
        0,
        &ps[..],
    );

    let sql_list = [
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where bin = 0x0001020304",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where lang = 'en'",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where sxt = 'מרחב וזמן'",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where b",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where i8 = 0",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where i16 = 0",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where i32 = 1",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where i64 = 0",
        "SELECT b, i8, i16, i32, i64, d, t, lang, sxt, bin from table where d = 1",
    ];

    for sql in sql_list {
        let query =
            QueryExpr::try_new(sql.parse().unwrap(), "namespace".into(), &accessor).unwrap();
        let plan = query.proof_expr();

        let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
            &EVMProofPlan::new(plan.clone()),
            &accessor,
            &&ps[..],
            &[],
        )
        .unwrap();

        assert!(evm_verifier_all(plan, &verifiable_result, &accessor));

        verifiable_result
            .clone()
            .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
            .unwrap();
    }
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b FROM table WHERE a = 5".parse().unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let plan = query.proof_expr();

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

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_simple_filter_with_negative_literal_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, -2, 5, 3, -2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b FROM table WHERE a = -2".parse().unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let plan = query.proof_expr();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_filter_with_arithmetic_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2]),
            bigint("b", [0, 1, 2, 3, 4, 5]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT a, b FROM table WHERE a + b = a - b"
            .parse()
            .unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let plan = query.proof_expr();

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

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_filter_with_cast_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
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

    assert!(evm_verifier_all(&plan, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_filter_with_int_to_decimal_cast_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
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

    assert!(evm_verifier_all(&plan, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}

#[ignore = "This test requires the forge binary to be present"]
#[test]
fn we_can_verify_a_complex_filter_using_the_evm() {
    let (ps, vk) = hyperkzg::load_small_setup_for_testing();

    let accessor = OwnedTableTestAccessor::<HyperKZGCommitmentEvaluationProof>::new_from_table(
        "namespace.table".parse().unwrap(),
        owned_table([
            bigint("a", [5, 3, 2, 5, 3, 2, 102, 104, 107, 108]),
            bigint("b", [0, 1, 2, 3, 4, 5, 33, 44, 55, 6]),
            bigint("c", [0, 7, 8, 9, 10, 11, 14, 15, 73, 23]),
            bigint("d", [5, 7, 2, 5, 4, 1, 12, 22, 22, 22]),
        ]),
        0,
        &ps[..],
    );
    let query = QueryExpr::try_new(
        "SELECT b,c FROM table WHERE (a + b = d) and (b = a * c)"
            .parse()
            .unwrap(),
        "namespace".into(),
        &accessor,
    )
    .unwrap();
    let plan = query.proof_expr();
    let verifiable_result = VerifiableQueryResult::<HyperKZGCommitmentEvaluationProof>::new(
        &EVMProofPlan::new(plan.clone()),
        &accessor,
        &&ps[..],
        &[],
    )
    .unwrap();

    assert!(evm_verifier_all(plan, &verifiable_result, &accessor));

    verifiable_result
        .clone()
        .verify(&EVMProofPlan::new(plan.clone()), &accessor, &&vk, &[])
        .unwrap();
}
