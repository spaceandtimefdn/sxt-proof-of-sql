use super::DynProofPlan;
use crate::{
    base::{
        database::{
            join_util::{
                apply_sort_merge_join_indexes, get_columns_of_table, get_sort_merge_join_indexes,
                ordered_set_union,
            },
            slice_operation::apply_slice_to_indexes,
            ColumnField, ColumnRef, LiteralValue, OwnedTable, Table, TableEvaluation, TableOptions,
            TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_gadgets::{
            final_round_evaluate_membership_check, final_round_evaluate_monotonic,
            first_round_evaluate_membership_check, first_round_evaluate_monotonic,
            verify_membership_check, verify_monotonic,
        },
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;
use tracing::{span, Level};

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> INNER JOIN <ProofPlan>
///     ON col1 = col2
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortMergeJoinExec {
    pub(super) left: Box<DynProofPlan>,
    pub(super) right: Box<DynProofPlan>,
    // `j_l` in the protocol
    pub(super) left_join_column_indexes: Vec<usize>,
    // `j_r` in the protocol
    pub(super) right_join_column_indexes: Vec<usize>,
    pub(super) result_idents: Vec<Ident>,
}

impl SortMergeJoinExec {
    /// Create a new `SortMergeJoinExec` with the given left and right plans
    ///
    /// # Panics
    /// Panics if one of the following conditions is met:
    /// - The join column index is out of bounds
    /// - The number of join columns is different
    /// - The number of result idents is different from the expected number of columns
    #[must_use]
    pub fn new(
        left: Box<DynProofPlan>,
        right: Box<DynProofPlan>,
        left_join_column_indexes: Vec<usize>,
        right_join_column_indexes: Vec<usize>,
        result_idents: Vec<Ident>,
    ) -> Self {
        let num_columns_left = left.get_column_result_fields().len();
        let num_columns_right = right.get_column_result_fields().len();
        let max_left_join_column_index = left_join_column_indexes.iter().max().unwrap_or(&0);
        let max_right_join_column_index = right_join_column_indexes.iter().max().unwrap_or(&0);
        if *max_left_join_column_index >= num_columns_left
            || *max_right_join_column_index >= num_columns_right
        {
            panic!("Join column index out of bounds");
        }
        let num_join_columns = left_join_column_indexes.len();
        assert!(
            (num_join_columns == right_join_column_indexes.len()),
            "Join columns should have the same number of columns"
        );
        assert!(
            (result_idents.len() == num_columns_left + num_columns_right - num_join_columns),
            "The amount of result idents should be the same as the expected number of columns"
        );
        Self {
            left,
            right,
            left_join_column_indexes,
            right_join_column_indexes,
            result_idents,
        }
    }

    pub(crate) fn left_plan(&self) -> &DynProofPlan {
        &self.left
    }

    pub(crate) fn right_plan(&self) -> &DynProofPlan {
        &self.right
    }

    pub(crate) fn left_join_column_indexes(&self) -> &Vec<usize> {
        &self.left_join_column_indexes
    }

    pub(crate) fn right_join_column_indexes(&self) -> &Vec<usize> {
        &self.right_join_column_indexes
    }

    pub(crate) fn result_idents(&self) -> &Vec<Ident> {
        &self.result_idents
    }
}

#[expect(clippy::missing_panics_doc)]
fn compute_hat_column_evals<S: Scalar>(
    eval: &TableEvaluation<S>,
    rho_eval: S,
    join_column_indexes: &[usize],
) -> (Vec<S>, Vec<S>, usize) {
    let column_evals = eval.column_evals();
    let num_columns = column_evals.len();
    let column_and_rho_evals = column_evals
        .iter()
        .chain(core::iter::once(&rho_eval))
        .copied()
        .collect::<Vec<_>>();
    let hat_column_indexes = join_column_indexes
        .iter()
        .copied()
        .chain((0..=num_columns).filter(|i| !join_column_indexes.contains(i)))
        .collect::<Vec<_>>();
    let hat_column_evals = apply_slice_to_indexes(&column_and_rho_evals, &hat_column_indexes)
        .expect("Indexes can not be out of bounds");
    let join_column_evals = apply_slice_to_indexes(&column_and_rho_evals, join_column_indexes)
        .expect("Indexes can not be out of bounds");
    (hat_column_evals, join_column_evals, num_columns)
}

impl ProofPlan for SortMergeJoinExec
where
    SortMergeJoinExec: ProverEvaluate,
{
    #[expect(clippy::similar_names)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        let left_eval =
            self.left
                .verifier_evaluate(builder, accessor, None, chi_eval_map, params)?;
        let right_eval =
            self.right
                .verifier_evaluate(builder, accessor, None, chi_eval_map, params)?;
        let res_chi = builder.try_consume_chi_evaluation()?;
        // 2. alpha, beta
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        // 3. Chi evals and rho evals
        let left_rho_eval = builder.try_consume_rho_evaluation()?;
        let left_chi_eval = left_eval.chi_eval();
        let right_chi_eval = right_eval.chi_eval();
        // 4. column evals
        let (hat_left_column_evals, left_join_column_evals, num_columns_left) =
            compute_hat_column_evals(&left_eval, left_rho_eval, &self.left_join_column_indexes);
        let num_columns_u = self.left_join_column_indexes.len();
        if num_columns_u != 1 {
            return Err(ProofError::VerificationError {
                error: "Join on multiple columns not supported yet",
            });
        }
        let res_u_column_evals = builder.try_consume_first_round_mle_evaluations(num_columns_u)?;
        let res_left_column_evals =
            builder.try_consume_first_round_mle_evaluations(num_columns_left - num_columns_u)?;
        let rho_bar_left_eval = builder.try_consume_first_round_mle_evaluation()?;
        verify_membership_check(
            builder,
            alpha,
            beta,
            left_chi_eval,
            res_chi.0,
            &hat_left_column_evals,
            &res_u_column_evals
                .iter()
                .copied()
                .chain(res_left_column_evals.iter().copied())
                .chain(core::iter::once(rho_bar_left_eval))
                .collect::<Vec<_>>(),
        )?;
        let right_rho_eval = builder.try_consume_rho_evaluation()?;
        let (hat_right_column_evals, right_join_column_evals, num_columns_right) =
            compute_hat_column_evals(&right_eval, right_rho_eval, &self.right_join_column_indexes);
        let res_right_column_evals =
            builder.try_consume_first_round_mle_evaluations(num_columns_right - num_columns_u)?;
        let rho_bar_right_eval = builder.try_consume_first_round_mle_evaluation()?;
        // 5. Membership checks to verify output columns are subsets of input columns
        verify_membership_check(
            builder,
            alpha,
            beta,
            right_chi_eval,
            res_chi.0,
            &hat_right_column_evals,
            &res_u_column_evals
                .iter()
                .copied()
                .chain(res_right_column_evals.iter().copied())
                .chain(core::iter::once(rho_bar_right_eval))
                .collect::<Vec<_>>(),
        )?;
        //TODO: Relax to allow multiple columns
        if left_join_column_evals.len() != 1 || right_join_column_evals.len() != 1 {
            return Err(ProofError::VerificationError {
                error: "Left and right join columns should have exactly one column",
            });
        }
        // 6. Monotonicity checks
        let i_eval: S = itertools::repeat_n(S::TWO, 64_usize).product::<S>() * rho_bar_left_eval
            + rho_bar_right_eval;
        verify_monotonic::<S, true, true>(builder, alpha, beta, i_eval, res_chi.0)?;
        let u_chi_eval = builder.try_consume_chi_evaluation()?.0;
        let u_column_eval = builder.try_consume_first_round_mle_evaluation()?;
        verify_monotonic::<S, true, true>(builder, alpha, beta, u_column_eval, u_chi_eval)?;
        // 7. Membership checks to verify join columns
        let w_l_eval = verify_membership_check(
            builder,
            alpha,
            beta,
            u_chi_eval,
            left_chi_eval,
            &[u_column_eval],
            &left_join_column_evals,
        )?;
        let w_r_eval = verify_membership_check(
            builder,
            alpha,
            beta,
            u_chi_eval,
            right_chi_eval,
            &[u_column_eval],
            &right_join_column_evals,
        )?;
        // 8. Prove that sum w_l * w_r = chi_m
        // sum w_l * w_r - chi_m = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            w_l_eval * w_r_eval - res_chi.0,
            2,
        )?;
        // 9. Return the result
        // Drop the two rho columns of `\hat{J}` to get `J`
        let res_column_evals = res_u_column_evals
            .into_iter()
            .chain(res_left_column_evals)
            .chain(res_right_column_evals)
            .collect();
        Ok(TableEvaluation::new(res_column_evals, res_chi))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let left_other_column_indexes = (0..self.left.get_column_result_fields().len())
            .filter(|i| !self.left_join_column_indexes.contains(i))
            .collect::<Vec<_>>();
        let right_other_column_indexes = (0..self.right.get_column_result_fields().len())
            .filter(|i| !self.right_join_column_indexes.contains(i))
            .collect::<Vec<_>>();
        let left_join_column_fields = apply_slice_to_indexes(
            &self.left.get_column_result_fields(),
            &self.left_join_column_indexes,
        )
        .expect("Indexes can not be out of bounds");
        let left_other_column_fields = apply_slice_to_indexes(
            &self.left.get_column_result_fields(),
            &left_other_column_indexes,
        )
        .expect("Indexes can not be out of bounds");
        let right_other_column_fields = apply_slice_to_indexes(
            &self.right.get_column_result_fields(),
            &right_other_column_indexes,
        )
        .expect("Indexes can not be out of bounds");
        let column_types = left_join_column_fields
            .iter()
            .chain(left_other_column_fields.iter())
            .chain(right_other_column_fields.iter())
            .map(ColumnField::data_type)
            .collect::<Vec<_>>();
        self.result_idents
            .iter()
            .zip_eq(column_types)
            .map(|(ident, column_type)| ColumnField::new(ident.clone(), column_type))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.left
            .get_column_references()
            .into_iter()
            .chain(self.right.get_column_references())
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.left
            .get_table_references()
            .into_iter()
            .chain(self.right.get_table_references())
            .collect()
    }
}

impl ProverEvaluate for SortMergeJoinExec {
    #[tracing::instrument(
        name = "SortMergeJoinExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let left = self
            .left
            .first_round_evaluate(builder, alloc, table_map, params)?;
        let right = self
            .right
            .first_round_evaluate(builder, alloc, table_map, params)?;
        let num_rows_left = left.num_rows();
        let num_rows_right = right.num_rows();
        let num_columns_left = left.num_columns();
        let num_columns_right = right.num_columns();
        let c_l = get_columns_of_table(&left, &self.left_join_column_indexes)
            .expect("Indexes can not be out of bounds");
        let c_r = get_columns_of_table(&right, &self.right_join_column_indexes)
            .expect("Indexes can not be out of bounds");
        // 1. Conduct the join
        let (left_row_indexes, right_row_indexes): (Vec<usize>, Vec<usize>) =
            get_sort_merge_join_indexes(&c_l, &c_r, num_rows_left, num_rows_right)
                .iter()
                .copied()
                .unzip();
        let num_rows_res = left_row_indexes.len();
        builder.produce_chi_evaluation_length(num_rows_res);
        builder.request_post_result_challenges(2);

        // 2. Membership checks to verify output columns are subsets of input columns
        builder.produce_rho_evaluation_length(num_rows_left);
        let join_left_right_columns = apply_sort_merge_join_indexes(
            &left,
            &right,
            &self.left_join_column_indexes,
            &self.right_join_column_indexes,
            &left_row_indexes,
            &right_row_indexes,
            alloc,
        )
        .expect("Can not do sort merge join");
        let left_columns = join_left_right_columns.left_columns();
        for column in &left_columns {
            builder.produce_intermediate_mle(*column);
        }
        let hat_left_column_indexes = self
            .left_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_left).filter(|i| !self.left_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_left_columns =
            get_columns_of_table(&left.add_rho_column(alloc), &hat_left_column_indexes)
                .expect("Indexes can not be out of bounds");
        first_round_evaluate_membership_check(builder, alloc, &hat_left_columns, &left_columns);
        builder.produce_rho_evaluation_length(num_rows_right);
        let right_less_join_columns = join_left_right_columns.right_less_join_columns();
        for column in right_less_join_columns {
            builder.produce_intermediate_mle(column);
        }

        let hat_right_column_indexes = self
            .right_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_right).filter(|i| !self.right_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_right_columns =
            get_columns_of_table(&right.add_rho_column(alloc), &hat_right_column_indexes)
                .expect("Indexes can not be out of bounds");
        first_round_evaluate_membership_check(
            builder,
            alloc,
            &hat_right_columns,
            &join_left_right_columns.right_columns(),
        );
        // 3. Monotonicity checks
        let i = left_row_indexes
            .iter()
            .zip_eq(right_row_indexes.iter())
            .map(|(l, r)| S::from(*l as u64) * S::TWO_POW_64 + S::from(*r as u64))
            .collect::<Vec<_>>();
        let alloc_i = alloc.alloc_slice_copy(i.as_slice());
        first_round_evaluate_monotonic(builder, alloc, alloc_i);
        let u = ordered_set_union(&c_l, &c_r, alloc).unwrap();
        let num_columns_u = u.len();
        assert!(
            (num_columns_u == 1),
            "Join on multiple columns not supported yet"
        );
        let u_0 = u[0].to_scalar();
        let num_rows_u = u[0].len();
        let span = span!(Level::DEBUG, "allocate u_0").entered();
        let alloc_u_0 = alloc.alloc_slice_copy(u_0.as_slice());
        span.exit();
        builder.produce_chi_evaluation_length(num_rows_u);
        builder.produce_intermediate_mle(alloc_u_0 as &[_]);
        first_round_evaluate_monotonic(builder, alloc, alloc_u_0);

        // 4. Membership checks to prove join columns
        first_round_evaluate_membership_check(builder, alloc, &u, &c_l);
        first_round_evaluate_membership_check(builder, alloc, &u, &c_r);
        // 5. Return join result
        let tab = Table::try_from_iter_with_options(
            self.result_idents
                .iter()
                .cloned()
                .zip_eq(join_left_right_columns.result_columns()),
            TableOptions::new(Some(num_rows_res)),
        )
        .expect("Can not create table");
        Ok(tab)
    }

    #[tracing::instrument(
        name = "SortMergeJoinExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    #[expect(clippy::too_many_lines)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let left = self
            .left
            .final_round_evaluate(builder, alloc, table_map, params)?;
        let right = self
            .right
            .final_round_evaluate(builder, alloc, table_map, params)?;
        let num_rows_left = left.num_rows();
        let num_rows_right = right.num_rows();
        let num_columns_left = left.num_columns();
        let num_columns_right = right.num_columns();

        let chi_m_l = alloc.alloc_slice_fill_copy(num_rows_left, true);
        let chi_m_r = alloc.alloc_slice_fill_copy(num_rows_right, true);

        let c_l = get_columns_of_table(&left, &self.left_join_column_indexes)
            .expect("Indexes can not be out of bounds");
        let c_r = get_columns_of_table(&right, &self.right_join_column_indexes)
            .expect("Indexes can not be out of bounds");

        // 1. Conduct the join
        let (left_row_indexes, right_row_indexes): (Vec<usize>, Vec<usize>) =
            get_sort_merge_join_indexes(&c_l, &c_r, num_rows_left, num_rows_right)
                .iter()
                .copied()
                .unzip();
        let num_rows_res = left_row_indexes.len();
        let chi_res = alloc.alloc_slice_fill_copy(num_rows_res, true);
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        // Instead of storing the join result in a local `Vec`, we copy it into bump-allocated memory
        // so it will outlive this scope (matching the `'a` lifetime) and avoid borrow issues.
        let join_left_right_columns = apply_sort_merge_join_indexes(
            &left,
            &right,
            &self.left_join_column_indexes,
            &self.right_join_column_indexes,
            &left_row_indexes,
            &right_row_indexes,
            alloc,
        )
        .expect("Can not do sort merge join");

        // 2. Membership checks to verify output columns are subsets of input columns
        let hat_left_column_indexes = self
            .left_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_left).filter(|i| !self.left_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_left_columns =
            get_columns_of_table(&left.add_rho_column(alloc), &hat_left_column_indexes)
                .expect("Indexes can not be out of bounds");
        final_round_evaluate_membership_check(
            builder,
            alloc,
            alpha,
            beta,
            chi_m_l,
            chi_res,
            &hat_left_columns,
            &join_left_right_columns.left_columns(),
        );

        let hat_right_column_indexes = self
            .right_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_right).filter(|i| !self.right_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();

        let hat_right_columns =
            get_columns_of_table(&right.add_rho_column(alloc), &hat_right_column_indexes)
                .expect("Indexes can not be out of bounds");
        final_round_evaluate_membership_check(
            builder,
            alloc,
            alpha,
            beta,
            chi_m_r,
            chi_res,
            &hat_right_columns,
            &join_left_right_columns.right_columns(),
        );

        // 3. Monotonicity checks
        let i = left_row_indexes
            .iter()
            .zip_eq(right_row_indexes.iter())
            .map(|(l, r)| S::from(*l as u64) * S::TWO_POW_64 + S::from(*r as u64))
            .collect::<Vec<_>>();
        let alloc_i = alloc.alloc_slice_copy(i.as_slice());
        final_round_evaluate_monotonic::<S, true, true>(builder, alloc, alpha, beta, alloc_i);
        let u = ordered_set_union(&c_l, &c_r, alloc).unwrap();

        let num_columns_u = u.len();
        assert!(
            (num_columns_u == 1),
            "Join on multiple columns not supported yet"
        );
        let u_0 = u[0].to_scalar();
        let num_rows_u = u[0].len();
        let span = span!(Level::DEBUG, "allocate slices").entered();
        let alloc_u_0 = alloc.alloc_slice_copy(u_0.as_slice());
        let chi_u = alloc.alloc_slice_fill_copy(num_rows_u, true);
        span.exit();
        final_round_evaluate_monotonic::<S, true, true>(builder, alloc, alpha, beta, alloc_u_0);
        // 4. Membership checks to prove join columns
        let w_l = final_round_evaluate_membership_check(
            builder, alloc, alpha, beta, chi_u, chi_m_l, &u, &c_l,
        );
        let w_r = final_round_evaluate_membership_check(
            builder, alloc, alpha, beta, chi_u, chi_m_r, &u, &c_r,
        );

        // 5. Prove that sum w_l * w_r = chi_m
        // sum w_l * w_r - chi_m = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::ZeroSum,
            vec![
                (S::one(), vec![Box::new(w_l as &[_]), Box::new(w_r as &[_])]),
                (-S::one(), vec![Box::new(chi_res as &[_])]),
            ],
        );

        // 6. Return join result
        Ok(Table::try_from_iter_with_options(
            self.result_idents
                .iter()
                .cloned()
                .zip_eq(join_left_right_columns.result_columns()),
            TableOptions::new(Some(num_rows_res)),
        )
        .expect("Can not create table"))
    }
}
