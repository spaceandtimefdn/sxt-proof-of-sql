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
                ColumnField, ColumnRef, ColumnType, Table, TableRef,
            },
            map::{indexmap, indexset},
            scalar::test_scalar::TestScalar,
        },
        sql::proof::{
            mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder,
            FirstRoundBuilder, ProofPlan, ProverEvaluate,
        },
    };
    use alloc::collections::VecDeque;
    #[cfg(feature = "blitzar")]
    use blitzar::proof::InnerProductProof;
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    fn demo_plan() -> (DemoMockPlan, TableRef, ColumnRef) {
        let table_ref = TableRef::new("namespace", "table_name");
        let column_ref = ColumnRef::new(
            table_ref.clone(),
            Ident::new("column_name"),
            ColumnType::BigInt,
        );
        (
            DemoMockPlan {
                column: column_ref.clone(),
            },
            table_ref,
            column_ref,
        )
    }

    #[test]
    fn demo_mock_plan_reports_single_column_and_table_references() {
        let (plan, table_ref, column_ref) = demo_plan();

        assert_eq!(
            plan.get_column_result_fields(),
            vec![ColumnField::new("column_name".into(), ColumnType::BigInt)]
        );
        assert_eq!(plan.get_column_references(), indexset! { column_ref });
        assert_eq!(plan.get_table_references(), indexset! { table_ref });
    }

    #[test]
    fn demo_mock_plan_prover_evaluations_return_source_table() {
        let (plan, table_ref, _) = demo_plan();
        let alloc = Bump::new();
        let source_table: Table<'_, TestScalar> =
            table([borrowed_bigint("column_name", [1_i64, 2, 3], &alloc)]);
        let table_map = indexmap! { table_ref => source_table.clone() };
        let mut first_round_builder = FirstRoundBuilder::<TestScalar>::new(source_table.num_rows());
        let mut final_round_builder =
            FinalRoundBuilder::<TestScalar>::new(source_table.num_rows(), VecDeque::new());

        let first_round_result = plan
            .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
            .unwrap();
        let final_round_result = plan
            .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
            .unwrap();

        assert_eq!(first_round_result, source_table);
        assert_eq!(final_round_result, source_table);
        assert!(first_round_builder.pcs_proof_mles().is_empty());
        assert!(final_round_builder.pcs_proof_mles().is_empty());
        assert!(final_round_builder.sumcheck_subpolynomials().is_empty());
    }

    #[test]
    fn demo_mock_plan_verifier_evaluates_selected_column_and_chi() {
        let (plan, table_ref, _) = demo_plan();
        let column_id = Ident::new("column_name");
        let column_eval = TestScalar::from(17);
        let chi = (TestScalar::from(9), 3);
        let accessor = indexmap! {
            table_ref.clone() => indexmap! {
                column_id => column_eval,
            },
        };
        let chi_eval_map = indexmap! { table_ref => chi };
        let mut builder =
            MockVerificationBuilder::new(vec![], 0, vec![], vec![], vec![], vec![], vec![]);

        let evaluation = plan
            .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
            .unwrap();

        assert_eq!(evaluation.column_evals(), &[column_eval]);
        assert_eq!(evaluation.chi(), chi);
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
