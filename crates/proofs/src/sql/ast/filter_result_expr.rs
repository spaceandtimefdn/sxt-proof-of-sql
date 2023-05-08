use crate::base::database::{Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::polynomial::{to_ark_scalar, ArkScalar};
use crate::base::scalar::One;
use crate::base::scalar::ToArkScalar;
use crate::sql::proof::EncodeProvableResultElement;
use crate::sql::proof::{
    DenseProvableResultColumn, MultilinearExtensionImpl, ProofBuilder, ProofCounts,
    SumcheckSubpolynomial, VerificationBuilder,
};
use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use std::cmp::max;

/// Provable expression for a result column within a filter SQL expression
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FilterResultExpr {
    column_ref: ColumnRef,
}

impl FilterResultExpr {
    /// Create a new filter result expression
    pub fn new(column_ref: ColumnRef) -> Self {
        Self { column_ref }
    }

    /// Return the column referenced by this FilterResultExpr
    pub fn get_column_reference(&self) -> ColumnRef {
        self.column_ref
    }

    /// Wrap the column output name and its type within the ColumnField
    pub fn get_column_field(&self) -> ColumnField {
        ColumnField::new(self.column_ref.column_id(), *self.column_ref.column_type())
    }

    /// Count the number of proof terms needed by this expression
    pub fn count(&self, counts: &mut ProofCounts) {
        counts.result_columns += 1;
        counts.sumcheck_subpolynomials += 1;
        counts.anchored_mles += 1;
        counts.sumcheck_max_multiplicands = max(counts.sumcheck_max_multiplicands, 3);
    }

    /// Given the selected rows (as a slice of booleans), evaluate the filter result expression and
    /// add the components needed to prove the result
    pub fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
        selection: &'a [bool],
    ) {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => {
                prover_evaluate_impl(builder, alloc, counts, selection, col, col)
            }
            Column::HashedBytes((col, scals)) => {
                prover_evaluate_impl(builder, alloc, counts, selection, col, scals)
            }
        };
    }

    /// Given the evaluation of the selected row's multilinear extension at sumcheck's random point,
    /// add components needed to verify this filter result expression
    #[cfg_attr(not(test), deprecated = "use `verifier_evaluate_ark()` instead")]
    pub fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
        selection_eval: &Scalar,
    ) {
        self.verifier_evaluate_ark(builder, counts, accessor, &to_ark_scalar(selection_eval))
    }
    /// Given the evaluation of the selected row's multilinear extension at sumcheck's random point,
    /// add components needed to verify this filter result expression
    pub fn verifier_evaluate_ark(
        &self,
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
        selection_eval: &ArkScalar,
    ) {
        let col_commit = accessor.get_commitment(self.column_ref);

        let result_eval = builder.consume_result_mle_ark();
        let col_eval = builder.consume_anchored_mle_ark(&col_commit);

        let poly_eval = builder.mle_evaluations.get_random_evaluation_ark()
            * (result_eval - col_eval * selection_eval);
        builder.produce_sumcheck_subpolynomial_evaluation_ark(&poly_eval);
    }
}

fn prover_evaluate_impl<
    'a,
    T: EncodeProvableResultElement,
    S: ToArkScalar + Clone + Default + Sync,
>(
    builder: &mut ProofBuilder<'a>,
    alloc: &'a Bump,
    counts: &ProofCounts,
    selection: &'a [bool],
    col_data: &'a [T],
    col_scalars: &'a [S],
) where
    [T]: ToOwned,
{
    // add result column
    builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col_data)));

    // make a column of selected result values only
    let selected_vals = alloc.alloc_slice_fill_with(counts.table_length, |i| {
        if selection[i] {
            col_scalars[i].clone()
        } else {
            S::default()
        }
    });

    // add sumcheck term for col * selection
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
        (
            One::one(),
            vec![Box::new(MultilinearExtensionImpl::new(selected_vals))],
        ),
        (
            -Scalar::one(),
            vec![
                Box::new(MultilinearExtensionImpl::new(col_scalars)),
                Box::new(MultilinearExtensionImpl::new(selection)),
            ],
        ),
    ]));

    // add MLE for result column
    builder.produce_anchored_mle(col_scalars);
}
