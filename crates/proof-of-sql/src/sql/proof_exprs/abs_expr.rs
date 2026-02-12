use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_neg_type, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_gadgets::{
            final_round_evaluate_sign, first_round_evaluate_sign, verifier_evaluate_sign,
        },
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable numerical `||` expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbsExpr {
    input_expr: Box<DynProofExpr>,
}

impl AbsExpr {
    /// Create numerical `||` expression
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
fn get_absolute_value_column<'a, S: Scalar>(
    data_type: ColumnType,
    alloc: &'a Bump,
    is_negative_column: &[bool],
    input_column: &[S],
) -> Column<'a, S> {
    let abs_value_column_as_scalars = alloc.alloc_slice_fill_iter(
        is_negative_column
            .iter()
            .zip(input_column.iter().copied())
            .map(|(is_negative, input_value)| {
                if *is_negative {
                    -input_value
                } else {
                    input_value
                }
            }),
    );
    match data_type {
        ColumnType::Scalar => Column::Scalar(abs_value_column_as_scalars),
        ColumnType::Decimal75(precision, scale) => {
            Column::Decimal75(precision, scale, abs_value_column_as_scalars)
        }
        _ => panic!("Only scalars and decimals are returned by absolute value"),
    }
}

impl ProofExpr for AbsExpr {
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
        let input_column = input_expr.to_scalar();
        let is_negative_column = first_round_evaluate_sign(table.num_rows(), alloc, &input_column);
        Ok(get_absolute_value_column(
            self.data_type(),
            alloc,
            is_negative_column,
            &input_column,
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
        let input_column = input_expr.to_scalar();
        let input_column_as_slice = alloc.alloc_slice_fill_iter(input_column.iter().copied());
        let is_negative_column = final_round_evaluate_sign(builder, alloc, input_column_as_slice);
        let abs_value_column =
            get_absolute_value_column(self.data_type(), alloc, is_negative_column, &input_column);
        builder.produce_intermediate_mle(abs_value_column);

        // |expr| + (2*is_neg - 1) * expr = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(abs_value_column)]),
                (
                    S::TWO,
                    vec![Box::new(is_negative_column), Box::new(input_expr)],
                ),
                (-S::one(), vec![Box::new(input_expr)]),
            ],
        );

        Ok(abs_value_column)
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
        let is_negative_column = verifier_evaluate_sign(builder, input_expr, chi_eval, None)?;
        let abs_expr = builder.try_consume_final_round_mle_evaluation()?;

        // |expr| + (2*is_neg - 1) * expr = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            abs_expr + (S::TWO * is_negative_column - S::ONE) * input_expr,
            2,
        )?;
        Ok(abs_expr)
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
            abs_expr::{get_absolute_value_column, AbsExpr},
            DynProofExpr, ProofExpr,
        },
    };
    use bumpalo::Bump;

    #[test]
    fn we_can_get_absolute_value_column_from_int_input() {
        let alloc = Bump::new();
        let is_negative_column = [true, false];
        let input_column = vec![-TestScalar::ONE, TestScalar::ONE];
        let abs_column = get_absolute_value_column(
            ColumnType::Decimal75(Precision::new(10).unwrap(), 0),
            &alloc,
            &is_negative_column,
            &input_column,
        );
        assert_eq!(
            abs_column,
            Column::Decimal75(
                Precision::new(10).unwrap(),
                0,
                &[TestScalar::ONE, TestScalar::ONE]
            )
        );
    }

    #[test]
    fn we_can_get_absolute_value_column_from_scalar_input() {
        let alloc = Bump::new();
        let is_negative_column = [true, false];
        let input_column = vec![-TestScalar::ONE, TestScalar::ONE];
        let abs_column = get_absolute_value_column(
            ColumnType::Scalar,
            &alloc,
            &is_negative_column,
            &input_column,
        );
        assert_eq!(
            abs_column,
            Column::Scalar(&[TestScalar::ONE, TestScalar::ONE])
        );
    }

    #[test]
    #[should_panic(expected = "Only scalars and decimals are returned by absolute value")]
    fn we_cannot_get_absolute_value_column_from_varchar_input() {
        let alloc = Bump::new();
        let is_negative_column = [true, false];
        let input_column = vec![-TestScalar::ONE, TestScalar::ONE];
        get_absolute_value_column(
            ColumnType::VarChar,
            &alloc,
            &is_negative_column,
            &input_column,
        );
    }

    #[test]
    fn we_can_get_column_references() {
        let col = ColumnRef::new(
            TableRef::from_names(None, "test"),
            "test_column".into(),
            ColumnType::BigInt,
        );
        let input_expr = Box::new(DynProofExpr::new_column(col.clone()));
        let abs_expr = AbsExpr::try_new(input_expr).unwrap();
        let mut columns = IndexSet::default();
        abs_expr.get_column_references(&mut columns);
        assert_eq!(columns.into_iter().collect::<Vec<_>>(), vec![col]);
    }
}
