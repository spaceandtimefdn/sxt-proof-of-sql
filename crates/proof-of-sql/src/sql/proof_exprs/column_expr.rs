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
        base::{
            database::{
                table_utility::{borrowed_int, table},
                Column, ColumnField, TableRef,
            },
            map::{IndexMap, IndexSet},
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::collections::VecDeque;

    fn column_expr() -> (ColumnExpr, ColumnRef) {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "quantity".into(),
            ColumnType::Int,
        );
        (ColumnExpr::new(column_ref.clone()), column_ref)
    }

    #[test]
    fn we_can_inspect_column_expr_metadata_and_references() {
        let (expr, column_ref) = column_expr();

        assert_eq!(expr.get_column_reference(), column_ref);
        assert_eq!(expr.column_ref(), &column_ref);
        assert_eq!(
            expr.get_column_field(),
            ColumnField::new("quantity".into(), ColumnType::Int)
        );
        assert_eq!(expr.column_id(), "quantity".into());
        assert_eq!(expr.data_type(), ColumnType::Int);

        let mut columns = IndexSet::default();
        expr.get_column_references(&mut columns);
        assert_eq!(columns.len(), 1);
        assert!(columns.contains(&column_ref));
    }

    #[test]
    fn we_can_evaluate_column_expr_rounds() {
        let alloc = Bump::new();
        let data = table([borrowed_int("quantity", [3, 5, 8], &alloc)]);
        let (expr, _) = column_expr();
        let expected = Column::Int(&[3, 5, 8]);

        assert_eq!(expr.fetch_column(&data), expected);
        assert_eq!(
            expr.first_round_evaluate(&alloc, &data, &[]).unwrap(),
            expected
        );

        let mut builder = FinalRoundBuilder::<TestScalar>::new(2, VecDeque::new());
        assert_eq!(
            expr.final_round_evaluate(&mut builder, &alloc, &data, &[])
                .unwrap(),
            expected
        );
    }

    #[test]
    fn we_can_verify_column_expr_from_accessor() {
        let (expr, _) = column_expr();
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        let mut accessor = IndexMap::default();
        accessor.insert("quantity".into(), TestScalar::from(13_u64));

        assert_eq!(
            expr.verifier_evaluate(&mut builder, &accessor, TestScalar::from(7_u64), &[])
                .unwrap(),
            TestScalar::from(13_u64)
        );
    }

    #[test]
    fn we_cannot_verify_column_expr_without_accessor_value() {
        let (expr, _) = column_expr();
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        let accessor = IndexMap::default();

        assert!(matches!(
            expr.verifier_evaluate(&mut builder, &accessor, TestScalar::from(7_u64), &[]),
            Err(ProofError::VerificationError {
                error: "Column Not Found"
            })
        ));
    }
}
