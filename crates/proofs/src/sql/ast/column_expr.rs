use crate::{
    base::{
        database::{Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor},
        scalar::ArkScalar,
    },
    sql::proof::{
        CountBuilder, DenseProvableResultColumn, MultilinearExtensionImpl, ProofBuilder,
        ResultBuilder, SumcheckSubpolynomial, SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use bumpalo::Bump;
use num_traits::One;
use serde::{Deserialize, Serialize};

/// Provable expression for a column
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ColumnExpr {
    column_ref: ColumnRef,
}

impl ColumnExpr {
    /// Create a new column expression
    pub fn new(column_ref: ColumnRef) -> Self {
        Self { column_ref }
    }

    /// Return the column referenced by this ColumnExpr
    pub fn get_column_reference(&self) -> ColumnRef {
        self.column_ref
    }

    /// Wrap the column output name and its type within the ColumnField
    pub fn get_column_field(&self) -> ColumnField {
        ColumnField::new(self.column_ref.column_id(), *self.column_ref.column_type())
    }

    /// Count the number of proof terms needed by this expression
    pub fn count(&self, builder: &mut CountBuilder) {
        builder.count_result_columns(1);
        builder.count_subpolynomials(1);
        builder.count_anchored_mles(1);
        builder.count_degree(3);
    }

    /// Evaluate the column expression and
    /// add the result to the ResultBuilder
    pub fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        accessor: &'a dyn DataAccessor,
    ) {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => {
                builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
            }
            Column::Int128(col) => {
                builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
            }
            Column::VarChar((col, _)) => {
                builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
            }
            #[cfg(test)]
            // While implementing this for a Scalar columns is very simple
            // major refactoring is required to create tests for this
            // (in particular the tests need to used the OwnedTableTestAccessor)
            Column::Scalar(_) => {
                todo!("Scalar column type not supported in ColumnExpr")
            }
        };
    }

    /// Given the selected rows (as a slice of booleans), evaluate the column expression and
    /// add the components needed to prove the result
    pub fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
        selection: &'a [bool],
    ) {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => prover_evaluate_impl(builder, alloc, selection, col),
            Column::Int128(col) => prover_evaluate_impl(builder, alloc, selection, col),
            Column::VarChar((_, scals)) => prover_evaluate_impl(builder, alloc, selection, scals),
            #[cfg(test)]
            // While implementing this for a Scalar columns is very simple
            // major refactoring is required to create tests for this
            // (in particular the tests need to used the OwnedTableTestAccessor)
            Column::Scalar(_) => {
                todo!("Scalar column type not supported in dense_filter_result_expr")
            }
        };
    }

    /// Given the evaluation of the selected row's multilinear extension at sumcheck's random point,
    /// add components needed to verify this column expression
    pub fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
        selection_eval: &ArkScalar,
    ) {
        let col_commit = accessor.get_commitment(self.column_ref);

        let result_eval = builder.consume_result_mle();
        let col_eval = builder.consume_anchored_mle(&col_commit);

        let poly_eval =
            builder.mle_evaluations.random_evaluation * (result_eval - col_eval * *selection_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&poly_eval);
    }
}

fn prover_evaluate_impl<'a, S: Clone + Default + Sync>(
    builder: &mut ProofBuilder<'a>,
    alloc: &'a Bump,
    selection: &'a [bool],
    col_scalars: &'a [S],
) where
    &'a S: Into<ArkScalar>,
    &'a bool: Into<ArkScalar>,
{
    // make a column of selected result values only
    let selected_vals = alloc.alloc_slice_fill_with(builder.table_length(), |i| {
        if selection[i] {
            col_scalars[i].clone()
        } else {
            S::default()
        }
    });

    // add sumcheck term for col * selection
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                One::one(),
                vec![Box::new(MultilinearExtensionImpl::new(selected_vals))],
            ),
            (
                -ArkScalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(col_scalars)),
                    Box::new(MultilinearExtensionImpl::new(selection)),
                ],
            ),
        ],
    ));

    // add MLE for result column
    builder.produce_anchored_mle(col_scalars);
}
