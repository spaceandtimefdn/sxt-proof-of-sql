use crate::{
    base::{
        database::{ColumnField, ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{EVMProofPlanError, EVMProofPlanResult},
        proof_plans::TableExec,
    },
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Represents a table execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMTableExec {
    table_number: usize,
    column_numbers: Vec<usize>,
}

impl EVMTableExec {
    /// Try to create a `EVMTableExec` from a `TableExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &TableExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(plan.table_ref())
                .ok_or(EVMProofPlanError::TableNotFound)?,
            column_numbers: column_refs
                .iter()
                .enumerate()
                .filter_map(|(i, col_ref)| (&col_ref.table_ref() == plan.table_ref()).then_some(i))
                .collect(),
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

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnField, ColumnRef, ColumnType, TableRef},
            map::indexset,
        },
        sql::{
            evm_proof_plan::{plans::EVMTableExec, EVMProofPlanError},
            proof_plans::TableExec,
        },
    };
    use sqlparser::ast::Ident;

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

        let evm_table_exec = EVMTableExec::try_from_proof_plan(
            &table_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        let expected_evm_table_exec = EVMTableExec {
            table_number: 0,
            column_numbers: vec![0, 1],
        };

        assert_eq!(evm_table_exec, expected_evm_table_exec);

        // Roundtrip
        let roundtripped_table_exec = EVMTableExec::try_into_proof_plan(
            &evm_table_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        assert_eq!(
            *roundtripped_table_exec.table_ref(),
            *table_exec.table_ref()
        );
        assert_eq!(roundtripped_table_exec.schema().len(), 2);
    }

    #[test]
    fn table_exec_fails_with_table_not_found_into_proof_plan() {
        let evm_table_exec = EVMTableExec {
            table_number: 0,
            column_numbers: Vec::new(),
        };

        // Use an empty table_refs to trigger TableNotFound
        let result = EVMTableExec::try_into_proof_plan(&evm_table_exec, &indexset![], &indexset![]);

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn table_exec_fails_with_table_not_found_from_proof_plan() {
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();

        let column_fields = vec![
            ColumnField::new(Ident::new("a"), ColumnType::BigInt),
            ColumnField::new(Ident::new("b"), ColumnType::BigInt),
        ];

        let table_exec = TableExec::new(missing_table_ref, column_fields);

        let result = EVMTableExec::try_from_proof_plan(&table_exec, &indexset![], &indexset![]);

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }
}
