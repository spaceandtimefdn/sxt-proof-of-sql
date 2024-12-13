use super::{
    count_equals_zero, count_or, count_sign, prover_evaluate_equals_zero, prover_evaluate_or,
    prover_evaluate_sign, result_evaluate_equals_zero, result_evaluate_or, result_evaluate_sign,
    scale_and_add_subtract_eval, scale_and_subtract, verifier_evaluate_equals_zero,
    verifier_evaluate_or, verifier_evaluate_sign, DynProofExpr, ProofExpr,
};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable AST expression for an inequality expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InequalityExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
    is_lte: bool,
}

impl InequalityExpr {
    /// Create a new less than or equal expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>, is_lte: bool) -> Self {
        Self { lhs, rhs, is_lte }
    }
}

impl ProofExpr for InequalityExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        count_equals_zero(builder);
        count_sign(builder)?;
        count_or(builder);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "InequalityExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column = self.lhs.result_evaluate(alloc, table);
        let rhs_column = self.rhs.result_evaluate(alloc, table);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let table_length = table.num_rows();
        let diff = if self.is_lte {
            scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, false)
                .expect("Failed to scale and subtract")
        } else {
            scale_and_subtract(alloc, rhs_column, lhs_column, rhs_scale, lhs_scale, false)
                .expect("Failed to scale and subtract")
        };

        // diff == 0
        let equals_zero = result_evaluate_equals_zero(table_length, alloc, diff);

        // sign(diff) == -1
        let sign = result_evaluate_sign(table_length, alloc, diff);

        // (diff == 0) || (sign(diff) == -1)
        Column::Boolean(result_evaluate_or(table_length, alloc, equals_zero, sign))
    }

    #[tracing::instrument(name = "InequalityExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        let lhs_column = self.lhs.prover_evaluate(builder, alloc, table);
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, table);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let diff = if self.is_lte {
            scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, false)
                .expect("Failed to scale and subtract")
        } else {
            scale_and_subtract(alloc, rhs_column, lhs_column, rhs_scale, lhs_scale, false)
                .expect("Failed to scale and subtract")
        };

        // diff == 0
        let equals_zero = prover_evaluate_equals_zero(table.num_rows(), builder, alloc, diff);

        // sign(diff) == -1
        let sign = prover_evaluate_sign(builder, alloc, diff);

        // (diff == 0) || (sign(diff) == -1)
        Column::Boolean(prover_evaluate_or(builder, alloc, equals_zero, sign))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        one_eval: S,
    ) -> Result<S, ProofError> {
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor, one_eval)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor, one_eval)?;
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let diff_eval = if self.is_lte {
            scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, true)
        } else {
            scale_and_add_subtract_eval(rhs_eval, lhs_eval, rhs_scale, lhs_scale, true)
        };

        // diff == 0
        let equals_zero = verifier_evaluate_equals_zero(builder, diff_eval, one_eval);

        // sign(diff) == -1
        let sign = verifier_evaluate_sign(builder, diff_eval, one_eval)?;

        // (diff == 0) || (sign(diff) == -1)
        Ok(verifier_evaluate_or(builder, &equals_zero, &sign))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
