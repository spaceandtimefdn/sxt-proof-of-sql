use super::ProofExpr;
use crate::{
    base::{
        database::{Column, ColumnField, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;
/// Provable expression for a column
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ColumnExpr {
    column_ref: ColumnRef,
}

impl ColumnExpr {
    /// Create a new column expression
    #[must_use]
    pub fn new(column_ref: ColumnRef) -> Self {
        Self { column_ref }
    }

    /// Return the column referenced by this [`ColumnExpr`]
    #[must_use]
    pub fn get_column_reference(&self) -> ColumnRef {
        self.column_ref.clone()
    }

    /// Get the column reference
    #[must_use]
    pub fn column_ref(&self) -> &ColumnRef {
        &self.column_ref
    }

    /// Wrap the column output name and its type within the [`ColumnField`]
    #[must_use]
    pub fn get_column_field(&self) -> ColumnField {
        ColumnField::new(self.column_ref.column_id(), *self.column_ref.column_type())
    }

    /// Get the column identifier
    #[must_use]
    pub fn column_id(&self) -> Ident {
        self.column_ref.column_id()
    }

    /// Get the column
    /// # Panics
    ///
    /// Will panic if the column is not found. Shouldn't happen in practice since
    /// code in `sql/parse` should have already checked that the column exists.
    #[must_use]
    pub fn fetch_column<'a, S: Scalar>(&self, table: &Table<'a, S>) -> Column<'a, S> {
        *table
            .inner_table()
            .get(&self.column_ref.column_id())
            .expect("Column not found")
    }
}

impl ProofExpr for ColumnExpr {
    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType {
        *self.get_column_reference().column_type()
    }

    /// Evaluate the column expression and
    /// add the result to the [`FirstRoundBuilder`](crate::sql::proof::FirstRoundBuilder)
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _alloc: &'a Bump,
        table: &Table<'a, S>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        Ok(self.fetch_column(table))
    }

    /// Given the selected rows (as a slice of booleans), evaluate the column expression and
    /// add the components needed to prove the result
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table: &Table<'a, S>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        Ok(self.fetch_column(table))
    }

    /// Evaluate the column expression at the sumcheck's random point,
    /// add components needed to verify this column expression
    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        _chi_eval: S,
        _params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        Ok(*accessor
            .get(&self.column_ref.column_id())
            .ok_or(ProofError::VerificationError {
                error: "Column Not Found",
            })?)
    }

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        columns.insert(self.column_ref.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{database::TableRef, math::decimal::Precision, scalar::test_scalar::TestScalar},
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec::Vec};

    fn column_ref(name: &str, column_type: ColumnType) -> ColumnRef {
        ColumnRef::new(
            TableRef::new("sxt", "blocks"),
            Ident::new(name),
            column_type,
        )
    }

    #[test]
    fn column_expr_exposes_its_reference_and_field_metadata() {
        let reference = column_ref("block_number", ColumnType::BigInt);
        let expr = ColumnExpr::new(reference.clone());

        assert_eq!(expr.get_column_reference(), reference);
        assert_eq!(expr.column_ref(), &reference);
        assert_eq!(expr.column_id(), Ident::new("block_number"));
        assert_eq!(
            expr.get_column_field(),
            ColumnField::new("block_number".into(), ColumnType::BigInt)
        );
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn column_expr_evaluates_by_fetching_the_referenced_table_column() {
        let expr = ColumnExpr::new(column_ref("flag", ColumnType::Boolean));
        let table: Table<TestScalar> =
            Table::try_from_iter([(Ident::new("flag"), Column::Boolean(&[true, false]))]).unwrap();
        let alloc = Bump::new();

        assert_eq!(expr.fetch_column(&table), Column::Boolean(&[true, false]));
        assert_eq!(
            expr.first_round_evaluate(&alloc, &table, &[]).unwrap(),
            Column::Boolean(&[true, false])
        );

        let mut builder = FinalRoundBuilder::new(0, VecDeque::new());
        assert_eq!(
            expr.final_round_evaluate(&mut builder, &alloc, &table, &[])
                .unwrap(),
            Column::Boolean(&[true, false])
        );
    }

    #[test]
    fn column_expr_verifier_reads_the_column_evaluation_or_errors() {
        let expr = ColumnExpr::new(column_ref("score", ColumnType::Int));
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let mut accessor = IndexMap::default();
        accessor.insert(Ident::new("score"), TestScalar::from(99_u8));

        assert_eq!(
            expr.verifier_evaluate(&mut builder, &accessor, TestScalar::from(1_u8), &[])
                .unwrap(),
            TestScalar::from(99_u8)
        );

        let error = expr
            .verifier_evaluate(
                &mut builder,
                &IndexMap::default(),
                TestScalar::from(1_u8),
                &[],
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ProofError::VerificationError {
                error: "Column Not Found"
            }
        ));
    }

    #[test]
    fn column_expr_reports_its_column_reference() {
        let reference = column_ref(
            "amount",
            ColumnType::Decimal75(Precision::new(9).unwrap(), 2),
        );
        let expr = ColumnExpr::new(reference.clone());
        let mut columns = IndexSet::default();

        expr.get_column_references(&mut columns);

        assert!(columns.contains(&reference));
        assert_eq!(columns.len(), 1);
    }
}
