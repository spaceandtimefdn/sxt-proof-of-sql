use super::{DynProofPlan, ProjectionExec, TableExec};
use crate::{
    base::{
        bit::BitDistribution,
        database::{
            table_utility::*, ColumnField, ColumnRef, ColumnType, TableEvaluation, TableRef,
        },
        map::indexmap,
        proof::ProofSizeMismatch,
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, DynProofExpr},
    },
};
use alloc::{collections::VecDeque, vec, vec::Vec};
use bumpalo::Bump;
use sqlparser::ast::Ident;

struct NoopVerificationBuilder;

impl<S: Scalar> VerificationBuilder<S> for NoopVerificationBuilder {
    fn try_consume_chi_evaluation(&mut self) -> Result<(S, usize), ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewChiLengths)
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewRhoLengths)
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_first_round_mle_evaluations(
        &mut self,
        _count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        _count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewMLEEvaluations)
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        Err(ProofSizeMismatch::TooFewBitDistributions)
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        _subpolynomial_type: SumcheckSubpolynomialType,
        _eval: S,
        _degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        Ok(())
    }

    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        Err(ProofSizeMismatch::PostResultCountMismatch)
    }

    fn singleton_chi_evaluation(&self) -> S {
        S::zero()
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        None
    }
}

fn column_expr(table_ref: &TableRef, name: &str, column_type: ColumnType) -> DynProofExpr {
    DynProofExpr::new_column(ColumnRef::new(
        table_ref.clone(),
        Ident::new(name),
        column_type,
    ))
}

fn aliased_expr(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr,
        alias: Ident::new(alias),
    }
}

fn projection_exec(table_ref: &TableRef) -> ProjectionExec {
    ProjectionExec::new(
        vec![
            aliased_expr(column_expr(table_ref, "b", ColumnType::BigInt), "renamed_b"),
            aliased_expr(
                column_expr(table_ref, "label", ColumnType::VarChar),
                "renamed_label",
            ),
        ],
        Box::new(DynProofPlan::Table(TableExec::new(
            table_ref.clone(),
            vec![
                ColumnField::new("a".into(), ColumnType::BigInt),
                ColumnField::new("b".into(), ColumnType::BigInt),
                ColumnField::new("label".into(), ColumnType::VarChar),
            ],
        ))),
    )
}

#[test]
fn projection_exposes_input_and_aliased_results_without_blitzar() {
    let table_ref = TableRef::new("sxt", "projection_source");
    let projection = projection_exec(&table_ref);

    assert_eq!(projection.aliased_results().len(), 2);
    assert!(matches!(projection.input(), DynProofPlan::Table(_)));
    assert_eq!(
        projection.get_column_result_fields(),
        vec![
            ColumnField::new("renamed_b".into(), ColumnType::BigInt),
            ColumnField::new("renamed_label".into(), ColumnType::VarChar),
        ]
    );
    assert_eq!(
        projection
            .get_table_references()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![table_ref]
    );
}

#[test]
fn projection_verifier_evaluate_uses_input_chi_and_projects_column_evaluations_without_blitzar() {
    let table_ref = TableRef::new("sxt", "projection_source");
    let projection = projection_exec(&table_ref);
    let accessor = indexmap! {
        table_ref.clone() => indexmap! {
            Ident::new("a") => TestScalar::from(3),
            Ident::new("b") => TestScalar::from(5),
            Ident::new("label") => TestScalar::from(7),
        }
    };
    let chi_eval_map = indexmap! {
        table_ref => (TestScalar::from(11), 3)
    };
    let mut builder = NoopVerificationBuilder;

    let evaluation = projection
        .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
        .unwrap();

    assert_eq!(
        evaluation,
        TableEvaluation::new(
            vec![TestScalar::from(5), TestScalar::from(7)],
            (11.into(), 3)
        )
    );
}

#[test]
fn projection_first_and_final_round_evaluate_without_blitzar() {
    let alloc = Bump::new();
    let table_ref = TableRef::new("sxt", "projection_source");
    let input = table::<TestScalar>([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [10, 20, 30], &alloc),
        borrowed_varchar("label", ["left", "middle", "right"], &alloc),
    ]);
    let table_map = indexmap! {
        table_ref.clone() => input
    };
    let projection = projection_exec(&table_ref);
    let expected = table::<TestScalar>([
        borrowed_bigint("renamed_b", [10, 20, 30], &alloc),
        borrowed_varchar("renamed_label", ["left", "middle", "right"], &alloc),
    ]);

    let mut first_round_builder = FirstRoundBuilder::new(3);
    let first_round_result = projection
        .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
        .unwrap();
    assert_eq!(first_round_result, expected);

    let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
    let final_round_result = projection
        .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
        .unwrap();
    assert_eq!(final_round_result, expected);
}
