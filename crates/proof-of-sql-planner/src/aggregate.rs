use super::{PlannerError, PlannerResult};
use crate::expr_to_proof_expr;
use datafusion::logical_expr::{expr::AggregateFunction, Expr};
use proof_of_sql::{
    base::database::{ColumnType, LiteralValue},
    sql::proof_exprs::DynProofExpr,
};
use sqlparser::ast::Ident;

/// An aggregate function we support
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggregateFunc {
    /// Sum
    Sum,
    /// Count
    Count,
}

/// Convert an [`AggregateFunction`] to a [`DynProofExpr`]
pub(crate) fn aggregate_function_to_proof_expr(
    function: &AggregateFunction,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<(AggregateFunc, DynProofExpr)> {
    match function {
        AggregateFunction {
            distinct: false,
            filter: None,
            order_by: None,
            args,
            func,
            ..
        } if args.len() == 1 => {
            let arg = &args[0];
            let func_name = func.name();
            match arg {
                &Expr::Wildcard { .. } if func_name.eq_ignore_ascii_case("count") => {
                    // Special case for COUNT(*)
                    let proof_expr = DynProofExpr::new_literal(LiteralValue::BigInt(1));
                    Ok((AggregateFunc::Count, proof_expr))
                }
                _ if func_name.eq_ignore_ascii_case("sum") => {
                    Ok((AggregateFunc::Sum, expr_to_proof_expr(arg, schema)?))
                }
                _ if func_name.eq_ignore_ascii_case("count") => {
                    Ok((AggregateFunc::Count, expr_to_proof_expr(arg, schema)?))
                }
                _ => Err(PlannerError::UnsupportedAggregateFunctionName {
                    name: func_name.to_string(),
                }),
            }
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
    use datafusion::functions_aggregate::{count::count_udaf, sum::sum_udaf};
    use proof_of_sql::base::database::{ColumnRef, ColumnType, TableRef};

    // AggregateFunction to DynProofExpr
    #[test]
    fn we_can_convert_an_aggregate_function_to_proof_expr() {
        let expr = df_column("table", "a");
        let schema: Vec<(Ident, ColumnType)> = vec![("a".into(), ColumnType::BigInt)];
        for (func_udaf, operator) in &[
            (sum_udaf(), AggregateFunc::Sum),
            (count_udaf(), AggregateFunc::Count),
        ] {
            let function = AggregateFunction::new_udf(
                func_udaf.clone(),
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
    fn we_can_convert_count_star_to_count_one() {
        use datafusion::logical_expr::expr::WildcardOptions;
        use proof_of_sql::base::database::LiteralValue;

        let wildcard_expr = Expr::Wildcard {
            qualifier: None,
            options: WildcardOptions::default(),
        };
        let schema: Vec<(Ident, ColumnType)> = vec![("a".into(), ColumnType::BigInt)];
        let function =
            AggregateFunction::new_udf(count_udaf(), vec![wildcard_expr], false, None, None, None);
        assert_eq!(
            aggregate_function_to_proof_expr(&function, &schema).unwrap(),
            (
                AggregateFunc::Count,
                DynProofExpr::new_literal(LiteralValue::BigInt(1))
            )
        );
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported() {
        use datafusion::functions_aggregate::regr::regr_intercept_udaf;

        let expr = df_column("table", "a");
        let schema = vec![("a".into(), ColumnType::BigInt)];
        let function = AggregateFunction::new_udf(
            regr_intercept_udaf(),
            vec![expr.clone()],
            false,
            None,
            None,
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunctionName { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_too_many_or_no_exprs() {
        let expr = df_column("table", "a");
        let schema = vec![("a".into(), ColumnType::BigInt)];
        // Too many exprs
        let function =
            AggregateFunction::new_udf(sum_udaf(), vec![expr.clone(); 2], false, None, None, None);
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // No exprs
        let function =
            AggregateFunction::new_udf(sum_udaf(), Vec::<Expr>::new(), false, None, None, None);
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_an_aggregate_function_to_pair_if_unsupported_options() {
        use datafusion::logical_expr::expr::Sort;
        // No distinct, filter, or order_by

        // Distinct
        let expr = df_column("table", "a");
        let schema = vec![("a".into(), ColumnType::BigInt)];
        let function =
            AggregateFunction::new_udf(count_udaf(), vec![expr.clone()], true, None, None, None);
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));

        // Filter
        let function = AggregateFunction::new_udf(
            count_udaf(),
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

        // OrderBy - using Sort objects
        let sort = Sort {
            expr: expr.clone(),
            asc: true,
            nulls_first: true,
        };
        let function = AggregateFunction::new_udf(
            count_udaf(),
            vec![expr.clone()],
            false,
            None,
            Some(vec![sort]),
            None,
        );
        assert!(matches!(
            aggregate_function_to_proof_expr(&function, &schema),
            Err(PlannerError::UnsupportedAggregateFunction { .. })
        ));
    }
}
