use super::{EVMDynProofExpr, EVMProofPlanError, EVMProofPlanResult};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        proof_exprs::{AliasedDynProofExpr, TableExpr},
        proof_plans::{DynProofPlan, EmptyExec, FilterExec, ProjectionExec, SliceExec, TableExec},
    },
};
use alloc::{boxed::Box, string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a plan that can be serialized for EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EVMDynProofPlan {
    Filter(EVMFilterExec),
    Empty(EVMEmptyExec),
    Table(EVMTableExec),
    Projection(EVMProjectionExec),
    Slice(EVMSliceExec),
}

impl EVMDynProofPlan {
    /// Try to create a `EVMDynProofPlan` from a `DynProofPlan`.
    pub(crate) fn try_from_proof_plan(
        plan: &DynProofPlan,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        match plan {
            DynProofPlan::Empty(empty_exec) => {
                Ok(Self::Empty(EVMEmptyExec::try_from_proof_plan(empty_exec)))
            }
            DynProofPlan::Table(table_exec) => {
                EVMTableExec::try_from_proof_plan(table_exec, table_refs).map(Self::Table)
            }
            DynProofPlan::Filter(filter_exec) => {
                EVMFilterExec::try_from_proof_plan(filter_exec, table_refs, column_refs)
                    .map(Self::Filter)
            }
            DynProofPlan::Projection(projection_exec) => {
                EVMProjectionExec::try_from_proof_plan(projection_exec, table_refs, column_refs)
                    .map(Self::Projection)
            }
            DynProofPlan::Slice(slice_exec) => {
                EVMSliceExec::try_from_proof_plan(slice_exec, table_refs, column_refs)
                    .map(Self::Slice)
            }
            _ => Err(EVMProofPlanError::NotSupported),
        }
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: &IndexSet<String>,
    ) -> EVMProofPlanResult<DynProofPlan> {
        match self {
            EVMDynProofPlan::Empty(_empty_exec) => {
                Ok(DynProofPlan::Empty(EVMEmptyExec::try_into_proof_plan()))
            }
            EVMDynProofPlan::Table(table_exec) => Ok(DynProofPlan::Table(
                table_exec.try_into_proof_plan(table_refs, column_refs)?,
            )),
            EVMDynProofPlan::Filter(filter_exec) => Ok(DynProofPlan::Filter(
                filter_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::Projection(projection_exec) => Ok(DynProofPlan::Projection(
                projection_exec.try_into_proof_plan(
                    table_refs,
                    column_refs,
                    output_column_names,
                )?,
            )),
            EVMDynProofPlan::Slice(slice_exec) => Ok(DynProofPlan::Slice(
                slice_exec.try_into_proof_plan(table_refs, column_refs)?,
            )),
        }
    }
}

/// Represents a empty execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMEmptyExec {}

impl EVMEmptyExec {
    /// Create a `EVMEmptyExec` from a `EmptyExec`.
    pub(crate) fn try_from_proof_plan(_plan: &EmptyExec) -> Self {
        Self {}
    }

    /// Convert into a proof plan
    pub(crate) fn try_into_proof_plan() -> EmptyExec {
        EmptyExec::new()
    }
}

/// Represents a table execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMTableExec {
    table_number: usize,
}

impl EVMTableExec {
    /// Try to create a `EVMTableExec` from a `TableExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &TableExec,
        table_refs: &IndexSet<TableRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table_ref)
                .ok_or(EVMProofPlanError::TableNotFound)?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<TableExec> {
        let table_ref = table_refs
            .get_index(self.table_number)
            .cloned()
            .ok_or(EVMProofPlanError::TableNotFound)?;

        // Extract column fields for this table reference
        let schema = column_refs
            .iter()
            .filter(|col_ref| col_ref.table_ref() == table_ref.clone())
            .map(|col_ref| ColumnField::new(col_ref.column_id(), *col_ref.column_type()))
            .collect();

        Ok(TableExec::new(table_ref, schema))
    }
}

/// Represents a filter execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMFilterExec {
    table_number: usize,
    where_clause: EVMDynProofExpr,
    results: Vec<EVMDynProofExpr>,
}

