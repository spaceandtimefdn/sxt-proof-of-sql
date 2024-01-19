use crate::{
    base::{
        database::{Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor},
        scalar::ArkScalar,
    },
    sql::proof::{
        CountBuilder, ProofBuilder, ResultBuilder, SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use num_traits::One;
use serde::{Deserialize, Serialize};

/// Provable expression for a result column within a filter SQL expression
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    pub fn count(&self, builder: &mut CountBuilder) {
        builder.count_result_columns(1);
        builder.count_subpolynomials(1);
        builder.count_anchored_mles(1);
        builder.count_degree(3);
    }

    /// Given the selected rows (as a slice of booleans), evaluate the filter result expression and
    /// add the result to the ResultBuilder
    pub fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) {
        builder.produce_result_column(accessor.get_column(self.column_ref));
    }

    /// Given the selected rows (as a slice of booleans), evaluate the filter result expression and
    /// add the components needed to prove the result
    pub fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, ArkScalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
        selection: &'a [bool],
    ) {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => prover_evaluate_impl(builder, alloc, selection, col),
            Column::Int128(col) => prover_evaluate_impl(builder, alloc, selection, col),
            Column::VarChar((_, scals)) => prover_evaluate_impl(builder, alloc, selection, scals),
            // While implementing this for a Scalar columns is very simple
            // major refactoring is required to create tests for this
            // (in particular the tests need to used the OwnedTableTestAccessor)
            Column::Scalar(_) => todo!("Scalar column type not supported in filter_result_expr"),
        };
    }

    /// Given the evaluation of the selected row's multilinear extension at sumcheck's random point,
    /// add components needed to verify this filter result expression
    pub fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<RistrettoPoint>,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
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
    builder: &mut ProofBuilder<'a, ArkScalar>,
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
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (One::one(), vec![Box::new(selected_vals as &[_])]),
            (
                -ArkScalar::one(),
                vec![Box::new(col_scalars), Box::new(selection)],
            ),
        ],
    );

    // add MLE for result column
    builder.produce_anchored_mle(col_scalars);
}
