use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{
            plans::{
                EVMAggregateExec, EVMEmptyExec, EVMFilterExec, EVMGroupByExec, EVMLegacyFilterExec,
                EVMProjectionExec, EVMSliceExec, EVMSortMergeJoinExec, EVMTableExec, EVMUnionExec,
            },
            EVMProofPlanResult,
        },
        proof_plans::DynProofPlan,
    },
};
use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Represents a plan that can be serialized for EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EVMDynProofPlan {
    LegacyFilter(EVMLegacyFilterExec),
    Empty(EVMEmptyExec),
    Table(EVMTableExec),
    Projection(EVMProjectionExec),
    Slice(EVMSliceExec),
    GroupBy(EVMGroupByExec),
    Union(EVMUnionExec),
    SortMergeJoin(EVMSortMergeJoinExec),
    Filter(EVMFilterExec),
    Aggregate(EVMAggregateExec),
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
                EVMTableExec::try_from_proof_plan(table_exec, table_refs, column_refs)
                    .map(Self::Table)
            }
            DynProofPlan::LegacyFilter(filter_exec) => {
                EVMLegacyFilterExec::try_from_proof_plan(filter_exec, table_refs, column_refs)
                    .map(Self::LegacyFilter)
            }
            DynProofPlan::Projection(projection_exec) => {
                EVMProjectionExec::try_from_proof_plan(projection_exec, table_refs, column_refs)
                    .map(Self::Projection)
            }
            DynProofPlan::Slice(slice_exec) => {
                EVMSliceExec::try_from_proof_plan(slice_exec, table_refs, column_refs)
                    .map(Self::Slice)
            }
            DynProofPlan::GroupBy(group_by_exec) => {
                EVMGroupByExec::try_from_proof_plan(group_by_exec, table_refs, column_refs)
                    .map(Self::GroupBy)
            }
            DynProofPlan::Union(union_exec) => {
                EVMUnionExec::try_from_proof_plan(union_exec, table_refs, column_refs)
                    .map(Self::Union)
            }
            DynProofPlan::SortMergeJoin(sort_merge_join_exec) => {
                EVMSortMergeJoinExec::try_from_proof_plan(
                    sort_merge_join_exec,
                    table_refs,
                    column_refs,
                )
                .map(Self::SortMergeJoin)
            }
            DynProofPlan::Filter(filter_exec) => {
                EVMFilterExec::try_from_proof_plan(filter_exec, table_refs, column_refs)
                    .map(Self::Filter)
            }
            DynProofPlan::Aggregate(aggregate_exec) => {
                EVMAggregateExec::try_from_proof_plan(aggregate_exec, table_refs, column_refs)
                    .map(Self::Aggregate)
            }
        }
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<DynProofPlan> {
        match self {
            EVMDynProofPlan::Empty(_empty_exec) => {
                Ok(DynProofPlan::Empty(EVMEmptyExec::try_into_proof_plan()))
            }
            EVMDynProofPlan::Table(table_exec) => Ok(DynProofPlan::Table(
                table_exec.try_into_proof_plan(table_refs, column_refs)?,
            )),
            EVMDynProofPlan::LegacyFilter(filter_exec) => Ok(DynProofPlan::LegacyFilter(
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
                slice_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::GroupBy(group_by_exec) => Ok(DynProofPlan::GroupBy(
                group_by_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::Union(union_exec) => Ok(DynProofPlan::Union(
                union_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::SortMergeJoin(sort_merge_join_exec) => {
                Ok(DynProofPlan::SortMergeJoin(
                    sort_merge_join_exec.try_into_proof_plan(table_refs, column_refs)?,
                ))
            }
            EVMDynProofPlan::Filter(filter_exec) => Ok(DynProofPlan::Filter(
                filter_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::Aggregate(aggregate_exec) => Ok(DynProofPlan::Aggregate(
                aggregate_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
        }
    }
}