impl EVMFilterExec {
    /// Try to create a `FilterExec` from a `proof_plans::FilterExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &FilterExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table.table_ref)
                .ok_or(EVMProofPlanError::TableNotFound)?,
            results: plan
                .aliased_results
                .iter()
                .map(|result| EVMDynProofExpr::try_from_proof_expr(&result.expr, column_refs))
                .collect::<Result<_, _>>()?,
            where_clause: EVMDynProofExpr::try_from_proof_expr(&plan.where_clause, column_refs)?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: &IndexSet<String>,
    ) -> EVMProofPlanResult<FilterExec> {
        Ok(FilterExec::new(
            self.results
                .iter()
                .zip(output_column_names.iter())
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            TableExpr {
                table_ref: table_refs
                    .get_index(self.table_number)
                    .cloned()
                    .ok_or(EVMProofPlanError::TableNotFound)?,
            },
            self.where_clause.try_into_proof_expr(column_refs)?,
        ))
    }
}

/// Represents a projection execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMProjectionExec {
    input_plan: Box<EVMDynProofPlan>,
    results: Vec<EVMDynProofExpr>,
}

impl EVMProjectionExec {
    /// Try to create a `EVMProjectionExec` from a `ProjectionExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &ProjectionExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            results: plan
                .aliased_results()
                .iter()
                .map(|result| EVMDynProofExpr::try_from_proof_expr(&result.expr, column_refs))
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: &IndexSet<String>,
    ) -> EVMProofPlanResult<ProjectionExec> {
        Ok(ProjectionExec::new(
            self.results
                .iter()
                .zip(output_column_names.iter())
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            Box::new(self.input_plan.try_into_proof_plan(
                table_refs,
                column_refs,
                output_column_names,
            )?),
        ))
    }
}

/// Represents a slice execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMSliceExec {
    input_plan: Box<EVMDynProofPlan>,
    skip: usize,
    fetch: Option<usize>,
}

impl EVMSliceExec {
    /// Try to create a `EVMSliceExec` from a `SliceExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &SliceExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            skip: plan.skip(),
            fetch: plan.fetch(),
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<SliceExec> {
        Ok(SliceExec::new(
            Box::new(self.input_plan.try_into_proof_plan(
                table_refs,
                column_refs,
                &IndexSet::default(),
            )?),
            self.skip,
            self.fetch,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{ColumnType, LiteralValue},
            map::indexset,
        },
        sql::{
            evm_proof_plan::exprs::{EVMColumnExpr, EVMEqualsExpr, EVMLiteralExpr},
            proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr},
            proof_plans::DynProofPlan,
        },
    };

