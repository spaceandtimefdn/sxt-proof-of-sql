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
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{table_utility::table_with_row_count, ColumnType, LiteralValue, TableRef},
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec};
    use sqlparser::ast::Ident;

    #[test]
    fn cast_expr_casts_literal_columns_without_blitzar() {
        let alloc = Bump::new();
        let table = table_with_row_count::<TestScalar>([], 2);
        let cast_expr = CastExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::Boolean(true))),
            ColumnType::BigInt,
        )
        .unwrap();

        assert_eq!(cast_expr.data_type(), ColumnType::BigInt);
        assert_eq!(cast_expr.to_type(), &ColumnType::BigInt);
        assert_eq!(cast_expr.get_from_expr().data_type(), ColumnType::Boolean);

        let first_round = cast_expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round, Column::BigInt(&[1_i64, 1]));

        let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
        let final_round = cast_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, Column::BigInt(&[1_i64, 1]));
    }

    #[test]
    fn cast_expr_delegates_verifier_and_column_refs_without_blitzar() {
        let cast_expr = CastExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::TinyInt(7))),
            ColumnType::BigInt,
        )
        .unwrap();
        let mut verifier = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            0,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let eval = cast_expr
            .verifier_evaluate(&mut verifier, &IndexMap::default(), 3_i64.into(), &[])
            .unwrap();
        assert_eq!(eval, TestScalar::from(21_i64));

        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "casts"),
            Ident::new("flag"),
            ColumnType::Boolean,
        );
        let cast_expr = CastExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref.clone())),
            ColumnType::BigInt,
        )
        .unwrap();
        let mut column_refs = IndexSet::default();

        cast_expr.get_column_references(&mut column_refs);

        assert_eq!(column_refs.len(), 1);
        assert!(column_refs.contains(&column_ref));
    }

    #[test]
    fn cast_expr_rejects_unsupported_casts_without_blitzar() {
        let error = CastExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::VarChar(
                "not_numeric".into(),
            ))),
            ColumnType::BigInt,
        )
        .unwrap_err();

        assert!(matches!(error, AnalyzeError::DataTypeMismatch { .. }));
    }
}
