use super::{
    column_to_column_ref_from_fields, placeholder_to_placeholder_expr,
    scalar_value_to_literal_value, PlannerError, PlannerResult,
};
use datafusion::logical_expr::{
    expr::{Alias, Cast, Placeholder},
    BinaryExpr, Expr, Operator,
};
use indexmap::IndexSet;
use proof_of_sql::{
    base::database::{ColumnField, ColumnRef, ColumnType},
    sql::{proof_exprs::DynProofExpr, scale_cast_binary_op},
};
use sqlparser::ast::Ident;

/// Recursively extract all column identifiers referenced in an expression
pub(crate) fn get_column_idents_from_expr(expr: &Expr) -> IndexSet<Ident> {
    match expr {
        Expr::Column(col) => {
            let mut set = IndexSet::new();
            set.insert(col.name.as_str().into());
            set
        }
        Expr::BinaryExpr(BinaryExpr { left, right, .. }) => {
            let mut left_idents = get_column_idents_from_expr(left);
            left_idents.extend(get_column_idents_from_expr(right));
            left_idents
        }
        Expr::IsNull(inner) | Expr::IsNotNull(inner) => match &**inner {
            Expr::Column(col) => {
                let mut set = IndexSet::new();
                set.insert(ColumnRef::presence_column_id(&col.name.as_str().into()));
                set
            }
            _ => get_column_idents_from_expr(inner),
        },
        Expr::Not(inner) => get_column_idents_from_expr(inner),
        Expr::Alias(Alias { expr, .. }) | Expr::Cast(Cast { expr, .. }) => {
            get_column_idents_from_expr(expr)
        }
        Expr::AggregateFunction(agg) => agg
            .args
            .iter()
            .flat_map(get_column_idents_from_expr)
            .collect(),
        _ => IndexSet::new(),
    }
}

fn is_filter_comparison_operator(op: Operator) -> bool {
    matches!(
        op,
        Operator::Eq
            | Operator::NotEq
            | Operator::Lt
            | Operator::Gt
            | Operator::LtEq
            | Operator::GtEq
    )
}

fn nullable_column_refs_in_expr(
    expr: &Expr,
    schema: &[ColumnField],
) -> PlannerResult<IndexSet<ColumnRef>> {
    match expr {
        Expr::Column(col) => {
            let column_ref = column_to_column_ref_from_fields(col, schema)?;
            Ok(if column_ref.is_nullable() {
                IndexSet::from_iter([column_ref])
            } else {
                IndexSet::new()
            })
        }
        Expr::BinaryExpr(BinaryExpr { left, right, .. }) => {
            let mut left_refs = nullable_column_refs_in_expr(left, schema)?;
            left_refs.extend(nullable_column_refs_in_expr(right, schema)?);
            Ok(left_refs)
        }
        Expr::Not(inner)
        | Expr::Alias(Alias { expr: inner, .. })
        | Expr::Cast(Cast { expr: inner, .. }) => nullable_column_refs_in_expr(inner, schema),
        Expr::IsNull(_) | Expr::IsNotNull(_) => Ok(IndexSet::new()),
        _ => Ok(IndexSet::new()),
    }
}

fn and_nullable_presence_guards(
    mut proof_expr: DynProofExpr,
    column_refs: IndexSet<ColumnRef>,
) -> PlannerResult<DynProofExpr> {
    for column_ref in column_refs {
        proof_expr =
            DynProofExpr::try_new_and(proof_expr, DynProofExpr::new_is_not_null(column_ref))?;
    }
    Ok(proof_expr)
}

