use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{
            plans::{try_unwrap_output_column_names, EVMDynProofPlan},
            EVMDynProofExpr, EVMProofPlanResult,
        },
        proof_exprs::AliasedDynProofExpr,
        proof_plans::FilterExec,
    },
};
use alloc::{boxed::Box, string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a filter execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMFilterExec {
    input_plan: Box<EVMDynProofPlan>,
    where_clause: EVMDynProofExpr,
    results: Vec<EVMDynProofExpr>,
}

impl EVMFilterExec {
    /// Try to create a `EVMFilterExec` from a `FilterExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &FilterExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        let input_result_column_refs = plan.input().get_column_result_fields_as_references();
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            where_clause: EVMDynProofExpr::try_from_proof_expr(
                plan.where_clause(),
                &input_result_column_refs,
            )?,
            results: plan
                .aliased_results()
                .iter()
                .map(|result| {
                    EVMDynProofExpr::try_from_proof_expr(&result.expr, &input_result_column_refs)
                })
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<FilterExec> {
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, self.results.len())?;
        let input = self
            .input_plan
            .try_into_proof_plan(table_refs, column_refs, None)?;
        let input_result_column_refs = input.get_column_result_fields_as_references();
        Ok(FilterExec::new(
            self.results
                .iter()
                .zip(output_column_names.iter())
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            Box::new(input),
            self.where_clause
                .try_into_proof_expr(&input_result_column_refs)?,
        ))
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
                plans::{EVMDynProofPlan, EVMFilterExec},
                EVMDynProofExpr, EVMProofPlanError,
            },
            proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr},
            proof_plans::{DynProofPlan, FilterExec, SliceExec, TableExec},
        },
    };
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_put_simple_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(Ident::new("a"), ColumnType::BigInt),
            ColumnField::new(Ident::new("b"), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Convert to EVM plan
        let evm_filter_exec = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_filter_exec.results.len(), 1);
        assert!(matches!(
            evm_filter_exec.results[0],
            EVMDynProofExpr::Column(_)
        ));
        assert!(matches!(
            evm_filter_exec.where_clause,
            EVMDynProofExpr::Equals(_)
        ));
        assert!(matches!(
            *evm_filter_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_filter_exec = EVMFilterExec::try_into_proof_plan(
            &evm_filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![alias]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_filter_exec.aliased_results().len(), 1);
        assert!(matches!(
            roundtripped_filter_exec.aliased_results()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert!(matches!(
            roundtripped_filter_exec.where_clause(),
            DynProofExpr::Equals(_)
        ));
        assert!(matches!(
            *roundtripped_filter_exec.input(),
            DynProofPlan::Table(_)
        ));
    }

    #[test]
    fn we_can_put_complex_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a slice exec as the input (to test nested plans)
        let slice_exec = SliceExec::new(Box::new(DynProofPlan::Table(table_exec)), 5, Some(10));

        // Create a filter exec with the slice as input
        let filter_exec = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Slice(slice_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(42),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Convert to EVM plan
        let evm_filter_exec = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify nested structure
        assert!(matches!(
            *evm_filter_exec.input_plan,
            EVMDynProofPlan::Slice(_)
        ));

        // Roundtrip
        let roundtripped = evm_filter_exec
            .try_into_proof_plan(
                &indexset![table_ref],
                &indexset![column_ref_a, column_ref_b],
                Some(&indexset![ident_a.value, ident_c.value]),
            )
            .unwrap();

        // Verify the roundtripped plan
        assert_eq!(roundtripped.aliased_results().len(), 2);
        assert!(matches!(*roundtripped.input(), DynProofPlan::Slice(_)));

        assert!(matches!(
            evm_filter_exec
                .try_into_proof_plan(&indexset![], &indexset![], Some(&indexset![]),)
                .unwrap_err(),
            EVMProofPlanError::InvalidOutputColumnName
        ));
    }

    #[expect(clippy::too_many_lines)]
    #[test]
    fn we_can_put_nested_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();
        let alias_1 = "result_1";
        let alias_2 = "result_2";
        let alias_3 = "result_3";

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let column_ref_c = ColumnRef::new(table_ref.clone(), ident_c.clone(), ColumnType::BigInt);

        let column_ref_1 = ColumnRef::new(table_ref.clone(), alias_1.into(), ColumnType::BigInt);
        let column_ref_2 = ColumnRef::new(table_ref.clone(), alias_2.into(), ColumnType::BigInt);

        // Create a table exec as the base
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
            ColumnField::new(ident_c.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // First filter: filter where a = 10
        let filter_1 = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: Ident::new(alias_1),
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
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(10),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Second filter: filter where b > 20
        let filter_2 = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_1.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: Ident::new(alias_2),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Filter(filter_1)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(20),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Third filter: filter where c = 30
        let filter_3 = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_2.clone())),
                    alias: ident_b.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: Ident::new(alias_3),
                },
            ],
            Box::new(DynProofPlan::Filter(filter_2)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(30),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Convert to EVM plan
        let evm_filter_3 = EVMFilterExec::try_from_proof_plan(
            &filter_3,
            &indexset![table_ref.clone()],
            &indexset![
                column_ref_a.clone(),
                column_ref_b.clone(),
                column_ref_c.clone()
            ],
        )
        .unwrap();

        // Verify nested structure: should have Filter containing Filter containing Table
        assert!(matches!(
            *evm_filter_3.input_plan,
            EVMDynProofPlan::Filter(_)
        ));
        if let EVMDynProofPlan::Filter(ref evm_filter_2) = *evm_filter_3.input_plan {
            assert!(matches!(
                *evm_filter_2.input_plan,
                EVMDynProofPlan::Filter(_)
            ));
            if let EVMDynProofPlan::Filter(ref evm_filter_1) = *evm_filter_2.input_plan {
                assert!(matches!(
                    *evm_filter_1.input_plan,
                    EVMDynProofPlan::Table(_)
                ));
            }
        }

        // Roundtrip
        let roundtripped = evm_filter_3
            .try_into_proof_plan(
                &indexset![table_ref],
                &indexset![column_ref_a, column_ref_b, column_ref_c],
                Some(&indexset![
                    ident_a.value,
                    ident_b.value,
                    alias_3.to_string()
                ]),
            )
            .unwrap();

        // Verify the roundtripped plan has the expected nested structure
        assert_eq!(roundtripped.aliased_results().len(), 3);
        assert!(matches!(*roundtripped.input(), DynProofPlan::Filter(_)));

        // Verify second level
        if let DynProofPlan::Filter(ref filter_2_roundtripped) = *roundtripped.input() {
            assert!(matches!(
                *filter_2_roundtripped.input(),
                DynProofPlan::Filter(_)
            ));

            // Verify third level (innermost)
            if let DynProofPlan::Filter(ref filter_1_roundtripped) = *filter_2_roundtripped.input()
            {
                assert!(matches!(
                    *filter_1_roundtripped.input(),
                    DynProofPlan::Table(_)
                ));
            }
        }
    }

    #[test]
    fn filter_exec_fails_with_column_not_found_in_where_clause() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let missing_ident: Ident = "missing".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let missing_column =
            ColumnRef::new(table_ref.clone(), missing_ident.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec with a where clause that references a missing column
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(missing_column))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let result = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn filter_exec_fails_with_column_not_found_in_results() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let missing_ident: Ident = "missing".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let missing_column =
            ColumnRef::new(table_ref.clone(), missing_ident.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec with a result that references a missing column
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(missing_column)),
                alias: Ident::new(alias),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let result = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn filter_exec_fails_with_table_not_found_in_input() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec with a missing table reference
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(missing_table_ref, column_fields);

        // Create a filter exec
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let result = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }
}
