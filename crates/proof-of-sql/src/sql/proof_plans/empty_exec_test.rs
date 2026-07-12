use super::EmptyExec;
use crate::{
    base::{
        bit::BitDistribution,
        database::{LiteralValue, Table, TableEvaluation, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofSizeMismatch,
        scalar::{test_scalar::TestScalar, Scalar},
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, SumcheckSubpolynomialType,
        VerificationBuilder,
    },
};
use alloc::{collections::VecDeque, vec::Vec};
use bumpalo::Bump;
use sqlparser::ast::Ident;

struct SingletonChiBuilder<S> {
    chi: S,
}

impl<S: Scalar> VerificationBuilder<S> for SingletonChiBuilder<S> {
    fn try_consume_chi_evaluation(&mut self) -> Result<(S, usize), ProofSizeMismatch> {
        unreachable!("EmptyExec uses the singleton chi evaluation directly")
    }

    fn try_consume_rho_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume rho evaluations")
    }

    fn try_consume_first_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume first-round MLE evaluations")
    }

    fn try_consume_first_round_mle_evaluations(
        &mut self,
        _count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume first-round MLE evaluations")
    }

    fn try_consume_final_round_mle_evaluation(&mut self) -> Result<S, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume final-round MLE evaluations")
    }

    fn try_consume_final_round_mle_evaluations(
        &mut self,
        _count: usize,
    ) -> Result<Vec<S>, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume final-round MLE evaluations")
    }

    fn try_consume_bit_distribution(&mut self) -> Result<BitDistribution, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume bit distributions")
    }

    fn try_produce_sumcheck_subpolynomial_evaluation(
        &mut self,
        _subpolynomial_type: SumcheckSubpolynomialType,
        _eval: S,
        _degree: usize,
    ) -> Result<(), ProofSizeMismatch> {
        unreachable!("EmptyExec does not produce sumcheck subpolynomial evaluations")
    }

    fn try_consume_post_result_challenge(&mut self) -> Result<S, ProofSizeMismatch> {
        unreachable!("EmptyExec does not consume post-result challenges")
    }

    fn singleton_chi_evaluation(&self) -> S {
        self.chi
    }

    fn rho_256_evaluation(&self) -> Option<S> {
        unreachable!("EmptyExec does not use rho_256 evaluations")
    }
}

#[test]
fn empty_exec_reports_no_inputs_or_outputs() {
    let plan = EmptyExec::default();

    assert_eq!(plan, EmptyExec::new());
    assert!(plan.get_column_result_fields().is_empty());
    assert_eq!(plan.get_column_references(), IndexSet::default());
    assert_eq!(plan.get_table_references(), IndexSet::default());
}

#[test]
fn empty_exec_verifier_evaluate_uses_singleton_chi_for_one_empty_row() {
    let plan = EmptyExec::new();
    let mut builder = SingletonChiBuilder {
        chi: TestScalar::from(17u8),
    };
    let accessor: IndexMap<TableRef, IndexMap<Ident, TestScalar>> = IndexMap::default();
    let chi_eval_map: IndexMap<TableRef, (TestScalar, usize)> = IndexMap::default();

    let evaluation = plan
        .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
        .unwrap();

    assert_eq!(
        evaluation,
        TableEvaluation::new(Vec::new(), (TestScalar::from(17u8), 1))
    );
}

#[test]
fn empty_exec_prover_rounds_return_empty_one_row_tables_without_builder_side_effects() {
    let plan = EmptyExec::new();
    let alloc = Bump::new();
    let table_map: IndexMap<TableRef, Table<'_, TestScalar>> = IndexMap::default();
    let params: &[LiteralValue] = &[];

    let mut first_round_builder = FirstRoundBuilder::new(0);
    let first_round_table = plan
        .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, params)
        .unwrap();

    assert!(first_round_table.is_empty());
    assert_eq!(first_round_table.num_columns(), 0);
    assert_eq!(first_round_table.num_rows(), 1);
    assert!(first_round_builder.pcs_proof_mles().is_empty());
    assert!(first_round_builder.chi_evaluation_lengths().is_empty());
    assert!(first_round_builder.rho_evaluation_lengths().is_empty());

    let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
    let final_round_table = plan
        .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, params)
        .unwrap();

    assert!(final_round_table.is_empty());
    assert_eq!(final_round_table.num_columns(), 0);
    assert_eq!(final_round_table.num_rows(), 1);
    assert!(final_round_builder.pcs_proof_mles().is_empty());
    assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 0);
}
