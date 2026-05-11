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
            database::{table_utility::*, Column, ColumnField, TableRef},
            map::{IndexMap, IndexSet},
        },
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
        sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
    };
    use alloc::collections::VecDeque;

    #[test]
    fn we_can_evaluate_column_expr_directly() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "t");
        let column_ref = ColumnRef::new(table_ref, "a".into(), ColumnType::BigInt);
        let expr = ColumnExpr::new(column_ref.clone());
        let data: Table<Curve25519Scalar> = table([borrowed_bigint("a", [5_i64, 8, 13], &alloc)]);

        assert_eq!(expr.get_column_reference(), column_ref);
        assert_eq!(expr.column_ref(), &column_ref);
        assert_eq!(
            expr.get_column_field(),
            ColumnField::new("a".into(), ColumnType::BigInt)
        );
        assert_eq!(expr.column_id(), Ident::new("a"));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
        assert_eq!(expr.fetch_column(&data), Column::BigInt(&[5, 8, 13]));

        let first_round = expr.first_round_evaluate(&alloc, &data, &[]).unwrap();
        assert_eq!(first_round, Column::BigInt(&[5, 8, 13]));

        let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &data, &[])
            .unwrap();
        assert_eq!(final_round, Column::BigInt(&[5, 8, 13]));
    }

    #[test]
    fn we_can_verify_column_expr_directly() {
        let table_ref = TableRef::new("sxt", "t");
        let column_ref = ColumnRef::new(table_ref, "a".into(), ColumnType::BigInt);
        let expr = ColumnExpr::new(column_ref.clone());
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
        accessor.insert("a".into(), Curve25519Scalar::from(987));

        let res = expr
            .verifier_evaluate(&mut builder, &accessor, Curve25519Scalar::from(123), &[])
            .unwrap();

        assert_eq!(res, Curve25519Scalar::from(987));
    }

    #[test]
    fn column_expr_records_its_column_reference() {
        let table_ref = TableRef::new("sxt", "t");
        let column_ref = ColumnRef::new(table_ref, "a".into(), ColumnType::BigInt);
        let expr = ColumnExpr::new(column_ref.clone());
        let mut columns = IndexSet::default();

        expr.get_column_references(&mut columns);

        assert_eq!(columns.len(), 1);
        assert!(columns.contains(&column_ref));
    }
}
