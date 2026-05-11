use super::ProofExpr;
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderError, PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable placeholder expression, that is, a placeholder in a SQL query
///
/// This node allows us to easily represent queries like
///    select $1, $2 from T
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceholderExpr {
    index: usize,
    column_type: ColumnType,
}

impl PlaceholderExpr {
    /// Creates a new `PlaceholderExpr`
    ///
    /// # Errors
    ///
    /// Returns [`PlaceholderError::ZeroPlaceholderId`] if `id` is 0.
    /// Placeholder IDs must be greater than 0 following `PostgreSQL` convention.
    pub fn try_new(id: usize, column_type: ColumnType) -> PlaceholderResult<Self> {
        (id > 0)
            .then(|| Self {
                index: id - 1,
                column_type,
            })
            .ok_or(PlaceholderError::ZeroPlaceholderId)
    }

    /// Creates a new `PlaceholderExpr` from an index directly.
    ///
    /// This is an infallible constructor that takes the internal index representation
    /// (0-based) rather than the PostgreSQL-style ID (1-based).
    pub(crate) fn new_from_index(index: usize, column_type: ColumnType) -> Self {
        Self { index, column_type }
    }

    /// Get the id of the placeholder
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the column type of the placeholder
    pub fn column_type(&self) -> ColumnType {
        self.column_type
    }

    /// Replace the placeholder with the correct value in `params`.
    ///
    /// Following `PostgreSQL` convention id starts from 1, so the first placeholder has id 1.
    ///
    /// Note that this function will return an error if
    /// 1. The placeholder id is out of bounds
    /// 2. The placeholder type does not match the type of the value in `params`
    fn interpolate<'a>(
        &self,
        params: &'a [LiteralValue],
    ) -> Result<&'a LiteralValue, PlaceholderError> {
        let pos = self.index;
        let param_value = params
            .get(pos)
            .ok_or(PlaceholderError::InvalidPlaceholderIndex {
                index: self.index,
                num_params: params.len(),
            })?;
        if param_value.column_type() != self.column_type {
            return Err(PlaceholderError::InvalidPlaceholderType {
                index: self.index,
                expected: self.column_type,
                actual: param_value.column_type(),
            });
        }
        Ok(param_value)
    }
}

impl ProofExpr for PlaceholderExpr {
    fn data_type(&self) -> ColumnType {
        self.column_type
    }

    #[tracing::instrument(
        name = "PlaceholderExpr::first_round_evaluate",
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

        let param_value = self.interpolate(params)?;
        let res = Column::from_literal_with_length(param_value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "PlaceholderExpr::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let param_value = self.interpolate(params)?;
        let res = Column::from_literal_with_length(param_value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let param_value = self.interpolate(params)?;
        Ok(chi_eval * param_value.to_scalar::<S>())
    }

    fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::database::table_utility::{borrowed_bigint, table},
        base::map::{IndexMap, IndexSet},
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
        sql::proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
    };
    use alloc::collections::VecDeque;
    use sqlparser::ast::Ident;

    // new
    #[test]
    fn we_cannot_create_a_placeholder_with_zero_id() {
        let res = PlaceholderExpr::try_new(0, ColumnType::Boolean);
        assert!(matches!(res, Err(PlaceholderError::ZeroPlaceholderId)));
    }

    #[test]
    fn we_can_create_a_placeholder_from_index() {
        let placeholder = PlaceholderExpr::new_from_index(5, ColumnType::BigInt);
        assert_eq!(placeholder.index(), 5);
        assert_eq!(placeholder.column_type(), ColumnType::BigInt);
    }

    // interpolate
    #[test]
    fn we_cannot_interpolate_placeholder_if_id_is_out_of_bounds() {
        // Empty params
        let placeholder_expr = PlaceholderExpr::try_new(1, ColumnType::Boolean).unwrap();
        let params = vec![];
        let res = placeholder_expr.interpolate(&params);
        assert!(matches!(
            res,
            Err(PlaceholderError::InvalidPlaceholderIndex { .. })
        ));

        // Params exist but not enough of them
        let placeholder_expr = PlaceholderExpr::try_new(3, ColumnType::Boolean).unwrap();
        let params = vec![LiteralValue::Boolean(true), LiteralValue::Boolean(false)];
        let res = placeholder_expr.interpolate(&params);
        assert!(matches!(
            res,
            Err(PlaceholderError::InvalidPlaceholderIndex { .. })
        ));
    }

    #[test]
    fn we_cannot_interpolate_placeholder_if_types_do_not_match() {
        let placeholder_expr = PlaceholderExpr::try_new(1, ColumnType::Boolean).unwrap();
        let params = vec![LiteralValue::BigInt(123)];
        let res = placeholder_expr.interpolate(&params);
        assert!(matches!(
            res,
            Err(PlaceholderError::InvalidPlaceholderType { .. })
        ));
    }

    #[test]
    fn we_can_interpolate_placeholder_if_id_is_in_bounds_and_types_match() {
        let placeholder_expr = PlaceholderExpr::try_new(1, ColumnType::Boolean).unwrap();
        let params = vec![LiteralValue::Boolean(true)];
        let res = placeholder_expr.interpolate(&params);
        assert_eq!(res.unwrap(), &LiteralValue::Boolean(true));
    }

    #[test]
    fn we_can_evaluate_placeholder_expr_directly() {
        let alloc = Bump::new();
        let data: Table<Curve25519Scalar> = table([borrowed_bigint("a", [1_i64, 2, 3, 4], &alloc)]);
        let expr = PlaceholderExpr::try_new(1, ColumnType::BigInt).unwrap();
        let params = [LiteralValue::BigInt(11)];

        assert_eq!(expr.data_type(), ColumnType::BigInt);

        let first_round = expr.first_round_evaluate(&alloc, &data, &params).unwrap();
        assert_eq!(first_round, Column::BigInt(&[11, 11, 11, 11]));

        let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &data, &params)
            .unwrap();
        assert_eq!(final_round, Column::BigInt(&[11, 11, 11, 11]));
    }

    #[test]
    fn we_can_verify_placeholder_expr_directly() {
        let expr = PlaceholderExpr::try_new(1, ColumnType::BigInt).unwrap();
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
        let params = [LiteralValue::BigInt(7)];

        let res = expr
            .verifier_evaluate(&mut builder, &accessor, Curve25519Scalar::from(3), &params)
            .unwrap();

        assert_eq!(res, Curve25519Scalar::from(21));
    }

    #[test]
    fn placeholder_expr_has_no_column_references() {
        let expr = PlaceholderExpr::try_new(1, ColumnType::Boolean).unwrap();
        let mut columns = IndexSet::default();

        expr.get_column_references(&mut columns);

        assert!(columns.is_empty());
    }
}
