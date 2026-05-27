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
    use super::NotExpr;
    use crate::{
        base::{
            database::{Column, ColumnRef, ColumnType, Table, TableRef},
            map::{indexmap, IndexSet},
            polynomial::MultilinearExtension,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::{
            proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
            proof_exprs::{ColumnExpr, DynProofExpr, ProofExpr},
        },
    };
    use bumpalo::Bump;
    use sqlparser::ast::Ident;
    use std::{collections::VecDeque, vec::Vec};

    #[test]
    fn we_can_evaluate_a_not_expr_directly() {
        let alloc = Bump::new();
        let t: TableRef = "sxt.t".parse().unwrap();
        let input = &[true, false, true, false];
        let expected = &[false, true, false, true];
        let table = Table::try_new(indexmap! {
            "flag".into() => Column::Boolean::<TestScalar>(input),
        })
        .unwrap();
        let flag = ColumnRef::new(t, Ident::from("flag"), ColumnType::Boolean);
        let not_expr = NotExpr::try_new(Box::new(DynProofExpr::Column(ColumnExpr::new(
            flag.clone(),
        ))))
        .unwrap();

        assert_eq!(not_expr.data_type(), ColumnType::Boolean);
        assert_eq!(not_expr.input().data_type(), ColumnType::Boolean);

        let first_round_result = not_expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round_result, Column::Boolean(expected));

        let mut final_round_builder: FinalRoundBuilder<'_, TestScalar> =
            FinalRoundBuilder::new(4, VecDeque::new());
        let final_round_result = not_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round_result, Column::Boolean(expected));

        let mut columns = IndexSet::default();
        not_expr.get_column_references(&mut columns);
        assert_eq!(columns.len(), 1);
        assert!(columns.contains(&flag));
    }

    #[test]
    fn we_can_verify_a_not_expr_directly() {
        let t: TableRef = "sxt.t".parse().unwrap();
        let input = &[true, false, true, false];
        let expected = &[false, true, false, true];
        let flag = ColumnRef::new(t, Ident::from("flag"), ColumnType::Boolean);
        let not_expr = NotExpr::try_new(Box::new(DynProofExpr::Column(ColumnExpr::new(
            flag.clone(),
        ))))
        .unwrap();
        let evaluation_points: Vec<Vec<_>> = (0..input.len())
            .map(|i| {
                (0..input.len())
                    .map(|j| {
                        if i == j {
                            TestScalar::ONE
                        } else {
                            TestScalar::ZERO
                        }
                    })
                    .collect()
            })
            .collect();
        let mut verification_builder: MockVerificationBuilder<TestScalar> =
            MockVerificationBuilder::new(
                Vec::new(),
                1,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            );

        for evaluation_point in &evaluation_points {
            let chi_eval = (&[1, 1, 1, 1]).inner_product(evaluation_point);
            let accessor = indexmap! {
                flag.clone().column_id() => input.inner_product(evaluation_point),
            };
            let result = not_expr
                .verifier_evaluate(&mut verification_builder, &accessor, chi_eval, &[])
                .unwrap();
            assert_eq!(result, expected.inner_product(evaluation_point));
            verification_builder.increment_row_index();
        }
    }
}
