use super::{
    final_round_evaluate_boolean_or, final_round_evaluate_nullable_boolean_or_presence,
    first_round_evaluate_boolean_or, first_round_evaluate_nullable_boolean_or_presence,
    verifier_evaluate_boolean_or, verifier_evaluate_nullable_boolean_or_presence, DynProofExpr,
    NullableColumnEvaluation, ProofExpr,
};
use crate::{
    base::{
        database::{
            can_and_or_types, Column, ColumnRef, ColumnType, LiteralValue, NullableColumn, Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable logical OR expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl OrExpr {
    /// Create logical OR expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        can_and_or_types(left_datatype, right_datatype)
            .then_some(Self { lhs, rhs })
            .ok_or_else(|| AnalyzeError::DataTypeMismatch {
                left_type: left_datatype.to_string(),
                right_type: right_datatype.to_string(),
            })
    }

    /// Get the left-hand side expression
    pub fn lhs(&self) -> &DynProofExpr {
        &self.lhs
    }

    /// Get the right-hand side expression
    pub fn rhs(&self) -> &DynProofExpr {
        &self.rhs
    }
}

impl ProofExpr for OrExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "OrExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result = Column::Boolean(first_round_evaluate_boolean_or(
            table.num_rows(),
            alloc,
            lhs,
            rhs,
        ));

        log::log_memory_usage("End");

        Ok(result)
    }

    #[tracing::instrument(name = "OrExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result = Column::Boolean(final_round_evaluate_boolean_or(builder, alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(result)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;

        verifier_evaluate_boolean_or(builder, lhs, rhs)
    }

    fn is_nullable(&self) -> bool {
        self.lhs.is_nullable() || self.rhs.is_nullable()
    }

    fn first_round_evaluate_nullable<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<NullableColumn<'a, S>> {
        let lhs_column = self
            .lhs
            .first_round_evaluate_nullable(alloc, table, params)?;
        let rhs_column = self
            .rhs
            .first_round_evaluate_nullable(alloc, table, params)?;
        let lhs = lhs_column
            .values()
            .as_boolean()
            .expect("lhs is not boolean");
        let rhs = rhs_column
            .values()
            .as_boolean()
            .expect("rhs is not boolean");
        let values = Column::Boolean(first_round_evaluate_boolean_or(
            table.num_rows(),
            alloc,
            lhs,
            rhs,
        ));
        let presence = first_round_evaluate_nullable_boolean_or_presence(
            table.num_rows(),
            alloc,
            lhs,
            lhs_column.presence(),
            rhs,
            rhs_column.presence(),
        );
        Ok(NullableColumn::try_new(values, presence).expect("presence length should match values"))
    }

    fn final_round_evaluate_nullable<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<NullableColumn<'a, S>> {
        let lhs_column = self
            .lhs
            .final_round_evaluate_nullable(builder, alloc, table, params)?;
        let rhs_column = self
            .rhs
            .final_round_evaluate_nullable(builder, alloc, table, params)?;
        let lhs = lhs_column
            .values()
            .as_boolean()
            .expect("lhs is not boolean");
        let rhs = rhs_column
            .values()
            .as_boolean()
            .expect("rhs is not boolean");
        let values = Column::Boolean(final_round_evaluate_boolean_or(builder, alloc, lhs, rhs));
        let presence = final_round_evaluate_nullable_boolean_or_presence(
            builder,
            alloc,
            lhs,
            lhs_column.presence(),
            rhs,
            rhs_column.presence(),
        );
        Ok(NullableColumn::try_new(values, presence).expect("presence length should match values"))
    }

    fn verifier_evaluate_nullable<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<NullableColumnEvaluation<S>, ProofError> {
        let lhs = self
            .lhs
            .verifier_evaluate_nullable(builder, accessor, chi_eval, params)?;
        let rhs = self
            .rhs
            .verifier_evaluate_nullable(builder, accessor, chi_eval, params)?;
        let value_eval = verifier_evaluate_boolean_or(builder, lhs.value_eval(), rhs.value_eval())?;
        let presence_eval = verifier_evaluate_nullable_boolean_or_presence(
            builder,
            lhs.value_eval(),
            lhs.presence_eval(),
            rhs.value_eval(),
            rhs.presence_eval(),
        )?;
        Ok(NullableColumnEvaluation::new(value_eval, presence_eval))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
