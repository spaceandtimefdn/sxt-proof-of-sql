use super::{add_subtract_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_inequality_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        proof_gadgets::{
            final_round_evaluate_sign, first_round_evaluate_sign, verifier_evaluate_sign,
        },
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable AST expression for an inequality expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InequalityExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
    is_lt: bool,
}

impl InequalityExpr {
    /// Create a new less than or equal
    pub fn try_new(
        lhs: Box<DynProofExpr>,
        rhs: Box<DynProofExpr>,
        is_lt: bool,
    ) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_inequality_types(left_datatype, right_datatype)
            .map(|()| Self { lhs, rhs, is_lt })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
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

    /// Get whether this is a less-than comparison
    pub fn is_lt(&self) -> bool {
        self.is_lt
    }
}

impl ProofExpr for InequalityExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(
        name = "InequalityExpr::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column = self.rhs.first_round_evaluate(alloc, table, params)?;
        let table_length = table.num_rows();
        let diff = if self.is_lt {
            add_subtract_columns(lhs_column, rhs_column, alloc, true)
        } else {
            add_subtract_columns(rhs_column, lhs_column, alloc, true)
        };

        // (sign(diff) == -1)
        let res = Column::Boolean(first_round_evaluate_sign(table_length, alloc, diff));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "InequalityExpr::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let diff = if self.is_lt {
            add_subtract_columns(lhs_column, rhs_column, alloc, true)
        } else {
            add_subtract_columns(rhs_column, lhs_column, alloc, true)
        };

        // (sign(diff) == -1)
        let res = Column::Boolean(final_round_evaluate_sign(builder, alloc, diff));

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
        let lhs_eval = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs_eval = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let diff_eval = if self.is_lt {
            lhs_eval - rhs_eval
        } else {
            rhs_eval - lhs_eval
        };

        // sign(diff) == -1
        verifier_evaluate_sign(builder, diff_eval, chi_eval, None)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{table_utility::*, TableRef},
            map::{indexmap, indexset},
            polynomial::MultilinearExtension,
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::{
                mock_verification_builder::run_verify_for_each_row, FinalRoundBuilder,
                FirstRoundBuilder,
            },
            proof_exprs::ColumnExpr,
        },
    };
    use alloc::{boxed::Box, collections::VecDeque};

    fn bigint_column(table_ref: &TableRef, name: &str) -> (ColumnRef, DynProofExpr) {
        let column_ref = ColumnRef::new(table_ref.clone(), name.into(), ColumnType::BigInt);
        (
            column_ref.clone(),
            DynProofExpr::Column(ColumnExpr::new(column_ref)),
        )
    }

    #[test]
    fn try_new_reports_inequality_type_mismatch() {
        let err = InequalityExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(1))),
            Box::new(DynProofExpr::new_literal(LiteralValue::VarChar(
                "one".into(),
            ))),
            true,
        )
        .unwrap_err();

        assert_eq!(
            err,
            AnalyzeError::DataTypeMismatch {
                left_type: "BIGINT".into(),
                right_type: "VARCHAR".into()
            }
        );
    }

    #[test]
    fn inequality_expr_evaluates_and_verifies_column_comparison() {
        let alloc = Bump::new();
        let lhs_values = [1_i64, 2, 3, 4];
        let rhs_values = [2_i64, 1, 3, 5];
        let table = table::<TestScalar>([
            borrowed_bigint("a", lhs_values, &alloc),
            borrowed_bigint("b", rhs_values, &alloc),
        ]);
        let table_ref = TableRef::new("sxt", "t");
        let (lhs_ref, lhs) = bigint_column(&table_ref, "a");
        let (rhs_ref, rhs) = bigint_column(&table_ref, "b");
        let lt_expr = InequalityExpr::try_new(Box::new(lhs), Box::new(rhs), true).unwrap();

        assert_eq!(lt_expr.data_type(), ColumnType::Boolean);
        assert!(lt_expr.is_lt());
        assert_eq!(lt_expr.lhs().data_type(), ColumnType::BigInt);
        assert_eq!(lt_expr.rhs().data_type(), ColumnType::BigInt);
        assert_eq!(
            lt_expr.first_round_evaluate(&alloc, &table, &[]).unwrap(),
            Column::Boolean(&[true, false, false, true])
        );

        let (_, lhs_for_gt) = bigint_column(&table_ref, "a");
        let (_, rhs_for_gt) = bigint_column(&table_ref, "b");
        let gt_expr =
            InequalityExpr::try_new(Box::new(lhs_for_gt), Box::new(rhs_for_gt), false).unwrap();
        assert_eq!(
            gt_expr.first_round_evaluate(&alloc, &table, &[]).unwrap(),
            Column::Boolean(&[false, true, false, false])
        );

        let mut final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
        assert_eq!(
            lt_expr
                .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
                .unwrap(),
            Column::Boolean(&[true, false, false, true])
        );
        assert_eq!(final_round_builder.bit_distributions().len(), 1);

        let first_round_builder: FirstRoundBuilder<'_, TestScalar> = FirstRoundBuilder::new(4);
        let verification_builder = run_verify_for_each_row(
            4,
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            3,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    lhs_ref.clone().column_id() => lhs_values.as_slice().inner_product(evaluation_point),
                    rhs_ref.clone().column_id() => rhs_values.as_slice().inner_product(evaluation_point),
                };
                lt_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                    .unwrap();
            },
        );
        assert!(verification_builder
            .get_identity_results()
            .iter()
            .flatten()
            .all(|is_valid| *is_valid));

        let mut gt_final_round_builder = FinalRoundBuilder::new(4, VecDeque::new());
        assert_eq!(
            gt_expr
                .final_round_evaluate(&mut gt_final_round_builder, &alloc, &table, &[])
                .unwrap(),
            Column::Boolean(&[false, true, false, false])
        );
        let gt_verification_builder = run_verify_for_each_row(
            4,
            &first_round_builder,
            &gt_final_round_builder,
            Vec::new(),
            3,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    lhs_ref.clone().column_id() => lhs_values.as_slice().inner_product(evaluation_point),
                    rhs_ref.clone().column_id() => rhs_values.as_slice().inner_product(evaluation_point),
                };
                gt_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                    .unwrap();
            },
        );
        assert!(gt_verification_builder
            .get_identity_results()
            .iter()
            .flatten()
            .all(|is_valid| *is_valid));

        let mut columns = IndexSet::default();
        lt_expr.get_column_references(&mut columns);
        assert_eq!(columns, indexset! { lhs_ref, rhs_ref });
    }
}
