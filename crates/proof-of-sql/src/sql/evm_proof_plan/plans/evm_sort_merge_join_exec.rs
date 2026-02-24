use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{plans::EVMDynProofPlan, EVMProofPlanResult},
        proof_plans::SortMergeJoinExec,
    },
};
use alloc::{boxed::Box, string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a group by execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMSortMergeJoinExec {
    left: Box<EVMDynProofPlan>,
    right: Box<EVMDynProofPlan>,
    left_join_column_indexes: Vec<usize>,
    right_join_column_indexes: Vec<usize>,
    result_aliases: Vec<String>,
}

impl EVMSortMergeJoinExec {
    pub(crate) fn try_from_proof_plan(
        plan: &SortMergeJoinExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        let left = Box::new(EVMDynProofPlan::try_from_proof_plan(
            plan.left_plan(),
            table_refs,
            column_refs,
        )?);
        let right = Box::new(EVMDynProofPlan::try_from_proof_plan(
            plan.right_plan(),
            table_refs,
            column_refs,
        )?);
        let left_join_column_indexes = plan.left_join_column_indexes().clone();
        let right_join_column_indexes = plan.right_join_column_indexes().clone();
        let result_aliases = plan
            .result_idents()
            .iter()
            .map(|id| id.value.clone())
            .collect();

        Ok(Self {
            left,
            right,
            left_join_column_indexes,
            right_join_column_indexes,
            result_aliases,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<SortMergeJoinExec> {
        let left = Box::new(
            self.left
                .try_into_proof_plan(table_refs, column_refs, None)?,
        );
        let right = Box::new(
            self.right
                .try_into_proof_plan(table_refs, column_refs, None)?,
        );
        let left_join_column_indexes = self.left_join_column_indexes.clone();
        let right_join_column_indexes = self.right_join_column_indexes.clone();
        let result_idents = self.result_aliases.iter().map(Ident::new).collect();

        Ok(SortMergeJoinExec::new(
            left,
            right,
            left_join_column_indexes,
            right_join_column_indexes,
            result_idents,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{ColumnField, ColumnType},
            map::indexset,
        },
        sql::{
            proof::ProofPlan,
            proof_plans::{DynProofPlan, SortMergeJoinExec},
        },
    };

    #[test]
    fn we_can_put_sort_merge_join_exec_in_evm() {
        let left_table_ref: TableRef = "namespace.left_table".parse().unwrap();
        let right_table_ref: TableRef = "namespace.right_table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();

        let left_column_ref_a =
            ColumnRef::new(left_table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let left_column_ref_b =
            ColumnRef::new(left_table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        let right_column_ref_a =
            ColumnRef::new(right_table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let right_column_ref_b =
            ColumnRef::new(right_table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create columns fields to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];

        // Create a sort merge join exec
        let sort_merge_join_exec = DynProofPlan::SortMergeJoin(SortMergeJoinExec::new(
            Box::new(DynProofPlan::new_table(
                left_table_ref.clone(),
                column_fields.clone(),
            )),
            Box::new(DynProofPlan::new_table(
                right_table_ref.clone(),
                column_fields,
            )),
            vec![0],
            vec![0],
            vec![ident_a, ident_b, ident_c],
        ));
        let output_column_names = sort_merge_join_exec
            .get_column_result_fields()
            .iter()
            .map(|cr| cr.name().to_string())
            .collect();

        let table_refs = &indexset![left_table_ref, right_table_ref];
        let column_refs = &indexset![
            left_column_ref_a,
            left_column_ref_b,
            right_column_ref_a,
            right_column_ref_b
        ];

        // Convert to EVM plan
        let evm_sort_merge_join_exec =
            EVMDynProofPlan::try_from_proof_plan(&sort_merge_join_exec, table_refs, column_refs)
                .unwrap();

        let round_tripped_sort_merge_join_exec = evm_sort_merge_join_exec
            .try_into_proof_plan(table_refs, column_refs, Some(&output_column_names))
            .unwrap();
        assert_eq!(sort_merge_join_exec, round_tripped_sort_merge_join_exec);
    }
}
