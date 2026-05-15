use super::{add_subtract_columns, DecimalProofExpr, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{
            try_add_subtract_column_types, Column, ColumnRef, ColumnType, LiteralValue, Table,
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

/// Provable numerical `+` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl AddExpr {
    /// Create numerical `+` expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_add_subtract_column_types(left_datatype, right_datatype)
            .map(|_| Self { lhs, rhs })
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
}

impl ProofExpr for AddExpr {
    fn data_type(&self) -> ColumnType {
        try_add_subtract_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to add/subtract column types")
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, false);
        Ok(Column::Decimal75(self.precision(), self.scale(), res))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.add_expr.final_round_evaluate",
        level = "info",
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

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, false);
        log::log_memory_usage("End");

        Ok(Column::Decimal75(self.precision(), self.scale(), res))
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
        Ok(lhs_eval + rhs_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

impl DecimalProofExpr for AddExpr {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{
                table_utility::{borrowed_bigint, borrowed_varchar, table},
                TableRef,
            },
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec::Vec};
    use bumpalo::Bump;

    fn bigint_column_expr(table_ref: &TableRef, name: &str) -> (ColumnRef, DynProofExpr) {
        let column_ref = ColumnRef::new(table_ref.clone(), name.into(), ColumnType::BigInt);
        (column_ref.clone(), DynProofExpr::new_column(column_ref))
    }

    #[test]
    fn add_expr_evaluates_column_and_literal_without_blitzar() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "t");
        let table = table([borrowed_bigint("a", [1_i64, 2, 3], &alloc)]);
        let (column_ref, lhs) = bigint_column_expr(&table_ref, "a");
        let rhs = DynProofExpr::new_literal(LiteralValue::BigInt(4));
        let expr = AddExpr::try_new(Box::new(lhs), Box::new(rhs)).unwrap();

        assert_eq!(
            expr.data_type(),
            ColumnType::Decimal75(expr.precision(), expr.scale())
        );

        let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(
            first_round,
            Column::Decimal75(
                expr.precision(),
                expr.scale(),
                &[
                    TestScalar::from(5),
                    TestScalar::from(6),
                    TestScalar::from(7)
                ]
            )
        );

        let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, first_round);
        assert!(final_round_builder.pcs_proof_mles().is_empty());
        assert!(final_round_builder.sumcheck_subpolynomials().is_empty());

        let mut accessor = IndexMap::default();
        accessor.insert("a".into(), TestScalar::from(9));
        let mut verification_builder = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let verifier_eval = expr
            .verifier_evaluate(
                &mut verification_builder,
                &accessor,
                TestScalar::from(3),
                &[],
            )
            .unwrap();
        assert_eq!(verifier_eval, TestScalar::from(21));

        let mut columns = IndexSet::default();
        expr.get_column_references(&mut columns);
        assert_eq!(columns.len(), 1);
        assert!(columns.contains(&column_ref));
    }

    #[test]
    fn add_expr_rejects_non_numeric_inputs_without_blitzar() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "t");
        let table = table([borrowed_varchar::<TestScalar>("name", ["a", "b"], &alloc)]);
        let column_ref = ColumnRef::new(table_ref, "name".into(), ColumnType::VarChar);
        let lhs = DynProofExpr::new_column(column_ref);
        let rhs = DynProofExpr::new_literal(LiteralValue::BigInt(1));

        let err = AddExpr::try_new(Box::new(lhs), Box::new(rhs)).unwrap_err();
        assert!(matches!(err, AnalyzeError::DataTypeMismatch { .. }));
        assert_eq!(table.num_rows(), 2);
    }
}
