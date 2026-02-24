use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{
            plans::{try_unwrap_output_column_names_with_count_alias, EVMDynProofPlan},
            EVMDynProofExpr, EVMProofPlanError, EVMProofPlanResult,
        },
        proof_exprs::AliasedDynProofExpr,
        proof_plans::AggregateExec,
    },
};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents an aggregate execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMAggregateExec {
    input_plan: Box<EVMDynProofPlan>,
    group_by_exprs: Vec<EVMDynProofExpr>,
    where_clause: EVMDynProofExpr,
    sum_expr: Vec<EVMDynProofExpr>,
    count_alias_name: String,
}

impl EVMAggregateExec {
    /// Try to create a `EVMAggregateExec` from an `AggregateExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &AggregateExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // Get the input result columns to use for expression conversion
        let input_result_column_refs = plan.input().get_column_result_fields_as_references();

        let group_by_exprs = plan
            .group_by_exprs()
            .iter()
            .map(|aliased_expr| {
                EVMDynProofExpr::try_from_proof_expr(&aliased_expr.expr, &input_result_column_refs)
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            group_by_exprs,
            sum_expr: plan
                .sum_expr()
                .iter()
                .map(|aliased_expr| {
                    EVMDynProofExpr::try_from_proof_expr(
                        &aliased_expr.expr,
                        &input_result_column_refs,
                    )
                })
                .collect::<Result<_, _>>()?,
            count_alias_name: plan.count_alias().value.clone(),
            where_clause: EVMDynProofExpr::try_from_proof_expr(
                plan.where_clause(),
                &input_result_column_refs,
            )?,
        })
    }

    #[expect(
        clippy::missing_panics_doc,
        reason = "There is a check before unwrapping"
    )]
    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<AggregateExec> {
        let required_alias_count = self.group_by_exprs.len() + self.sum_expr.len() + 1;
        let output_column_names = try_unwrap_output_column_names_with_count_alias(
            output_column_names,
            required_alias_count,
            &self.count_alias_name,
        )?;
        let input = self
            .input_plan
            .try_into_proof_plan(table_refs, column_refs, None)?;
        let input_result_column_refs = input.get_column_result_fields_as_references();

        let mut output_column_names = output_column_names.iter();
        // Map group by expressions to AliasedDynProofExpr objects
        let group_by_exprs = self
            .group_by_exprs
            .iter()
            .zip(&mut output_column_names)
            .map(|(expr, alias_name)| {
                Ok(AliasedDynProofExpr {
                    expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                    alias: Ident::new(alias_name),
                })
            })
            .collect::<EVMProofPlanResult<Vec<_>>>()?;

        // Map sum expressions to AliasedDynProofExpr objects
        let sum_expr = self
            .sum_expr
            .iter()
            .zip(&mut output_column_names)
            .map(|(expr, alias_name)| {
                Ok(AliasedDynProofExpr {
                    expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                    alias: Ident::new(alias_name),
                })
            })
            .collect::<EVMProofPlanResult<Vec<_>>>()?;

        // For safety, check if the provided count_alias_name matches
        if &self.count_alias_name
            != output_column_names
                .next()
                .expect("Value confirmed to exist")
        {
            Err(EVMProofPlanError::InvalidOutputColumnName)?;
        }

        AggregateExec::try_new(
            group_by_exprs,
            sum_expr,
            Ident::new(&self.count_alias_name),
            Box::new(input),
            self.where_clause
                .try_into_proof_expr(&input_result_column_refs)?,
        )
        .ok_or(EVMProofPlanError::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnField, ColumnRef, ColumnType, LiteralValue, TableRef},
            map::indexset,
        },
        sql::{
            evm_proof_plan::{
                plans::{EVMAggregateExec, EVMDynProofPlan},
                EVMProofPlanError,
            },
            proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, LiteralExpr},
            proof_plans::{AggregateExec, DynProofPlan, FilterExec, TableExec},
        },
    };
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_put_simple_aggregate_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec as input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create output column refs for aggregate (these come from the table's output)
        let output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create an aggregate exec
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_a.clone())),
                alias: ident_a.clone(),
            }],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        // Convert to EVM plan
        let evm_aggregate_exec = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_aggregate_exec.group_by_exprs.len(), 1);
        assert_eq!(evm_aggregate_exec.sum_expr.len(), 1);
        assert_eq!(evm_aggregate_exec.count_alias_name, count_alias);
        assert!(matches!(
            *evm_aggregate_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_aggregate_exec = EVMAggregateExec::try_into_proof_plan(
            &evm_aggregate_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
            Some(&indexset![ident_a.value, sum_alias, count_alias]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_aggregate_exec.group_by_exprs().len(), 1);
        assert_eq!(roundtripped_aggregate_exec.sum_expr().len(), 1);
        assert!(matches!(
            roundtripped_aggregate_exec.group_by_exprs()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert!(matches!(
            *roundtripped_aggregate_exec.input(),
            DynProofPlan::Table(_)
        ));
    }

    #[test]
    fn we_can_put_complex_aggregate_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();
        let sum_alias_b = "sum_b".to_string();
        let sum_alias_c = "sum_c".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let column_ref_c = ColumnRef::new(table_ref.clone(), ident_c.clone(), ColumnType::BigInt);

        // Create a table exec
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
            ColumnField::new(ident_c.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec as the input (to test nested plans)
        let filter_exec = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: ident_b.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        );

        // Output columns from filter (used by aggregate)
        let filter_output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let filter_output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );
        let filter_output_col_c = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_c.clone(),
            ColumnType::BigInt,
        );

        // Create an aggregate exec with the filter as input
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(filter_output_col_a.clone())),
                alias: ident_a.clone(),
            }],
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(filter_output_col_b.clone())),
                    alias: Ident::new(sum_alias_b.clone()),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(filter_output_col_c.clone())),
                    alias: Ident::new(sum_alias_c.clone()),
                },
            ],
            Ident::new(count_alias.clone()),
            Box::new(DynProofPlan::Filter(filter_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        // Convert to EVM plan
        let evm_aggregate_exec = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![
                column_ref_a.clone(),
                column_ref_b.clone(),
                column_ref_c.clone()
            ],
        )
        .unwrap();

        // Verify nested structure
        assert!(matches!(
            *evm_aggregate_exec.input_plan,
            EVMDynProofPlan::Filter(_)
        ));
        assert_eq!(evm_aggregate_exec.group_by_exprs.len(), 1);
        assert_eq!(evm_aggregate_exec.sum_expr.len(), 2);

        // Roundtrip
        let roundtripped = evm_aggregate_exec
            .try_into_proof_plan(
                &indexset![table_ref],
                &indexset![column_ref_a, column_ref_b, column_ref_c],
                Some(&indexset![
                    ident_a.value,
                    sum_alias_b.clone(),
                    sum_alias_c.clone(),
                    count_alias.clone()
                ]),
            )
            .unwrap();

        // Verify the roundtripped plan
        assert_eq!(roundtripped.group_by_exprs().len(), 1);
        assert_eq!(roundtripped.sum_expr().len(), 2);
        assert!(matches!(*roundtripped.input(), DynProofPlan::Filter(_)));
    }

    #[test]
    fn aggregate_exec_fails_with_table_not_found_in_input() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Output columns (from table's perspective)
        let output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create a table exec with a missing table reference
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(missing_table_ref, column_fields);

        // Create an aggregate exec
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_a)),
                alias: ident_a.clone(),
            }],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_b)),
                alias: Ident::new(sum_alias),
            }],
            Ident::new(count_alias),
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        let result = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn aggregate_exec_fails_with_invalid_output_column_names() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec as input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Output columns
        let output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create an aggregate exec
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_a)),
                alias: ident_a.clone(),
            }],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_b)),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        let evm_aggregate_exec = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Try to convert back with incorrect output column names (missing count_alias)
        let result = EVMAggregateExec::try_into_proof_plan(
            &evm_aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![ident_a.value.clone(), sum_alias.clone()]), // Missing count_alias
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));

        // Try with wrong count alias name
        let wrong_count_alias = "wrong_count".to_string();
        let result = EVMAggregateExec::try_into_proof_plan(
            &evm_aggregate_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
            Some(&indexset![ident_a.value, sum_alias, wrong_count_alias]),
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));
    }
}
