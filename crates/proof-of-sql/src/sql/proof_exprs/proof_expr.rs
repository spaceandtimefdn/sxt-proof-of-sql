use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        math::decimal::Precision,
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use core::fmt::Debug;
use sqlparser::ast::Ident;

/// Provable AST column expression that evaluates to a `Column`
#[enum_dispatch::enum_dispatch(DynProofExpr)]
pub trait ProofExpr: Debug + Send + Sync {
    /// Get the data type of the expression
    fn data_type(&self) -> ColumnType;

    /// This returns the result of evaluating the expression on the given table, and returns
    /// a column of values. This result slice is guaranteed to have length `table_length`.
    /// Implementations must ensure that the returned slice has length `table_length`.
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>>;

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of values
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>>;

    /// Compute the evaluation of a multilinear extension from this expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// [`VerificationBuilder<S>`]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError>;

    /// Insert in the [`IndexSet`] `columns` all the column
    /// references in the `BoolExpr` or forwards the call to some
    /// subsequent `bool_expr`
    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>);
}

/// A trait for `ProofExpr`s that always return a decimal type
pub(crate) trait DecimalProofExpr: ProofExpr {
    /// Get the precision of the expression
    ///
    /// # Panics
    /// This panics if the precision is invalid
    fn precision(&self) -> Precision {
        Precision::new(
            self.data_type()
                .precision_value()
                .expect("Precision should be valid"),
        )
        .expect("Precision should be valid")
    }

    /// Get the scale of the expression
    ///
    /// # Panics
    /// This panics if the scale is invalid
    fn scale(&self) -> i8 {
        self.data_type().scale().expect("Scale should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::{DecimalProofExpr, ProofExpr};
    use crate::{
        base::{
            database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
            map::{IndexMap, IndexSet},
            math::decimal::Precision,
            proof::{PlaceholderResult, ProofError},
            scalar::Scalar,
        },
        sql::proof::{FinalRoundBuilder, VerificationBuilder},
    };
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    #[derive(Debug)]
    struct TestDecimalExpr(ColumnType);

    impl ProofExpr for TestDecimalExpr {
        fn data_type(&self) -> ColumnType {
            self.0
        }

        fn first_round_evaluate<'a, S: Scalar>(
            &self,
            _alloc: &'a Bump,
            _table: &Table<'a, S>,
            _params: &[LiteralValue],
        ) -> PlaceholderResult<Column<'a, S>> {
            unimplemented!("not needed for DecimalProofExpr accessors")
        }

        fn final_round_evaluate<'a, S: Scalar>(
            &self,
            _builder: &mut FinalRoundBuilder<'a, S>,
            _alloc: &'a Bump,
            _table: &Table<'a, S>,
            _params: &[LiteralValue],
        ) -> PlaceholderResult<Column<'a, S>> {
            unimplemented!("not needed for DecimalProofExpr accessors")
        }

        fn verifier_evaluate<S: Scalar>(
            &self,
            _builder: &mut impl VerificationBuilder<S>,
            _accessor: &IndexMap<Ident, S>,
            _chi_eval: S,
            _params: &[LiteralValue],
        ) -> Result<S, ProofError> {
            unimplemented!("not needed for DecimalProofExpr accessors")
        }

        fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
    }

    impl DecimalProofExpr for TestDecimalExpr {}

    #[test]
    fn decimal_proof_expr_uses_decimal75_precision_and_scale() {
        let precision = Precision::new(18).unwrap();
        let expr = TestDecimalExpr(ColumnType::Decimal75(precision, 6));

        assert_eq!(expr.precision(), precision);
        assert_eq!(expr.scale(), 6);
    }

    #[test]
    fn decimal_proof_expr_uses_integer_precision_with_zero_scale() {
        let expr = TestDecimalExpr(ColumnType::BigInt);

        assert_eq!(expr.precision(), Precision::new(19).unwrap());
        assert_eq!(expr.scale(), 0);
    }
}
