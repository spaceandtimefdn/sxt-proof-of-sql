use super::DynProofPlan;
use crate::{
    base::{
        database::{
            union_util::table_union, Column, ColumnField, ColumnRef, LiteralValue, Table,
            TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        polynomial::MultilinearExtension,
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_gadgets::fold_log_expr::FoldLogExpr,
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan>
///     UNION ALL
///     <ProofPlan>
///     ...
///     UNION ALL
///     <ProofPlan>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UnionExec {
    pub(super) inputs: Vec<DynProofPlan>,
}

impl UnionExec {
    /// Tries to create a new union execution plan.
    pub fn try_new(inputs: Vec<DynProofPlan>) -> AnalyzeResult<Self> {
        (inputs.len() > 1)
            .then_some(Self { inputs })
            .ok_or(AnalyzeError::NotEnoughInputPlans)
    }

    pub(crate) fn input_plans(&self) -> &[DynProofPlan] {
        &self.inputs
    }
}

impl ProofPlan for UnionExec
where
    UnionExec: ProverEvaluate,
{
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let gamma = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let fold_log_gadget = FoldLogExpr::new(gamma, beta);
        let mut num_mle_evaluations = None;
        let c_star_evals = self
            .inputs
            .iter()
            .map(|input| -> Result<_, ProofError> {
                let table_evaluation =
                    input.verifier_evaluate(builder, accessor, chi_eval_map, params)?;
                let column_evals = table_evaluation.column_evals();
                num_mle_evaluations = num_mle_evaluations.or(Some(column_evals.len()));
                fold_log_gadget
                    .verify_evaluate(builder, column_evals, table_evaluation.chi_eval())
                    .map(|(star, _fold)| star)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let output_column_evals = builder.try_consume_first_round_mle_evaluations(
            num_mle_evaluations.expect("union should have multiple inputs"),
        )?;
        let chi_m = builder.try_consume_chi_evaluation()?;

        let (d_star_eval, _) =
            fold_log_gadget.verify_evaluate(builder, &output_column_evals, chi_m.0)?;

        // sum (sum c_star) - d_star = 0
        let zero_sum_terms_eval = c_star_evals
            .into_iter()
            .chain(core::iter::once(-d_star_eval))
            .sum::<S>();
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            zero_sum_terms_eval,
            1,
        )?;
        Ok(TableEvaluation::new(output_column_evals, chi_m))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.inputs
            .first()
            .expect("Union inputs should not be empty")
            .get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.inputs
            .iter()
            .flat_map(ProofPlan::get_column_references)
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.inputs
            .iter()
            .flat_map(ProofPlan::get_table_references)
            .collect()
    }
}

impl ProverEvaluate for UnionExec {
    #[tracing::instrument(name = "UnionExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        builder.request_post_result_challenges(2);
        let inputs = self
            .inputs
            .iter()
            .map(|input| -> PlaceholderResult<Table<'a, S>> {
                input.first_round_evaluate(builder, alloc, table_map, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        let res = table_union(&inputs, alloc).expect("Failed to union tables");

        // Produce intermediate MLEs for the union
        res.columns().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        builder.produce_chi_evaluation_length(res.num_rows());
        Ok(res)
    }

    #[tracing::instrument(name = "UnionExec::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let gamma = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        let fold_log_gadget = FoldLogExpr::new(gamma, beta);
        // Produce the proof for the union
        let (inputs, c_stars): (Vec<_>, Vec<_>) = self
            .inputs
            .iter()
            .map(|input| -> PlaceholderResult<_> {
                let table = input.final_round_evaluate(builder, alloc, table_map, params)?;
                let input_table = table.columns().copied().collect::<Vec<_>>();
                let (c_star, _) = fold_log_gadget.final_round_evaluate(
                    builder,
                    alloc,
                    &input_table,
                    table.num_rows(),
                );
                Ok((table, c_star))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let res = table_union(&inputs, alloc).expect("Failed to union tables");
        let output_columns: Vec<Column<'a, S>> = res.columns().copied().collect::<Vec<_>>();
        // No need to produce intermediate MLEs for `d_fold` because it is
        // the sum of `c_fold`
        let (d_star, _) =
            fold_log_gadget.final_round_evaluate(builder, alloc, &output_columns, res.num_rows());

        // sum (sum c_star) - d_star = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::ZeroSum,
            c_stars
                .into_iter()
                .map(|c_star| {
                    let boxed_c_star: Box<dyn MultilinearExtension<S>> = Box::new(c_star as &[_]);
                    (S::one(), vec![boxed_c_star])
                })
                .chain(core::iter::once({
                    let boxed_d_star: Box<dyn MultilinearExtension<S>> = Box::new(d_star as &[_]);
                    (-S::one(), vec![boxed_d_star])
                }))
                .collect(),
        );
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::UnionExec;
    use crate::{
        base::{map::IndexMap, scalar::test_scalar::TestScalar},
        sql::{
            proof::{mock_verification_builder::MockVerificationBuilder, ProofPlan},
            proof_plans::DynProofPlan,
        },
    };

    fn make_exec() -> UnionExec {
        let p1 = DynProofPlan::new_table("sxt.t1".parse().unwrap(), vec![]);
        let p2 = DynProofPlan::new_table("sxt.t2".parse().unwrap(), vec![]);
        UnionExec::try_new(vec![p1, p2]).unwrap()
    }

    #[test]
    fn we_get_error_on_first_missing_post_result_challenge() {
        let exec = make_exec();
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![], 0, vec![], vec![], vec![], vec![], vec![],
        );
        let result =
            exec.verifier_evaluate(&mut builder, &IndexMap::default(), &IndexMap::default(), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn we_get_error_on_second_missing_post_result_challenge() {
        let exec = make_exec();
        // Provide only one challenge so first succeeds but second fails
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![], 0, vec![], vec![], vec![TestScalar::ONE], vec![], vec![],
        );
        let result =
            exec.verifier_evaluate(&mut builder, &IndexMap::default(), &IndexMap::default(), &[]);
        assert!(result.is_err());
    }
}
