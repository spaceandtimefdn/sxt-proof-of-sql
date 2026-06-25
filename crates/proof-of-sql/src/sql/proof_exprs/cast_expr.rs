use super::{numerical_util::cast_column, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_cast_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable CAST expression
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CastExpr {
    from_expr: Box<DynProofExpr>,
    to_type: ColumnType,
}

impl CastExpr {
    /// Creates a new `CastExpr`
    pub fn try_new(from_expr: Box<DynProofExpr>, to_type: ColumnType) -> AnalyzeResult<Self> {
        let from_datatype = from_expr.data_type();
        try_cast_types(from_datatype, to_type)
            .map(|()| Self { from_expr, to_type })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: from_datatype.to_string(),
                right_type: to_type.to_string(),
            })
    }

    /// Returns the from expression
    pub fn get_from_expr(&self) -> &DynProofExpr {
        &self.from_expr
    }

    /// Returns the to type
    pub fn to_type(&self) -> &ColumnType {
        &self.to_type
    }
}

impl ProofExpr for CastExpr {
    fn data_type(&self) -> ColumnType {
        self.to_type
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self.from_expr.first_round_evaluate(alloc, table, params)?;
        Ok(cast_column(
            alloc,
            uncasted_result,
            self.from_expr.data_type(),
            self.to_type,
        ))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let uncasted_result = self
            .from_expr
            .final_round_evaluate(builder, alloc, table, params)?;
        Ok(cast_column(
            alloc,
            uncasted_result,
            self.from_expr.data_type(),
            self.to_type,
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        self.from_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.from_expr.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests_cast {
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::proof_exprs::{CastExpr, DynProofExpr, ProofExpr},
    };

    fn bool_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::Boolean(true))
    }
    fn tinyint_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::TinyInt(1))
    }

    #[test]
    fn try_new_bool_to_bigint_returns_ok() {
        assert!(
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::BigInt).is_ok()
        );
    }

    #[test]
    fn try_new_bigint_to_boolean_returns_err() {
        assert!(
            CastExpr::try_new(
                alloc::boxed::Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(1))),
                ColumnType::Boolean
            )
            .is_err()
        );
    }

    #[test]
    fn data_type_returns_to_type() {
        let e =
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::BigInt).unwrap();
        assert_eq!(e.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn to_type_accessor() {
        let e =
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::Int128).unwrap();
        assert_eq!(*e.to_type(), ColumnType::Int128);
    }

    #[test]
    fn from_expr_has_correct_type() {
        let e =
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::BigInt).unwrap();
        assert_eq!(e.get_from_expr().data_type(), ColumnType::Boolean);
    }

    #[test]
    fn equality_holds() {
        let a =
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::BigInt).unwrap();
        let b =
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::BigInt).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e =
            CastExpr::try_new(alloc::boxed::Box::new(bool_expr()), ColumnType::BigInt).unwrap();
        assert!(alloc::format!("{e:?}").contains("CastExpr"));
    }
}
