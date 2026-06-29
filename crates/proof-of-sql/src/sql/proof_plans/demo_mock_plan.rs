use crate::{
    base::{
        database::{ColumnField, ColumnRef, LiteralValue, Table, TableEvaluation, TableRef},
        map::{indexset, IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::Serialize;
use sqlparser::ast::Ident;

#[derive(Debug, Serialize)]
pub(crate) struct DemoMockPlan {
    pub column: ColumnRef,
}

impl ProofPlan for DemoMockPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        _params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // place verification logic you want to test here

        Ok(TableEvaluation::new(
            vec![accessor[&self.column.table_ref()][&self.column.column_id()]],
            chi_eval_map[&self.column.table_ref()],
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new(
            self.column.column_id(),
            *self.column.column_type(),
        )]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column.clone()}
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
    }
}

impl ProverEvaluate for DemoMockPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        // place prover logic you want to test here

        Ok(table_map[&self.column.table_ref()].clone())
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        // place prover logic you want to test here

        Ok(table_map[&self.column.table_ref()].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::DemoMockPlan;
    #[cfg(feature = "blitzar")]
    use crate::{
        base::database::{
            owned_table_utility::{bigint, owned_table},
            OwnedTableTestAccessor,
        },
        sql::proof::VerifiableQueryResult,
    };
    use crate::{
        base::{
            database::{
                table_utility::{borrowed_bigint, table},
                ColumnField, ColumnRef, ColumnType, TableRef,
            },
            map::{indexmap, indexset},
            scalar::test_scalar::TestScalar,
        },
        sql::proof::{
            mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder,
            FirstRoundBuilder, ProofPlan, ProverEvaluate,
        },
    };
    use alloc::{collections::VecDeque, vec};
    #[cfg(feature = "blitzar")]
    use blitzar::proof::InnerProductProof;
    use bumpalo::Bump;

    #[test]
    fn demo_mock_plan_reports_selected_column_metadata_and_refs() {
        let table_ref = TableRef::new("namespace", "table_name");
        let column_ref =
            ColumnRef::new(table_ref.clone(), "column_name".into(), ColumnType::BigInt);
        let plan = DemoMockPlan {
            column: column_ref.clone(),
        };

        assert_eq!(
            plan.get_column_result_fields(),
            vec![ColumnField::new("column_name".into(), ColumnType::BigInt)]
        );
        assert_eq!(plan.get_column_references(), indexset! {column_ref});
        assert_eq!(plan.get_table_references(), indexset! {table_ref});
    }

    #[test]
    fn demo_mock_plan_evaluates_selected_column_for_verifier_and_prover() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("namespace", "table_name");
        let column_ref =
            ColumnRef::new(table_ref.clone(), "column_name".into(), ColumnType::BigInt);
        let plan = DemoMockPlan {
            column: column_ref.clone(),
        };
        let table = table::<TestScalar>([
            borrowed_bigint("column_name", [10_i64, 20, 30], &alloc),
            borrowed_bigint("other", [1_i64, 2, 3], &alloc),
        ]);
        let table_map = indexmap! {table_ref.clone() => table.clone()};

        let mut first_round_builder = FirstRoundBuilder::new(table.num_rows());
        assert_eq!(
            plan.first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[]),
            Ok(table.clone())
        );

        let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
        assert_eq!(
            plan.final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[]),
            Ok(table)
        );

        let accessor = indexmap! {
            table_ref.clone() => indexmap! {
                "column_name".into() => TestScalar::from(77_u64),
                "other".into() => TestScalar::from(12_u64),
            },
        };
        let chi_eval_map = indexmap! {table_ref => (TestScalar::from(5_u64), 3_usize)};
        let mut verification_builder =
            MockVerificationBuilder::new(vec![], 0, vec![], vec![], vec![], vec![], vec![]);

        let evaluation = plan
            .verifier_evaluate(&mut verification_builder, &accessor, &chi_eval_map, &[])
            .unwrap();

        assert_eq!(evaluation.column_evals(), &[TestScalar::from(77_u64)]);
        assert_eq!(evaluation.chi(), (TestScalar::from(5_u64), 3));
    }

    #[test]
    #[cfg(feature = "blitzar")]
    fn we_can_create_and_prove_a_demo_mock_plan() {
        let table_ref = "namespace.table_name".parse::<TableRef>().unwrap();
        let table = owned_table([bigint("column_name", [0, 1, 2, 3])]);
        let column_ref =
            ColumnRef::new(table_ref.clone(), "column_name".into(), ColumnType::BigInt);
        let plan = DemoMockPlan { column: column_ref };
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            table_ref,
            table.clone(),
            0_usize,
            (),
        );
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        let res = verifiable_res
            .verify(&plan, &accessor, &(), &[])
            .expect("verification should suceeed")
            .table;
        assert_eq!(res, table);
    }
}