    #[test]
    fn we_can_put_projection_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a projection exec
        let projection_exec = ProjectionExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
        );

        // Convert to EVM plan
        let evm_projection_exec = EVMProjectionExec::try_from_proof_plan(
            &projection_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_projection_exec.results.len(), 1);
        assert!(matches!(
            evm_projection_exec.results[0],
            EVMDynProofExpr::Column(_)
        ));
        assert!(matches!(
            *evm_projection_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_projection_exec = EVMProjectionExec::try_into_proof_plan(
            &evm_projection_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            &indexset![alias],
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_projection_exec.aliased_results().len(), 1);
        assert!(matches!(
            roundtripped_projection_exec.aliased_results()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert!(matches!(
            *roundtripped_projection_exec.input(),
            DynProofPlan::Table(_)
        ));
    }

    #[test]
    fn we_can_put_slice_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a slice exec
        let skip = 10;
        let fetch = Some(5);
        let slice_exec = SliceExec::new(Box::new(DynProofPlan::Table(table_exec)), skip, fetch);

        // Convert to EVM plan
        let evm_slice_exec = EVMSliceExec::try_from_proof_plan(
            &slice_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_slice_exec.skip, skip);
        assert_eq!(evm_slice_exec.fetch, fetch);
        assert!(matches!(
            *evm_slice_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_slice_exec = EVMSliceExec::try_into_proof_plan(
            &evm_slice_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_slice_exec.skip(), skip);
        assert_eq!(roundtripped_slice_exec.fetch(), fetch);
        assert!(matches!(
            *roundtripped_slice_exec.input(),
            DynProofPlan::Table(_)
        ));
    }

    #[test]
    fn we_can_put_empty_exec_in_evm() {
        let empty_exec = EmptyExec::new();

        // Roundtrip
        let roundtripped_empty_exec = EVMEmptyExec::try_into_proof_plan();
        assert_eq!(roundtripped_empty_exec, empty_exec);
    }

    #[test]
    fn we_can_put_table_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        let column_fields = vec![
            ColumnField::new(ident_a, ColumnType::BigInt),
            ColumnField::new(ident_b, ColumnType::BigInt),
        ];

        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        let evm_table_exec =
            EVMTableExec::try_from_proof_plan(&table_exec, &indexset![table_ref.clone()]).unwrap();

        let expected_evm_table_exec = EVMTableExec { table_number: 0 };

        assert_eq!(evm_table_exec, expected_evm_table_exec);

        // Roundtrip
        let roundtripped_table_exec = EVMTableExec::try_into_proof_plan(
            &evm_table_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        assert_eq!(roundtripped_table_exec.table_ref, table_exec.table_ref);
        assert_eq!(roundtripped_table_exec.schema.len(), 2);
    }

    #[test]
    fn table_exec_fails_with_table_not_found_from_proof_plan() {
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();

        let column_fields = vec![
            ColumnField::new(Ident::new("a"), ColumnType::BigInt),
            ColumnField::new(Ident::new("b"), ColumnType::BigInt),
        ];

        let table_exec = TableExec::new(missing_table_ref, column_fields);

        let result = EVMTableExec::try_from_proof_plan(&table_exec, &indexset![]);

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn table_exec_fails_with_table_not_found_into_proof_plan() {
        let evm_table_exec = EVMTableExec { table_number: 0 };

        // Use an empty table_refs to trigger TableNotFound
        let result = EVMTableExec::try_into_proof_plan(&evm_table_exec, &indexset![], &indexset![]);

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn we_can_put_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
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
        );

        let evm_filter_exec = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        let expected_evm_filter_exec = EVMFilterExec {
            table_number: 0,
            where_clause: EVMDynProofExpr::Equals(EVMEqualsExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr::new(0)),
                EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(5)),
            )),
            results: vec![EVMDynProofExpr::Column(EVMColumnExpr::new(1))],
        };

        assert_eq!(evm_filter_exec, expected_evm_filter_exec);

        // Roundtrip
        let roundtripped_filter_exec = EVMFilterExec::try_into_proof_plan(
            &evm_filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            &indexset![alias],
        )
        .unwrap();
        assert_eq!(roundtripped_filter_exec, filter_exec);
    }

    #[test]
    fn we_cannot_put_unsupported_proof_plan_in_evm() {
        // Create a Union of two empty execs which is not supported in EVM
        let empty_exec1 = EmptyExec::new();
        let empty_exec2 = EmptyExec::new();
        let schema: Vec<ColumnField> = Vec::new();

        // Create a union plan with two empty execs
        let plan = DynProofPlan::new_union(
            vec![
                DynProofPlan::Empty(empty_exec1),
                DynProofPlan::Empty(empty_exec2),
            ],
            schema,
        );

        let table_refs = indexset![];
        let column_refs = indexset![];

        assert!(matches!(
            EVMDynProofPlan::try_from_proof_plan(&plan, &table_refs, &column_refs),
            Err(EVMProofPlanError::NotSupported)
        ));
    }
}
