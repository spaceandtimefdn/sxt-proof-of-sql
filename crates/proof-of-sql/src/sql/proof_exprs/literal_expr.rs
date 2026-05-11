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
        base::{
            database::{table_utility::*, Column},
            map::{IndexMap, IndexSet},
        },
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
        sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
    };
    use alloc::collections::VecDeque;
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_get_literal_expr_value_and_data_type() {
        let literal = LiteralValue::Int(123);
        let expr = LiteralExpr::new(literal.clone());

        assert_eq!(expr.value(), &literal);
        assert_eq!(expr.data_type(), ColumnType::Int);
    }

    #[test]
    fn we_can_evaluate_literal_expr_directly() {
        let alloc = Bump::new();
        let data: Table<Curve25519Scalar> = table([borrowed_bigint("a", [1_i64, 2, 3, 4], &alloc)]);
        let expr = LiteralExpr::new(LiteralValue::BigInt(11));

        let first_round = expr.first_round_evaluate(&alloc, &data, &[]).unwrap();
        assert_eq!(first_round, Column::BigInt(&[11, 11, 11, 11]));

        let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &data, &[])
            .unwrap();
        assert_eq!(final_round, Column::BigInt(&[11, 11, 11, 11]));
    }

    #[test]
    fn we_can_verify_literal_expr_directly() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(7));
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let accessor = IndexMap::<Ident, Curve25519Scalar>::default();

        let res = expr
            .verifier_evaluate(&mut builder, &accessor, Curve25519Scalar::from(3), &[])
            .unwrap();

        assert_eq!(res, Curve25519Scalar::from(21));
    }

    #[test]
    fn literal_expr_has_no_column_references() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(true));
        let mut columns = IndexSet::default();

        expr.get_column_references(&mut columns);

        assert!(columns.is_empty());
    }
}
