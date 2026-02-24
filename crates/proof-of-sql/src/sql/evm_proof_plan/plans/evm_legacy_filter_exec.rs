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
        proof_exprs::{AliasedDynProofExpr, TableExpr},
        proof_plans::LegacyFilterExec,
    },
};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a filter execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMLegacyFilterExec {
    table_number: usize,
    where_clause: EVMDynProofExpr,
    results: Vec<EVMDynProofExpr>,
}

impl EVMLegacyFilterExec {
    /// Try to create a `LegacyFilterExec` from a `proof_plans::LegacyFilterExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &LegacyFilterExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table().table_ref)
                .ok_or(EVMProofPlanError::TableNotFound)?,
            results: plan
                .aliased_results()
                .iter()
                .map(|result| EVMDynProofExpr::try_from_proof_expr(&result.expr, column_refs))
                .collect::<Result<_, _>>()?,
            where_clause: EVMDynProofExpr::try_from_proof_expr(plan.where_clause(), column_refs)?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<LegacyFilterExec> {
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, self.results.len())?;
        Ok(LegacyFilterExec::new(
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

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType, LiteralValue, TableRef},
            map::indexset,
        },
        sql::{
            evm_proof_plan::{
                exprs::{EVMColumnExpr, EVMEqualsExpr, EVMLiteralExpr},
                plans::EVMLegacyFilterExec,
                EVMDynProofExpr, EVMProofPlanError,
            },
            proof_exprs::{
                AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr, TableExpr,
            },
            proof_plans::LegacyFilterExec,
        },
    };
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_put_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let filter_exec = LegacyFilterExec::new(
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

        let evm_filter_exec = EVMLegacyFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        let expected_evm_filter_exec = EVMLegacyFilterExec {
            table_number: 0,
            where_clause: EVMDynProofExpr::Equals(EVMEqualsExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr::new(0)),
                EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(5))),
            )),
            results: vec![EVMDynProofExpr::Column(EVMColumnExpr::new(1))],
        };

        assert_eq!(evm_filter_exec, expected_evm_filter_exec);

        // Roundtrip
        let roundtripped_filter_exec = EVMLegacyFilterExec::try_into_proof_plan(
            &evm_filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![alias]),
        )
        .unwrap();
        assert_eq!(roundtripped_filter_exec, filter_exec);

        assert!(matches!(
            EVMLegacyFilterExec::try_into_proof_plan(
                &evm_filter_exec,
                &indexset![],
                &indexset![],
                Some(&indexset![]),
            )
            .unwrap_err(),
            EVMProofPlanError::InvalidOutputColumnName
        ));
    }
}
