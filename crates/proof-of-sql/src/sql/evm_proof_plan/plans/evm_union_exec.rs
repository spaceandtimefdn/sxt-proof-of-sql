use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        evm_proof_plan::{plans::EVMDynProofPlan, EVMProofPlanResult},
        proof_plans::UnionExec,
    },
};
use serde::{Deserialize, Serialize};

/// Represents a union execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMUnionExec {
    pub(super) inputs: Vec<EVMDynProofPlan>,
}

impl EVMUnionExec {
    /// Try to create a `EVMUnionExec` from a `UnionExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &UnionExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // Map column expressions to their indices in column_refs
        Ok(Self {
            inputs: plan
                .input_plans()
                .iter()
                .map(|plan| EVMDynProofPlan::try_from_proof_plan(plan, table_refs, column_refs))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<UnionExec> {
        // We need not supply the output column names to anything other than the first input plan
        let output_column_names_collection = core::iter::once(output_column_names)
            .chain(core::iter::repeat_with(|| None))
            .take(self.inputs.len());
        Ok(UnionExec::try_new(
            self.inputs
                .iter()
                .zip(output_column_names_collection)
                .map(|(plan, output_column_names)| {
                    plan.try_into_proof_plan(table_refs, column_refs, output_column_names)
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?)
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
            evm_proof_plan::plans::EVMUnionExec,
            proof::ProofPlan,
            proof_exprs::{AddExpr, AliasedDynProofExpr, ColumnExpr, DynProofExpr},
            proof_plans::{DynProofPlan, ProjectionExec, TableExec, UnionExec},
        },
    };
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_put_union_exec_in_evm() {
        let top_table_ref: TableRef = "namespace.top_table".parse().unwrap();
        let bottom_table_ref: TableRef = "namespace.bottom_table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();

        let top_column_ref_a =
            ColumnRef::new(top_table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let top_column_ref_b =
            ColumnRef::new(top_table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        let bottom_column_ref_a = ColumnRef::new(
            bottom_table_ref.clone(),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let bottom_column_ref_b = ColumnRef::new(
            bottom_table_ref.clone(),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create columns fields to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];

        // Create a union exec
        let union_exec = UnionExec::try_new(vec![
            DynProofPlan::Projection(ProjectionExec::new(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::Add(
                        AddExpr::try_new(
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_a.clone(),
                                ColumnType::BigInt,
                            )))),
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_b.clone(),
                                ColumnType::BigInt,
                            )))),
                        )
                        .unwrap(),
                    ),
                    alias: "ab_sum".into(),
                }],
                Box::new(DynProofPlan::Table(TableExec::new(
                    top_table_ref.clone(),
                    column_fields.clone(),
                ))),
            )),
            DynProofPlan::Projection(ProjectionExec::new(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::Add(
                        AddExpr::try_new(
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_a.clone(),
                                ColumnType::BigInt,
                            )))),
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_b.clone(),
                                ColumnType::BigInt,
                            )))),
                        )
                        .unwrap(),
                    ),
                    alias: "ab_sum".into(),
                }],
                Box::new(DynProofPlan::Table(TableExec::new(
                    bottom_table_ref.clone(),
                    column_fields.clone(),
                ))),
            )),
        ])
        .unwrap();
        let output_column_names = union_exec
            .get_column_result_fields()
            .iter()
            .map(|cr| cr.name().to_string())
            .collect();

        let table_refs = &indexset![top_table_ref, bottom_table_ref];
        let column_refs = &indexset![
            top_column_ref_a,
            top_column_ref_b,
            bottom_column_ref_a,
            bottom_column_ref_b
        ];

        // Convert to EVM plan
        let evm_union_exec =
            EVMUnionExec::try_from_proof_plan(&union_exec, table_refs, column_refs).unwrap();

        assert_eq!(evm_union_exec.inputs.len(), 2);

        let round_tripped_union_exec = evm_union_exec
            .try_into_proof_plan(table_refs, column_refs, Some(&output_column_names))
            .unwrap();
        assert_eq!(
            union_exec.get_column_result_fields(),
            round_tripped_union_exec.get_column_result_fields()
        );
    }
}
