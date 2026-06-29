use super::ProofExpr;
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable CONST expression
///
/// This node allows us to easily represent queries like
///    select * from T
/// and
///    select * from T where 1 = 2
/// as filter expressions with a constant where clause.
///
/// While this wouldn't be as efficient as using a new custom expression for
/// such queries, it allows us to easily support projects with minimal code
/// changes, and the performance is sufficient for present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralExpr {
    value: LiteralValue,
}

impl LiteralExpr {
    /// Create literal expression
    pub fn new(value: LiteralValue) -> Self {
        Self { value }
    }

    /// Get the literal value
    pub fn value(&self) -> &LiteralValue {
        &self.value
    }
}

impl ProofExpr for LiteralExpr {
    fn data_type(&self) -> ColumnType {
        self.value.column_type()
    }

    #[tracing::instrument(name = "LiteralExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let res = Column::from_literal_with_length(&self.value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "LiteralExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let table_length = table.num_rows();
        let res = Column::from_literal_with_length(&self.value, table_length, alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        _params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        Ok(chi_eval * self.value.to_scalar())
    }

    fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{database::TableOptions, scalar::test_scalar::TestScalar},
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec::Vec};

    fn empty_table_with_rows<'a>(rows: usize) -> Table<'a, TestScalar> {
        Table::try_new_with_options(IndexMap::default(), TableOptions::new(Some(rows))).unwrap()
    }

    #[test]
    fn literal_expr_reports_type_and_keeps_value() {
        let value = LiteralValue::Int(42);
        let expr = LiteralExpr::new(value.clone());

        assert_eq!(expr.value(), &value);
        assert_eq!(expr.data_type(), ColumnType::Int);
    }

    #[test]
    fn literal_expr_evaluates_to_constant_columns() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(-7));
        let alloc = Bump::new();
        let table = empty_table_with_rows(3);

        let first_round = expr
            .first_round_evaluate(&alloc, &table, &[])
            .expect("literal first round evaluation should succeed");
        assert_eq!(first_round, Column::BigInt(&[-7, -7, -7]));

        let mut builder = FinalRoundBuilder::new(0, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut builder, &alloc, &table, &[])
            .expect("literal final round evaluation should succeed");
        assert_eq!(final_round, Column::BigInt(&[-7, -7, -7]));
    }

    #[test]
    fn literal_expr_verifier_multiplies_chi_by_literal_scalar() {
        let expr = LiteralExpr::new(LiteralValue::Int(6));
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let chi_eval = TestScalar::from(7_u8);

        let result = expr
            .verifier_evaluate(&mut builder, &IndexMap::default(), chi_eval, &[])
            .unwrap();

        assert_eq!(result, TestScalar::from(42_u8));
    }

    #[test]
    fn literal_expr_does_not_reference_columns() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(true));
        let mut columns = IndexSet::default();

        expr.get_column_references(&mut columns);

        assert!(columns.is_empty());
    }
}
