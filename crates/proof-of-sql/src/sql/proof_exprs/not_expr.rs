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
        base::{
            database::{
                table_utility::{borrowed_boolean, table},
                TableRef,
            },
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec};

    fn boolean_column_ref(name: &str) -> ColumnRef {
        ColumnRef::new(
            TableRef::new("sxt", "not_inputs"),
            Ident::new(name),
            ColumnType::Boolean,
        )
    }

    #[test]
    fn not_expr_evaluates_rounds_without_blitzar() {
        let alloc = Bump::new();
        let table =
            table::<TestScalar>([borrowed_boolean("flag", [false, true, true, false], &alloc)]);
        let flag_ref = boolean_column_ref("flag");
        let not_expr =
            NotExpr::try_new(Box::new(DynProofExpr::new_column(flag_ref.clone()))).unwrap();

        assert_eq!(not_expr.data_type(), ColumnType::Boolean);
        assert_eq!(not_expr.input().data_type(), ColumnType::Boolean);

        let first_round = not_expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round, Column::Boolean(&[true, false, false, true]));

        let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
        let final_round = not_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, Column::Boolean(&[true, false, false, true]));

        let mut column_refs = IndexSet::default();
        not_expr.get_column_references(&mut column_refs);
        assert_eq!(column_refs.len(), 1);
        assert!(column_refs.contains(&flag_ref));
    }

    #[test]
    fn not_expr_verifier_evaluate_subtracts_inner_eval_without_blitzar() {
        let not_true = NotExpr::try_new(Box::new(DynProofExpr::new_literal(
            LiteralValue::Boolean(true),
        )))
        .unwrap();
        let not_false = NotExpr::try_new(Box::new(DynProofExpr::new_literal(
            LiteralValue::Boolean(false),
        )))
        .unwrap();
        let mut verifier = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        assert_eq!(
            not_true
                .verifier_evaluate(&mut verifier, &IndexMap::default(), 7_i64.into(), &[])
                .unwrap(),
            TestScalar::ZERO
        );
        assert_eq!(
            not_false
                .verifier_evaluate(&mut verifier, &IndexMap::default(), 7_i64.into(), &[])
                .unwrap(),
            TestScalar::from(7_i64)
        );
    }

    #[test]
    fn not_expr_rejects_non_boolean_operands_without_blitzar() {
        let error = NotExpr::try_new(Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(1))))
            .unwrap_err();

        assert!(matches!(error, AnalyzeError::InvalidDataType { .. }));
    }
}
