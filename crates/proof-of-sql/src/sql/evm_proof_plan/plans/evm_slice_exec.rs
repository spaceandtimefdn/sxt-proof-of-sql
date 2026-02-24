use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{plans::EVMDynProofPlan, EVMProofPlanResult},
        proof_plans::SliceExec,
    },
};
use serde::{Deserialize, Serialize};

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
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<SliceExec> {
        Ok(SliceExec::new(
            Box::new(self.input_plan.try_into_proof_plan(
                table_refs,
                column_refs,
                output_column_names,
            )?),
            self.skip,
            self.fetch,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnField, ColumnRef, ColumnType, TableRef},
            map::{indexset, IndexSet},
        },
        sql::{
            evm_proof_plan::plans::{EVMDynProofPlan, EVMSliceExec},
            proof_plans::{DynProofPlan, SliceExec, TableExec},
        },
    };
    use sqlparser::ast::Ident;

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
            Some(&IndexSet::default()),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_slice_exec.skip(), skip);
        assert_eq!(roundtripped_slice_exec.fetch(), fetch);
        assert!(matches!(
            *roundtripped_slice_exec.input(),
            DynProofPlan::Table(_)
        ));

        let evm_dyn_slice_exec = EVMDynProofPlan::Slice(evm_slice_exec);
        let dyn_slice_exec = evm_dyn_slice_exec
            .try_into_proof_plan(
                &indexset![table_ref.clone()],
                &indexset![column_ref_a.clone(), column_ref_b.clone()],
                Some(&IndexSet::default()),
            )
            .unwrap();

        assert_eq!(dyn_slice_exec, DynProofPlan::Slice(slice_exec));
    }
}
