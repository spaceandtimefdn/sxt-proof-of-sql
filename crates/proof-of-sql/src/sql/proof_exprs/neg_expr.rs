use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_neg_type, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable numerical `-` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NegExpr {
    input_expr: Box<DynProofExpr>,
}

impl NegExpr {
    /// Create numerical `-` expression
    pub fn try_new(input_expr: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let input_data_type = input_expr.data_type();
        try_neg_type(input_data_type)
            .map(|_| Self { input_expr })
            .map_err(|_| AnalyzeError::InvalidDataType {
                expr_type: input_data_type,
            })
    }

    pub(crate) fn input(&self) -> &DynProofExpr {
        &self.input_expr
    }
}

/// # Panics
///
/// Panics if the type being converted to is not a scalar or decimal
/// This shouldn't ever happen as long as this function is only used within this module
fn negate_column<'a, S: Scalar>(
    data_type: ColumnType,
    alloc: &'a Bump,
    input_column: &[S],
) -> Column<'a, S> {
    let negated_column_as_scalars =
        alloc.alloc_slice_fill_iter(input_column.iter().copied().map(S::neg));
    match data_type {
        ColumnType::Scalar => Column::Scalar(negated_column_as_scalars),
        ColumnType::Decimal75(precision, scale) => {
            Column::Decimal75(precision, scale, negated_column_as_scalars)
        }
        _ => panic!("Only scalars and decimals are returned by negation"),
    }
}

impl ProofExpr for NegExpr {
    fn data_type(&self) -> ColumnType {
        try_neg_type(self.input_expr.data_type()).expect("Datatype should already be confirmed")
    }

    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input_expr: Column<'a, S> =
            self.input_expr.first_round_evaluate(alloc, table, params)?;
        Ok(negate_column(
            self.data_type(),
            alloc,
            &input_expr.to_scalar(),
        ))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.add_expr.final_round_evaluate",
        level = "info",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        let input_expr: Column<'a, S> = self
            .input_expr
            .final_round_evaluate(builder, alloc, table, params)?;
        Ok(negate_column(
            self.data_type(),
            alloc,
            &input_expr.to_scalar(),
        ))
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let input_expr = self
            .input_expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        Ok(-input_expr)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.input_expr.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{Column, ColumnRef, ColumnType, TableRef},
            map::IndexSet,
            math::decimal::Precision,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof_exprs::{
            neg_expr::{negate_column, NegExpr},
            DynProofExpr, ProofExpr,
        },
    };
    use bumpalo::Bump;

    #[test]
    fn we_can_negate_column_from_int_input() {
        let alloc = Bump::new();
        let input_column = vec![-TestScalar::ONE, TestScalar::ONE];
        let neg_column = negate_column(
            ColumnType::Decimal75(Precision::new(10).unwrap(), 0),
            &alloc,
            &input_column,
        );
        assert_eq!(
            neg_column,
            Column::Decimal75(
                Precision::new(10).unwrap(),
                0,
                &[TestScalar::ONE, -TestScalar::ONE]
            )
        );
    }

    #[test]
    fn we_can_get_negated_column_from_scalar_input() {
        let alloc = Bump::new();
        let input_column = vec![-TestScalar::ONE, TestScalar::ONE];
        let neg_column = negate_column(ColumnType::Scalar, &alloc, &input_column);
        assert_eq!(
            neg_column,
            Column::Scalar(&[TestScalar::ONE, -TestScalar::ONE])
        );
    }

    #[test]
    #[should_panic(expected = "Only scalars and decimals are returned by negation")]
    fn we_cannot_get_negated_column_from_varchar_input() {
        let alloc = Bump::new();
        let input_column = vec![-TestScalar::ONE, TestScalar::ONE];
        negate_column(ColumnType::VarChar, &alloc, &input_column);
    }

    #[test]
    fn we_can_get_column_references() {
        let col = ColumnRef::new(
            TableRef::from_names(None, "test"),
            "test_column".into(),
            ColumnType::BigInt,
        );
        let input_expr = Box::new(DynProofExpr::new_column(col.clone()));
        let neg_expr = NegExpr::try_new(input_expr).unwrap();
        let mut columns = IndexSet::default();
        neg_expr.get_column_references(&mut columns);
        assert_eq!(columns.into_iter().collect::<Vec<_>>(), vec![col]);
    }
}
