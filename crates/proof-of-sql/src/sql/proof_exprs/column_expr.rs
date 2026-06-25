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
    use super::ColumnExpr;
    use crate::base::database::{ColumnRef, ColumnType, TableRef};
    use sqlparser::ast::Ident;

    fn make_col_ref() -> ColumnRef {
        ColumnRef::new(TableRef::new("s", "t"), Ident::new("col"), ColumnType::BigInt)
    }

    #[test]
    fn new_stores_column_ref() {
        let e = ColumnExpr::new(make_col_ref());
        assert_eq!(e.column_ref(), &make_col_ref());
    }

    #[test]
    fn get_column_reference_returns_clone() {
        let e = ColumnExpr::new(make_col_ref());
        assert_eq!(e.get_column_reference(), make_col_ref());
    }

    #[test]
    fn column_id_returns_ident() {
        let e = ColumnExpr::new(make_col_ref());
        assert_eq!(e.column_id().value.as_str(), "col");
    }

    #[test]
    fn get_column_field_has_correct_type() {
        let e = ColumnExpr::new(make_col_ref());
        assert_eq!(e.get_column_field().data_type(), ColumnType::BigInt);
    }

    #[test]
    fn equality_holds() {
        let a = ColumnExpr::new(make_col_ref());
        let b = ColumnExpr::new(make_col_ref());
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e = ColumnExpr::new(make_col_ref());
        assert!(alloc::format!("{e:?}").contains("ColumnExpr"));
    }
}
