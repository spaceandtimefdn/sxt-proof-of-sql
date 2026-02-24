use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{
            plans::try_unwrap_output_column_names, EVMDynProofExpr, EVMProofPlanError,
            EVMProofPlanResult,
        },
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, TableExpr},
        proof_plans::GroupByExec,
    },
};
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a group by execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMGroupByExec {
    table_number: usize,
    group_by_exprs: Vec<usize>,
    where_clause: EVMDynProofExpr,
    sum_expr: Vec<EVMDynProofExpr>,
    count_alias_name: String,
}

impl EVMGroupByExec {
    /// Try to create a `EVMGroupByExec` from a `GroupByExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &GroupByExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // Map column expressions to their indices in column_refs
        let group_by_exprs = plan
            .group_by_exprs()
            .iter()
            .map(|col_expr| {
                column_refs
                    .get_index_of(&col_expr.get_column_reference())
                    .ok_or(EVMProofPlanError::ColumnNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table().table_ref)
                .ok_or(EVMProofPlanError::TableNotFound)?,
            group_by_exprs: group_by_exprs.clone(),
            sum_expr: plan
                .sum_expr()
                .iter()
                .map(|aliased_expr| {
                    EVMDynProofExpr::try_from_proof_expr(&aliased_expr.expr, column_refs)
                })
                .collect::<Result<_, _>>()?,
            count_alias_name: plan.count_alias().value.clone(),
            where_clause: EVMDynProofExpr::try_from_proof_expr(plan.where_clause(), column_refs)?,
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
    ) -> EVMProofPlanResult<GroupByExec> {
        let grouping_column_count = self.group_by_exprs.len();
        let required_alias_count = grouping_column_count + self.sum_expr.len() + 1;
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, required_alias_count)?;
        if grouping_column_count > column_refs.len() {
            Err(EVMProofPlanError::ColumnNotFound)?;
        }
        // Convert indices back to ColumnExpr objects
        let group_by_exprs = column_refs
            .iter()
            .take(grouping_column_count)
            .cloned()
            .map(ColumnExpr::new)
            .collect::<Vec<_>>();

        let mut output_column_names = output_column_names.iter().skip(grouping_column_count);

        // Map sum expressions to AliasedDynProofExpr objects
        let sum_expr = self
            .sum_expr
            .iter()
            .zip(&mut output_column_names)
            .map(
                |(expr, alias_name)| -> EVMProofPlanResult<AliasedDynProofExpr> {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(column_refs)?,
                        alias: Ident::new(alias_name),
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        // For safety, check if the provided count_alias_name matches
        if &self.count_alias_name
            != output_column_names
                .next()
                .expect("Value confirmed to exist")
        {
            Err(EVMProofPlanError::InvalidOutputColumnName)?;
        }

        GroupByExec::try_new(
            group_by_exprs,
            sum_expr,
            Ident::new(&self.count_alias_name),
            TableExpr {
                table_ref: table_refs
                    .get_index(self.table_number)
                    .cloned()
                    .ok_or(EVMProofPlanError::TableNotFound)?,
            },
            self.where_clause.try_into_proof_expr(column_refs)?,
        )
        .ok_or(EVMProofPlanError::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType, LiteralValue, TableRef},
            map::indexset,
        },
        sql::{
            evm_proof_plan::{plans::EVMGroupByExec, EVMDynProofExpr, EVMProofPlanError},
            proof_exprs::{
                AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr, TableExpr,
            },
            proof_plans::GroupByExec,
        },
    };
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_put_group_by_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a group by exec
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        // Convert to EVM plan
        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_group_by_exec.table_number, 0);
        assert_eq!(evm_group_by_exec.group_by_exprs, vec![0]); // column_ref_a is at index 0
        assert_eq!(evm_group_by_exec.sum_expr.len(), 1);
        assert!(matches!(
            evm_group_by_exec.sum_expr[0],
            EVMDynProofExpr::Column(_)
        ));
        assert_eq!(evm_group_by_exec.count_alias_name, count_alias);
        assert!(matches!(
            evm_group_by_exec.where_clause,
            EVMDynProofExpr::Equals(_)
        ));

        // Roundtrip
        let roundtripped_group_by_exec = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                count_alias.clone()
            ]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_group_by_exec.group_by_exprs().len(), 1);
        assert_eq!(
            roundtripped_group_by_exec.group_by_exprs()[0].get_column_reference(),
            column_ref_a
        );
        assert_eq!(roundtripped_group_by_exec.sum_expr().len(), 1);
        assert!(matches!(
            roundtripped_group_by_exec.sum_expr()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert_eq!(roundtripped_group_by_exec.count_alias().value, count_alias);
        assert_eq!(roundtripped_group_by_exec.table().table_ref, table_ref);
        assert!(matches!(
            roundtripped_group_by_exec.where_clause(),
            DynProofExpr::Equals(_)
        ));
    }

    #[test]
    fn group_by_exec_fails_with_column_not_found_from_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let missing_ident: Ident = "missing".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let missing_column =
            ColumnRef::new(table_ref.clone(), missing_ident.clone(), ColumnType::BigInt);

        // Create a group by exec with a column that doesn't exist in column_refs
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(missing_column)],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias),
            }],
            Ident::new(count_alias),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let result = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_table_not_found_from_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a group by exec with a table that doesn't exist in table_refs
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias),
            }],
            Ident::new(count_alias),
            TableExpr {
                table_ref: missing_table_ref,
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let result = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_column_not_found_into_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a valid group by exec first
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Now try to convert back with an empty column_refs
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                count_alias.clone()
            ]),
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_table_not_found_into_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a valid group by exec first
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Now try to convert back with an empty table_refs
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                count_alias.clone()
            ]),
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_invalid_output_column_name_into_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a valid group by exec first
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Now try to convert back with incorrect output column names
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
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
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                wrong_count_alias
            ]),
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));
    }
}