/// Convert a [`BinaryExpr`] to [`DynProofExpr`]
#[expect(
    clippy::missing_panics_doc,
    reason = "Output of comparisons is always boolean"
)]
fn binary_expr_to_proof_expr(
    left: &Expr,
    right: &Expr,
    op: Operator,
    schema: &[ColumnField],
) -> PlannerResult<DynProofExpr> {
    let left_proof_expr = expr_to_proof_expr_with_fields(left, schema)?;
    let right_proof_expr = expr_to_proof_expr_with_fields(right, schema)?;

    let (left_proof_expr, right_proof_expr) = match op {
        Operator::Eq
        | Operator::NotEq
        | Operator::Lt
        | Operator::Gt
        | Operator::LtEq
        | Operator::GtEq
        | Operator::Plus
        | Operator::Minus => scale_cast_binary_op(left_proof_expr, right_proof_expr)?,
        _ => (left_proof_expr, right_proof_expr),
    };

    match op {
        Operator::And => Ok(DynProofExpr::try_new_and(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Or => Ok(DynProofExpr::try_new_or(left_proof_expr, right_proof_expr)?),
        Operator::Multiply => Ok(DynProofExpr::try_new_multiply(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Eq => Ok(DynProofExpr::try_new_equals(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::NotEq => Ok(DynProofExpr::try_new_not(DynProofExpr::try_new_equals(
            left_proof_expr,
            right_proof_expr,
        )?)
        .expect("An equality expression must have a boolean data type...")),
        Operator::Lt => Ok(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            true,
        )?),
        Operator::Gt => Ok(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            false,
        )?),
        Operator::LtEq => Ok(DynProofExpr::try_new_not(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            false,
        )?)
        .expect("An inequality expression must have a boolean data type...")),
        Operator::GtEq => Ok(DynProofExpr::try_new_not(DynProofExpr::try_new_inequality(
            left_proof_expr,
            right_proof_expr,
            true,
        )?)
        .expect("An inequality expression must have a boolean data type...")),
        Operator::Plus => Ok(DynProofExpr::try_new_add(
            left_proof_expr,
            right_proof_expr,
        )?),
        Operator::Minus => Ok(DynProofExpr::try_new_subtract(
            left_proof_expr,
            right_proof_expr,
        )?),
        // Any other operator is unsupported
        _ => Err(PlannerError::UnsupportedBinaryOperator { op }),
    }
}

/// Convert an [`datafusion::expr::Expr`] to [`DynProofExpr`]
///
/// # Panics
/// The function should not panic if Proof of SQL is working correctly
pub fn expr_to_proof_expr(
    expr: &Expr,
    schema: &[(Ident, ColumnType)],
) -> PlannerResult<DynProofExpr> {
    let column_fields = schema
        .iter()
        .map(|(ident, column_type)| ColumnField::new(ident.clone(), *column_type))
        .collect::<Vec<_>>();
    expr_to_proof_expr_with_fields(expr, &column_fields)
}

/// Convert a [`datafusion::expr::Expr`] to [`DynProofExpr`] while preserving column nullability.
///
/// # Panics
/// The function should not panic if Proof of SQL is working correctly
pub(crate) fn expr_to_proof_expr_with_fields(
    expr: &Expr,
    schema: &[ColumnField],
) -> PlannerResult<DynProofExpr> {
    match expr {
        Expr::Alias(Alias { expr, .. }) => expr_to_proof_expr_with_fields(expr, schema),
        Expr::Column(col) => Ok(DynProofExpr::new_column(column_to_column_ref_from_fields(
            col, schema,
        )?)),
        Expr::Placeholder(placeholder) => placeholder_to_placeholder_expr(placeholder),
        Expr::BinaryExpr(BinaryExpr { left, right, op }) => {
            binary_expr_to_proof_expr(left, right, *op, schema)
        }
        Expr::Literal(val) => Ok(DynProofExpr::new_literal(scalar_value_to_literal_value(
            val.clone(),
        )?)),
        Expr::Not(expr) => {
            let proof_expr = expr_to_proof_expr_with_fields(expr, schema)?;
            Ok(DynProofExpr::try_new_not(proof_expr)?)
        }
        Expr::IsNull(expr) => match &**expr {
            Expr::Column(col) => Ok(DynProofExpr::new_is_null(column_to_column_ref_from_fields(
                col, schema,
            )?)),
            _ => Err(PlannerError::UnsupportedLogicalExpression {
                expr: Box::new(expr.as_ref().clone()),
            }),
        },
        Expr::IsNotNull(expr) => match &**expr {
            Expr::Column(col) => Ok(DynProofExpr::new_is_not_null(
                column_to_column_ref_from_fields(col, schema)?,
            )),
            _ => Err(PlannerError::UnsupportedLogicalExpression {
                expr: Box::new(expr.as_ref().clone()),
            }),
        },
        Expr::Cast(cast) => {
            match &*cast.expr {
                // handle cases such as `$1::int`
                Expr::Placeholder(placeholder) if placeholder.data_type.is_none() => {
                    let typed_placeholder =
                        Placeholder::new(placeholder.id.clone(), Some(cast.data_type.clone()));
                    placeholder_to_placeholder_expr(&typed_placeholder)
                }
                _ => {
                    let from_expr = expr_to_proof_expr_with_fields(&cast.expr, schema)?;
                    let to_type = cast.data_type.clone().try_into().map_err(|_| {
                        PlannerError::UnsupportedDataType {
                            data_type: cast.data_type.clone(),
                        }
                    })?;
                    Ok(
                        DynProofExpr::try_new_cast(from_expr.clone(), to_type).map_or_else(
                            |_| DynProofExpr::try_new_scaling_cast(from_expr, to_type),
                            Ok,
                        )?,
                    )
                }
            }
        }
        _ => Err(PlannerError::UnsupportedLogicalExpression {
            expr: Box::new(expr.clone()),
        }),
    }
}

/// Convert a filter predicate to a [`DynProofExpr`] while applying SQL `IS TRUE`
/// semantics to direct nullable boolean columns.
pub(crate) fn filter_expr_to_proof_expr_with_fields(
    expr: &Expr,
    schema: &[ColumnField],
) -> PlannerResult<DynProofExpr> {
    match expr {
        Expr::Alias(Alias { expr, .. }) => filter_expr_to_proof_expr_with_fields(expr, schema),
        Expr::Column(col) => Ok(DynProofExpr::try_new_is_true(
            column_to_column_ref_from_fields(col, schema)?,
        )?),
        Expr::BinaryExpr(BinaryExpr { left, right, op }) if *op == Operator::And => {
            Ok(DynProofExpr::try_new_and(
                filter_expr_to_proof_expr_with_fields(left, schema)?,
                filter_expr_to_proof_expr_with_fields(right, schema)?,
            )?)
        }
        Expr::BinaryExpr(BinaryExpr { left, right, op }) if *op == Operator::Or => {
            Ok(DynProofExpr::try_new_or(
                filter_expr_to_proof_expr_with_fields(left, schema)?,
                filter_expr_to_proof_expr_with_fields(right, schema)?,
            )?)
        }
        Expr::BinaryExpr(BinaryExpr { op, .. }) if is_filter_comparison_operator(*op) => {
            and_nullable_presence_guards(
                expr_to_proof_expr_with_fields(expr, schema)?,
                nullable_column_refs_in_expr(expr, schema)?,
            )
        }
        Expr::Not(inner) => match &**inner {
            Expr::Column(col) => Ok(DynProofExpr::try_new_is_false(
                column_to_column_ref_from_fields(col, schema)?,
            )?),
            Expr::BinaryExpr(BinaryExpr { op, .. }) if is_filter_comparison_operator(*op) => {
                and_nullable_presence_guards(
                    DynProofExpr::try_new_not(expr_to_proof_expr_with_fields(inner, schema)?)?,
                    nullable_column_refs_in_expr(inner, schema)?,
                )
            }
            _ => expr_to_proof_expr_with_fields(expr, schema),
        },
        _ => expr_to_proof_expr_with_fields(expr, schema),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::df_util::*;
    use arrow::datatypes::DataType;
    use core::ops::{Add, Mul, Sub};
    use datafusion::{
        catalog::TableReference,
        common::{Column, ScalarValue},
        logical_expr::{expr::Placeholder, Cast},
    };
    use proof_of_sql::base::{
        database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
        math::decimal::Precision,
    };

    #[expect(non_snake_case)]
    fn COLUMN_INT() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column".into(),
            ColumnType::Int,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN1_SMALLINT() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::SmallInt,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN2_BIGINT() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column2".into(),
            ColumnType::BigInt,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN1_BOOLEAN() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column1".into(),
            ColumnType::Boolean,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN2_BOOLEAN() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column2".into(),
            ColumnType::Boolean,
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN3_DECIMAL_75_5() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column3".into(),
            ColumnType::Decimal75(
                Precision::new(75).expect("Precision is definitely valid"),
                5,
            ),
        ))
    }

    #[expect(non_snake_case)]
    fn COLUMN2_DECIMAL_25_5() -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column2".into(),
            ColumnType::Decimal75(
                Precision::new(25).expect("Precision is definitely valid"),
                5,
            ),
        ))
    }

    // Alias
    #[test]
    fn we_can_convert_alias_to_proof_expr() {
        // Column
        let expr = df_column("namespace.table_name", "column").alias("alias");
        let schema = vec![("column".into(), ColumnType::Int)];
        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), COLUMN_INT());
    }

    // Column
    #[test]
    fn we_can_convert_column_expr_to_proof_expr() {
        // Column
        let expr = df_column("namespace.table_name", "column");
        let schema = vec![("column".into(), ColumnType::Int)];
        assert_eq!(expr_to_proof_expr(&expr, &schema).unwrap(), COLUMN_INT());
    }

    #[test]
    fn we_can_convert_nullable_boolean_filter_column_to_is_true_proof_expr() {
        let expr = df_column("namespace.table_name", "is_paid");
        let schema = vec![ColumnField::new_nullable(
            "is_paid".into(),
            ColumnType::Boolean,
        )];
        let column_ref = ColumnRef::new_nullable(
            TableRef::from_names(Some("namespace"), "table_name"),
            "is_paid".into(),
            ColumnType::Boolean,
        );

        assert_eq!(
            filter_expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::try_new_is_true(column_ref).unwrap()
        );
    }

    #[test]
    fn we_can_convert_nullable_boolean_filter_not_column_to_is_false_proof_expr() {
        let expr = Expr::Not(Box::new(df_column("namespace.table_name", "is_paid")));
        let schema = vec![ColumnField::new_nullable(
            "is_paid".into(),
            ColumnType::Boolean,
        )];
        let column_ref = ColumnRef::new_nullable(
            TableRef::from_names(Some("namespace"), "table_name"),
            "is_paid".into(),
            ColumnType::Boolean,
        );

        assert_eq!(
            filter_expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::try_new_is_false(column_ref).unwrap()
        );
    }

    #[test]
    fn we_can_convert_nested_nullable_boolean_filter_columns_to_truth_proof_exprs() {
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "is_paid")),
            right: Box::new(
                df_column("namespace.table_name", "id")
                    .eq(Expr::Literal(ScalarValue::Int64(Some(4)))),
            ),
            op: Operator::Or,
        });
        let schema = vec![
            ColumnField::new_nullable("is_paid".into(), ColumnType::Boolean),
            ColumnField::new("id".into(), ColumnType::BigInt),
        ];
        let table_ref = TableRef::from_names(Some("namespace"), "table_name");
        let is_paid_ref =
            ColumnRef::new_nullable(table_ref.clone(), "is_paid".into(), ColumnType::Boolean);
        let id_ref = ColumnRef::new(table_ref, "id".into(), ColumnType::BigInt);

        assert_eq!(
            filter_expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::try_new_or(
                DynProofExpr::try_new_is_true(is_paid_ref).unwrap(),
                DynProofExpr::try_new_equals(
                    DynProofExpr::new_column(id_ref),
                    DynProofExpr::new_literal(LiteralValue::BigInt(4)),
                )
                .unwrap(),
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_convert_nullable_comparison_filter_to_presence_guarded_proof_expr() {
        let expr = df_column("namespace.table_name", "amount")
            .gt(Expr::Literal(ScalarValue::Int64(Some(15))));
        let schema = vec![ColumnField::new_nullable(
            "amount".into(),
            ColumnType::BigInt,
        )];
        let amount_ref = ColumnRef::new_nullable(
            TableRef::from_names(Some("namespace"), "table_name"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert_eq!(
            filter_expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::try_new_and(
                DynProofExpr::try_new_inequality(
                    DynProofExpr::new_column(amount_ref.clone()),
                    DynProofExpr::new_literal(LiteralValue::BigInt(15)),
                    false,
                )
                .unwrap(),
                DynProofExpr::new_is_not_null(amount_ref),
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_convert_negated_nullable_comparison_filter_to_presence_guarded_proof_expr() {
        let expr = Expr::Not(Box::new(
            df_column("namespace.table_name", "amount")
                .lt(Expr::Literal(ScalarValue::Int64(Some(15)))),
        ));
        let schema = vec![ColumnField::new_nullable(
            "amount".into(),
            ColumnType::BigInt,
        )];
        let amount_ref = ColumnRef::new_nullable(
            TableRef::from_names(Some("namespace"), "table_name"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert_eq!(
            filter_expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::try_new_and(
                DynProofExpr::try_new_not(
                    DynProofExpr::try_new_inequality(
                        DynProofExpr::new_column(amount_ref.clone()),
                        DynProofExpr::new_literal(LiteralValue::BigInt(15)),
                        true,
                    )
                    .unwrap(),
                )
                .unwrap(),
                DynProofExpr::new_is_not_null(amount_ref),
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_convert_nullable_column_is_null_expr_to_proof_expr() {
        let expr = Expr::IsNull(Box::new(df_column("namespace.table_name", "column")));
        let schema = vec![ColumnField::new_nullable("column".into(), ColumnType::Int)];
        let column_ref = ColumnRef::new_nullable(
            TableRef::from_names(Some("namespace"), "table_name"),
            "column".into(),
            ColumnType::Int,
        );

        assert_eq!(
            expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::new_is_null(column_ref)
        );
    }

    #[test]
    fn we_can_convert_non_nullable_column_is_not_null_expr_to_constant_true() {
        let expr = Expr::IsNotNull(Box::new(df_column("namespace.table_name", "column")));
        let schema = vec![ColumnField::new("column".into(), ColumnType::Int)];

        assert_eq!(
            expr_to_proof_expr_with_fields(&expr, &schema).unwrap(),
            DynProofExpr::new_literal(LiteralValue::Boolean(true))
        );
    }

    // BinaryExpr
    #[test]
    fn we_can_convert_comparison_binary_expr_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            ("column2".into(), ColumnType::BigInt),
        ];

        // Eq
        let expr = df_column("namespace.table_name", "column1")
            .eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_equals(COLUMN1_SMALLINT(), COLUMN2_BIGINT()).unwrap()
        );

        // Lt
        let expr = df_column("namespace.table_name", "column1")
            .lt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), true).unwrap()
        );

        // Gt
        let expr = df_column("namespace.table_name", "column1")
            .gt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), false).unwrap()
        );

        // LtEq
        let expr = df_column("namespace.table_name", "column1")
            .lt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), false)
                    .unwrap()
            )
            .unwrap()
        );

        // GtEq
        let expr = df_column("namespace.table_name", "column1")
            .gt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(COLUMN1_SMALLINT(), COLUMN2_BIGINT(), true)
                    .unwrap()
            )
            .unwrap()
        );
    }

    #[expect(clippy::too_many_lines)]
    #[test]
    fn we_can_convert_comparison_binary_expr_to_proof_expr_with_scale_cast() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            (
                "column2".into(),
                ColumnType::Decimal75(Precision::new(25).unwrap(), 5),
            ),
            (
                "column3".into(),
                ColumnType::Decimal75(Precision::new(75).unwrap(), 5),
            ),
        ];

        // Eq
        let expr = df_column("namespace.table_name", "column1")
            .eq(df_column("namespace.table_name", "column3"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_equals(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN3_DECIMAL_75_5()
            )
            .unwrap()
        );

        // Lt
        let expr = df_column("namespace.table_name", "column1")
            .lt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5(),
                true
            )
            .unwrap()
        );

        // Gt
        let expr = df_column("namespace.table_name", "column1")
            .gt(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_inequality(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5(),
                false
            )
            .unwrap()
        );

        // LtEq
        let expr = df_column("namespace.table_name", "column1")
            .lt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(
                    DynProofExpr::try_new_scaling_cast(
                        COLUMN1_SMALLINT(),
                        ColumnType::Decimal75(
                            Precision::new(10).expect("Precision is definitely valid"),
                            5
                        )
                    )
                    .unwrap(),
                    COLUMN2_DECIMAL_25_5(),
                    false
                )
                .unwrap()
            )
            .unwrap()
        );

        // GtEq
        let expr = df_column("namespace.table_name", "column1")
            .gt_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_inequality(
                    DynProofExpr::try_new_scaling_cast(
                        COLUMN1_SMALLINT(),
                        ColumnType::Decimal75(
                            Precision::new(10).expect("Precision is definitely valid"),
                            5
                        )
                    )
                    .unwrap(),
                    COLUMN2_DECIMAL_25_5(),
                    true
                )
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn we_can_convert_arithmetic_binary_expr_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            ("column2".into(), ColumnType::BigInt),
        ];

        // Plus
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::Plus,
        });
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_add(COLUMN1_SMALLINT(), COLUMN2_BIGINT(),).unwrap()
        );

        // Minus
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::Minus,
        });
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_subtract(COLUMN1_SMALLINT(), COLUMN2_BIGINT(),).unwrap()
        );

        // Multiply
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::Multiply,
        });
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_multiply(COLUMN1_SMALLINT(), COLUMN2_BIGINT(),).unwrap()
        );
    }

    #[test]
    fn we_can_convert_arithmetic_binary_expr_to_proof_expr_with_scale_cast() {
        let schema = vec![
            ("column1".into(), ColumnType::SmallInt),
            (
                "column2".into(),
                ColumnType::Decimal75(Precision::new(25).unwrap(), 5),
            ),
            (
                "column3".into(),
                ColumnType::Decimal75(Precision::new(75).unwrap(), 5),
            ),
        ];

        // Add
        let expr = df_column("namespace.table_name", "column1")
            .add(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_add(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5()
            )
            .unwrap()
        );

        // Subtract
        let expr = df_column("namespace.table_name", "column1")
            .sub(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_subtract(
                DynProofExpr::try_new_scaling_cast(
                    COLUMN1_SMALLINT(),
                    ColumnType::Decimal75(
                        Precision::new(10).expect("Precision is definitely valid"),
                        5
                    )
                )
                .unwrap(),
                COLUMN2_DECIMAL_25_5()
            )
            .unwrap()
        );

        // Multiply - No scale cast!
        let expr = df_column("namespace.table_name", "column1")
            .mul(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_multiply(COLUMN1_SMALLINT(), COLUMN2_DECIMAL_25_5()).unwrap()
        );
    }

    #[test]
    fn we_can_convert_logical_binary_expr_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::Boolean),
            ("column2".into(), ColumnType::Boolean),
        ];

        // And
        let expr = df_column("namespace.table_name", "column1")
            .and(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_and(COLUMN1_BOOLEAN(), COLUMN2_BOOLEAN()).unwrap()
        );

        // Or
        let expr = df_column("namespace.table_name", "column1")
            .or(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_or(COLUMN1_BOOLEAN(), COLUMN2_BOOLEAN()).unwrap()
        );
    }

    #[test]
    fn we_can_convert_logical_not_eq_to_proof_expr() {
        let schema = vec![
            ("column1".into(), ColumnType::BigInt),
            ("column2".into(), ColumnType::BigInt),
        ];

        let expr = df_column("namespace.table_name", "column1")
            .not_eq(df_column("namespace.table_name", "column2"));
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(
                DynProofExpr::try_new_equals(
                    DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(Some("namespace"), "table_name"),
                        "column1".into(),
                        ColumnType::BigInt,
                    )),
                    DynProofExpr::new_column(ColumnRef::new(
                        TableRef::from_names(Some("namespace"), "table_name"),
                        "column2".into(),
                        ColumnType::BigInt,
                    ))
                )
                .unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn we_cannot_convert_unsupported_binary_expr_to_proof_expr() {
        // Unsupported binary operator
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(df_column("namespace.table_name", "column1")),
            right: Box::new(df_column("namespace.table_name", "column2")),
            op: Operator::AtArrow,
        });
        let schema = vec![
            ("column1".into(), ColumnType::Boolean),
            ("column2".into(), ColumnType::Boolean),
        ];
        assert!(matches!(
            expr_to_proof_expr(&expr, &schema),
            Err(PlannerError::UnsupportedBinaryOperator { .. })
        ));
    }

    // Literal
    #[test]
    fn we_can_convert_literal_expr_to_proof_expr() {
        let expr = Expr::Literal(ScalarValue::Int32(Some(1)));
        assert_eq!(
            expr_to_proof_expr(&expr, &Vec::new()).unwrap(),
            DynProofExpr::new_literal(LiteralValue::Int(1))
        );
    }

    // Not
    #[test]
    fn we_can_convert_not_expr_to_proof_expr() {
        let expr = Expr::Not(Box::new(df_column("table_name", "column")));
        let schema = vec![("column".into(), ColumnType::Boolean)];
        assert_eq!(
            expr_to_proof_expr(&expr, &schema).unwrap(),
            DynProofExpr::try_new_not(DynProofExpr::new_column(ColumnRef::new(
                TableRef::from_names(None, "table_name"),
                "column".into(),
                ColumnType::Boolean
            )))
            .unwrap()
        );
    }

    // Cast
    #[test]
    fn we_can_convert_cast_expr_to_proof_expr() {
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::Boolean(Some(true)))),
            DataType::Int32,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_cast(
                DynProofExpr::new_literal(LiteralValue::Boolean(true)),
                ColumnType::Int
            )
            .unwrap()
        );
    }

    #[test]
    fn we_cannot_convert_cast_expr_to_proof_expr_when_inner_expr_to_proof_expr_fails() {
        // Unsupported logical expression
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::UInt64(Some(100)))),
            DataType::Int16,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap_err();
        assert!(matches!(
            expression,
            PlannerError::UnsupportedDataType { data_type: _ }
        ));
    }

    #[test]
    fn we_cannot_convert_cast_expr_to_proof_expr_for_unsupported_datatypes() {
        // Unsupported logical expression
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::Boolean(Some(true)))),
            DataType::UInt16,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap_err();
        assert!(matches!(
            expression,
            PlannerError::UnsupportedDataType { data_type: _ }
        ));
    }

    #[test]
    fn we_cannot_convert_cast_expr_to_proof_expr_for_datatypes_for_which_casting_is_not_supported()
    {
        // Unsupported logical expression
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Literal(ScalarValue::Int16(Some(100)))),
            DataType::Boolean,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap_err();
        assert!(matches!(
            expression,
            PlannerError::AnalyzeError { source: _ }
        ));
    }

    // Placeholder
    #[test]
    fn we_can_convert_placeholder_to_proof_expr() {
        let expr = Expr::Placeholder(Placeholder {
            id: "$1".to_string(),
            data_type: Some(DataType::Int32),
        });
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap()
        );
    }

    // Placeholder with data type specified by cast
    #[test]
    fn we_can_convert_placeholder_with_data_type_specified_by_cast_to_proof_expr() {
        let expr = Expr::Cast(Cast::new(
            Box::new(Expr::Placeholder(Placeholder {
                id: "$1".to_string(),
                data_type: None,
            })),
            DataType::Int32,
        ));
        let expression = expr_to_proof_expr(&expr, &Vec::new()).unwrap();
        assert_eq!(
            expression,
            DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap()
        );
    }

    // Unsupported logical expression
    #[test]
    fn we_cannot_convert_unsupported_expr_to_proof_expr() {
        let expr = Expr::OuterReferenceColumn(
            DataType::Int32,
            Column::new(None::<TableReference>, "column"),
        );
        assert!(matches!(
            expr_to_proof_expr(&expr, &Vec::new()),
            Err(PlannerError::UnsupportedLogicalExpression { .. })
        ));
    }

    #[test]
    fn we_can_get_proof_expr_for_timestamps_of_different_scale() {
        let lhs = Expr::Literal(ScalarValue::TimestampSecond(Some(1), None));
        let rhs = Expr::Literal(ScalarValue::TimestampNanosecond(Some(1), None));
        binary_expr_to_proof_expr(&lhs, &rhs, Operator::Gt, &Vec::new()).unwrap();
    }

    // get_column_idents_from_expr tests
    #[test]
    fn we_can_extract_single_column_ident() {
        let expr = df_column("table", "column_a");
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["column_a".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_binary_expr() {
        let expr = df_column("table", "a").add(df_column("table", "b"));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_nested_binary_expr() {
        // (a + b) * c
        let expr = df_column("table", "a")
            .add(df_column("table", "b"))
            .mul(df_column("table", "c"));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_not_expr() {
        let expr = Expr::Not(Box::new(df_column("table", "bool_col")));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["bool_col".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_is_null_expr() {
        let expr = Expr::IsNull(Box::new(df_column("table", "nullable_col")));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = [ColumnRef::presence_column_id(&"nullable_col".into())]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_alias_expr() {
        let expr = df_column("table", "col_x").alias("alias_name");
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["col_x".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_cast_expr() {
        let expr = Expr::Cast(Cast::new(
            Box::new(df_column("table", "num_col")),
            DataType::Int64,
        ));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["num_col".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_aggregate_function() {
        let expr = Expr::AggregateFunction(datafusion::logical_expr::expr::AggregateFunction {
            func_def: datafusion::logical_expr::expr::AggregateFunctionDefinition::BuiltIn(
                datafusion::physical_plan::aggregates::AggregateFunction::Sum,
            ),
            args: vec![df_column("table", "value")],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["value".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_from_aggregate_function_with_multiple_args() {
        let expr = Expr::AggregateFunction(datafusion::logical_expr::expr::AggregateFunction {
            func_def: datafusion::logical_expr::expr::AggregateFunctionDefinition::BuiltIn(
                datafusion::physical_plan::aggregates::AggregateFunction::Sum,
            ),
            args: vec![
                df_column("table", "col1"),
                df_column("table", "col2"),
                df_column("table", "col3"),
            ],
            distinct: false,
            filter: None,
            order_by: None,
            null_treatment: None,
        });
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["col1".into(), "col2".into(), "col3".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_no_column_idents_from_literal() {
        let expr = Expr::Literal(ScalarValue::Int32(Some(42)));
        let result = get_column_idents_from_expr(&expr);
        assert!(result.is_empty());
    }

    #[test]
    fn we_can_extract_column_idents_from_complex_nested_expr() {
        // NOT (a > b AND c < d)
        let inner = df_column("table", "a")
            .gt(df_column("table", "b"))
            .and(df_column("table", "c").lt(df_column("table", "d")));
        let expr = Expr::Not(Box::new(inner));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into(), "b".into(), "c".into(), "d".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_column_idents_preserving_order() {
        // IndexSet should preserve insertion order
        let expr = df_column("table", "z")
            .add(df_column("table", "a"))
            .add(df_column("table", "m"));
        let result = get_column_idents_from_expr(&expr);
        let idents: Vec<Ident> = result.into_iter().collect();
        assert_eq!(idents, vec!["z".into(), "a".into(), "m".into()]);
    }

    #[test]
    fn we_can_handle_duplicate_column_references() {
        // a + a should only have 'a' once
        let expr = df_column("table", "a").add(df_column("table", "a"));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["a".into()].into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn we_can_extract_columns_from_comparison_operations() {
        let expr = df_column("table", "price")
            .gt(df_column("table", "threshold"))
            .and(df_column("table", "active").eq(Expr::Literal(ScalarValue::Boolean(Some(true)))));
        let result = get_column_idents_from_expr(&expr);
        let expected: IndexSet<Ident> = ["price".into(), "threshold".into(), "active".into()]
            .into_iter()
            .collect();
        assert_eq!(result, expected);
    }
}
