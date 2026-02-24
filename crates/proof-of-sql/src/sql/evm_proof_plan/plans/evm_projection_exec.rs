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
        proof_plans::ProjectionExec,
    },
};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

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
        let input_result_column_refs = plan.input().get_column_result_fields_as_references();
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
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
    ) -> EVMProofPlanResult<ProjectionExec> {
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, self.results.len())?;
        let input = self
            .input_plan
            .try_into_proof_plan(table_refs, column_refs, None)?;
        let input_result_column_refs = input.get_column_result_fields_as_references();
        Ok(ProjectionExec::new(
            self.results
                .iter()
                .zip(output_column_names)
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            Box::new(input),
        ))
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
            evm_proof_plan::{
                plans::{EVMDynProofPlan, EVMProjectionExec},
                EVMDynProofExpr, EVMProofPlanError,
            },
            proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr},
            proof_plans::{DynProofPlan, ProjectionExec, TableExec},
        },
    };
    use sqlparser::ast::Ident;

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
            Some(&indexset![alias]),
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

        assert!(matches!(
            EVMProjectionExec::try_into_proof_plan(
                &evm_projection_exec,
                &indexset![],
                &indexset![],
                Some(&indexset![]),
            )
            .unwrap_err(),
            EVMProofPlanError::InvalidOutputColumnName
        ));
    }
}
