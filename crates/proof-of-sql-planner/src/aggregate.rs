use super::{PlannerError, PlannerResult};
use crate::expr_to_proof_expr;
use datafusion::{
    common::DFSchema,
    logical_expr::expr::{AggregateFunction, AggregateFunctionDefinition},
    physical_plan,
};
use proof_of_sql::sql::proof_exprs::DynProofExpr;

/// An aggregate function we support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunc {
    /// Sum
    Sum,
    /// Count
    Count,
}

/// Convert an [`AggregateFunction`] to a [`DynProofExpr`]
///
/// TODO: Some moderate changes are necessary once we upgrade `DataFusion` to 46.0.0
pub(crate) fn aggregate_function_to_proof_expr(
    function: &AggregateFunction,
    schema: &DFSchema,
) -> PlannerResult<(AggregateFunc, DynProofExpr)> {
    match function {
        AggregateFunction {
            distinct: false,
            filter: None,
            order_by: None,
            args,
            func_def: AggregateFunctionDefinition::BuiltIn(op),
            ..
        } if args.len() == 1 => {
            let aggregate_function = match op {
                physical_plan::aggregates::AggregateFunction::Sum => AggregateFunc::Sum,
                physical_plan::aggregates::AggregateFunction::Count => AggregateFunc::Count,
                _ => Err(PlannerError::UnsupportedAggregateOperation { op: op.clone() })?,
            };
            Ok((aggregate_function, expr_to_proof_expr(&args[0], schema)?))
        }
        _ => Err(PlannerError::UnsupportedAggregateFunction {
            function: function.clone(),
        })?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use arrow::datatypes::DataType;
    use proof_of_sql::base::database::{ColumnRef, ColumnType, TableRef};

    // AggregateFunction to DynProofExpr
    #[test]
    fn we_can_convert_an_aggregate_function_to_proof_expr() {
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        for (function, operator) in &[
            (
                physical_plan::aggregates::AggregateFunction::Sum,
                AggregateFunc::Sum,
            ),
            (
                physical_plan::aggregates::AggregateFunction::Count,
                AggregateFunc::Count,
            ),
        ] {
            let function = AggregateFunction::new(
                function.clone(),
                vec![expr.clone()],
                false,
                None,
                None,
                None,
            );
            assert_eq!(
                aggregate_function_to_proof_expr(&function, &schema).unwrap(),
                (
                    *operator,
                    DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(None, "table"),
                        "a".into(),
                        ColumnType::BigInt
                    ))
                )
            );
        }
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported() {
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::RegrIntercept,
            vec![expr.clone()],
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateOperation { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_too_many_or_no_exprs() {
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        // Too many exprs
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Sum,
            vec![expr.clone(); 2],
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // No exprs
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Sum,
            Vec::<_>::new(),
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported_options() {
        // No distinct, filter, or order_by

        // Distinct
        let expr = df_column("table", "a");
        let schema = df_schema("table", vec![("a", DataType::Int64)]);
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            vec![expr.clone()],
            true,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // Filter
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            vec![expr.clone()],
            false,
            Some(Box::new(expr.clone())),
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // OrderBy
        let function = AggregateFunction::new(
            physical_plan::aggregates::AggregateFunction::Count,
            vec![expr.clone()],
            false,
            None,
            Some(vec![expr.clone()]),
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }
}
