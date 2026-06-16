use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{can_not_type, Column, ColumnRef, ColumnType, LiteralValue, Table},
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
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable logical NOT expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotExpr {
    expr: Box<DynProofExpr>,
}

impl NotExpr {
    /// Create logical NOT expression
    pub fn try_new(expr: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let expr_type = expr.data_type();
        can_not_type(expr_type)
            .then_some(Self { expr })
            .ok_or(AnalyzeError::InvalidDataType { expr_type })
    }

    /// Get the input expression
    pub fn input(&self) -> &DynProofExpr {
        &self.expr
    }
}

impl ProofExpr for NotExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "NotExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let expr_column: Column<'a, S> = self.expr.first_round_evaluate(alloc, table, params)?;
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "NotExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let expr_column: Column<'a, S> = self
            .expr
            .final_round_evaluate(builder, alloc, table, params)?;
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let eval = self
            .expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        Ok(chi_eval - eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::{proof_exprs::DynProofExpr, AnalyzeError},
    };

    fn bool_literal() -> Box<DynProofExpr> {
        Box::new(DynProofExpr::Literal(LiteralExpr::new(
            LiteralValue::Boolean(false),
        )))
    }

    fn bigint_literal() -> Box<DynProofExpr> {
        Box::new(DynProofExpr::Literal(LiteralExpr::new(
            LiteralValue::BigInt(1),
        )))
    }

    #[test]
    fn try_new_boolean_succeeds() {
        assert!(NotExpr::try_new(bool_literal()).is_ok());
    }

    #[test]
    fn try_new_non_boolean_fails() {
        let result = NotExpr::try_new(bigint_literal());
        assert!(matches!(result, Err(AnalyzeError::InvalidDataType { .. })));
    }

    #[test]
    fn input_accessor_returns_boolean() {
        let expr = NotExpr::try_new(bool_literal()).unwrap();
        assert_eq!(expr.input().data_type(), ColumnType::Boolean);
    }

    #[test]
    fn data_type_is_boolean() {
        let expr = NotExpr::try_new(bool_literal()).unwrap();
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn equal_exprs_compare_equal() {
        let a = NotExpr::try_new(bool_literal()).unwrap();
        let b = NotExpr::try_new(bool_literal()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn clone_equals_original() {
        let a = NotExpr::try_new(bool_literal()).unwrap();
        assert_eq!(a.clone(), a);
    }
}
