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
            database::{table_utility::*, TableRef},
            map::{indexmap, indexset},
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::{mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder},
            proof_exprs::ColumnExpr,
        },
    };
    use alloc::{boxed::Box, collections::VecDeque};

    #[test]
    fn try_new_reports_cast_type_mismatch() {
        let err = CastExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(1))),
            ColumnType::Boolean,
        )
        .unwrap_err();

        assert_eq!(
            err,
            AnalyzeError::DataTypeMismatch {
                left_type: "BIGINT".into(),
                right_type: "BOOLEAN".into()
            }
        );
    }

    #[test]
    fn cast_expr_casts_columns_and_forwards_verifier_evaluation() {
        let alloc = Bump::new();
        let table = table::<TestScalar>([borrowed_uint8("a", [1_u8, 2, 255], &alloc)]);
        let table_ref = TableRef::new("sxt", "t");
        let column_ref = ColumnRef::new(table_ref, "a".into(), ColumnType::Uint8);
        let cast_expr = CastExpr::try_new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref.clone()))),
            ColumnType::BigInt,
        )
        .unwrap();

        assert_eq!(cast_expr.get_from_expr().data_type(), ColumnType::Uint8);
        assert_eq!(cast_expr.to_type(), &ColumnType::BigInt);
        assert_eq!(cast_expr.data_type(), ColumnType::BigInt);
        assert_eq!(
            cast_expr.first_round_evaluate(&alloc, &table, &[]).unwrap(),
            Column::BigInt(&[1_i64, 2, 255])
        );

        let mut builder = FinalRoundBuilder::new(3, VecDeque::new());
        assert_eq!(
            cast_expr
                .final_round_evaluate(&mut builder, &alloc, &table, &[])
                .unwrap(),
            Column::BigInt(&[1_i64, 2, 255])
        );

        let mut verifier = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let expected_eval = TestScalar::from(42);
        let accessor = indexmap! { column_ref.column_id() => expected_eval };
        assert_eq!(
            cast_expr
                .verifier_evaluate(&mut verifier, &accessor, TestScalar::from(3), &[])
                .unwrap(),
            expected_eval
        );

        let mut columns = IndexSet::default();
        cast_expr.get_column_references(&mut columns);
        assert_eq!(columns, indexset! { column_ref });
    }
}
