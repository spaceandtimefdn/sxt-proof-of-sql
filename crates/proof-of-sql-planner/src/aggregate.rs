use super::{PlannerError, PlannerResult};
use crate::expr_to_proof_expr;
use datafusion::{
    functions_aggregate::{count::count_udaf, sum::sum_udaf},
    logical_expr::{
        expr::{AggregateFunction, AggregateFunctionParams},
        AggregateUDF,
    },
};
use proof_of_sql::{base::database::ColumnType, sql::proof_exprs::DynProofExpr};
use sqlparser::ast::Ident;

/// An aggregate function we support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunc {
    /// Sum
    Sum,
    /// Count
    Count,
}

/// Convert a `DataFusion` [`AggregateUDF`] to an [`AggregateFunc`]
pub(crate) fn aggregate_udf_to_aggregate_func(udf: &AggregateUDF) -> PlannerResult<AggregateFunc> {
    if *udf == *sum_udaf() {
        Ok(AggregateFunc::Sum)
    } else if *udf == *count_udaf() {
        Ok(AggregateFunc::Count)
    } else {
        Err(PlannerError::UnsupportedAggregateUDF { udf: udf.clone() })?
    }
}

/// Convert an [`AggregateFunction`] to a [`DynProofExpr`]
///
/// TODO: Some moderate changes are necessary once we upgrade `DataFusion` to 46.0.0
pub(crate) fn aggregate_function_to_proof_expr(
    function: &AggregateFunction,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<(AggregateFunc, DynProofExpr)> {
    let agg_func = aggregate_udf_to_aggregate_func(&function.func)?;
    match &function.params {
        AggregateFunctionParams {
            distinct: false,
            filter: None,
            order_by,
            args,
            ..
        } if args.len() == 1 && order_by.is_empty() => {
            Ok((agg_func, expr_to_proof_expr(&args[0], schema)?))
        }
        _ => Err(PlannerError::UnsupportedAggregateFunctionParams {
            params: function.params.clone(),
        })?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use datafusion::logical_expr::expr::AggregateFunctionParams;
    use proof_of_sql::base::database::{ColumnRef, ColumnType, TableRef};

    // AggregateFunction to DynProofExpr
    #[test]
    fn we_can_convert_an_aggregate_function_to_proof_expr() {
        let expr = df_column("table", "a");
        let schema: Vec<(Ident, ColumnType)> = vec![("a".into(), ColumnType::BigInt)];
        for (udf, operator) in &[
            (sum_udaf(), AggregateFunc::Sum),
            (count_udaf(), AggregateFunc::Count),
        ] {
            let function = AggregateFunction {
                func: udf.clone(),
                params: AggregateFunctionParams {
                    args: vec![expr.clone()],
                    distinct: false,
                    filter: None,
                    order_by: vec![],
                    null_treatment: None,
                },
            };
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
        use datafusion::functions_aggregate::min_max::min_udaf;
        let expr = df_column("table", "a");
        let schema = vec![("a".into(), ColumnType::BigInt)];
        let function = AggregateFunction {
            func: min_udaf(),
            params: AggregateFunctionParams {
                args: vec![expr.clone()],
                distinct: false,
                filter: None,
                order_by: vec![],
                null_treatment: None,
            },
        };
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateUDF { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_too_many_or_no_exprs() {
        let expr = df_column("table", "a");
        let schema = vec![("a".into(), ColumnType::BigInt)];
        // Too many exprs
        let function = AggregateFunction {
            func: sum_udaf(),
            params: AggregateFunctionParams {
                args: vec![expr.clone(); 2],
                distinct: false,
                filter: None,
                order_by: vec![],
                null_treatment: None,
            },
        };
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunctionParams { .. })
        ));

        // No exprs
        let function = AggregateFunction {
            func: sum_udaf(),
            params: AggregateFunctionParams {
                args: Vec::<_>::new(),
                distinct: false,
                filter: None,
                order_by: vec![],
                null_treatment: None,
            },
        };
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunctionParams { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported_options() {
        // No distinct, filter, or order_by

        // Distinct
        let expr = df_column("table", "a");
        let schema = vec![("a".into(), ColumnType::BigInt)];
        let function = AggregateFunction {
            func: count_udaf(),
            params: AggregateFunctionParams {
                args: vec![expr.clone()],
                distinct: true,
                filter: None,
                order_by: vec![],
                null_treatment: None,
            },
        };
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunctionParams { .. })
        ));

        // Filter
        let function = AggregateFunction {
            func: count_udaf(),
            params: AggregateFunctionParams {
                args: vec![expr.clone()],
                distinct: false,
                filter: Some(Box::new(expr.clone())),
                order_by: vec![],
                null_treatment: None,
            },
        };
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunctionParams { .. })
        ));

        // OrderBy
        let function = AggregateFunction {
            func: count_udaf(),
            params: AggregateFunctionParams {
                args: vec![expr.clone()],
                distinct: false,
                filter: None,
                order_by: vec![datafusion::logical_expr::SortExpr::new(
                    expr.clone(),
                    true,
                    true,
                )],
                null_treatment: None,
            },
        };
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunctionParams { .. })
        ));
    }
}
