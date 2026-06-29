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
    use crate::{
        base::database::{
            owned_table_utility::{bigint, owned_table},
            table_utility::{borrowed_bigint, table},
            ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor, TableRef,
        },
        base::{map::indexmap, scalar::test_scalar::TestScalar},
        sql::proof::VerifiableQueryResult,
        sql::proof::{FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate},
    };
    use alloc::collections::VecDeque;
    #[cfg(feature = "blitzar")]
    use blitzar::proof::InnerProductProof;
    use bumpalo::Bump;

    #[test]
    fn demo_mock_plan_reports_its_single_column_and_table_reference() {
        let table_ref = "namespace.table_name".parse::<TableRef>().unwrap();
        let column_ref =
            ColumnRef::new(table_ref.clone(), "column_name".into(), ColumnType::BigInt);
        let plan = DemoMockPlan {
            column: column_ref.clone(),
        };

        let fields = plan.get_column_result_fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name().value, "column_name");
        assert_eq!(fields[0].data_type(), ColumnType::BigInt);

        let column_refs = plan.get_column_references();
        assert_eq!(column_refs.len(), 1);
        assert!(column_refs.contains(&column_ref));

        let table_refs = plan.get_table_references();
        assert_eq!(table_refs.len(), 1);
        assert!(table_refs.contains(&table_ref));
    }

    #[test]
    fn demo_mock_plan_prover_rounds_forward_the_source_table() {
        let alloc = Bump::new();
        let table_ref = "namespace.table_name".parse::<TableRef>().unwrap();
        let column_ref =
            ColumnRef::new(table_ref.clone(), "column_name".into(), ColumnType::BigInt);
        let plan = DemoMockPlan { column: column_ref };
        let source_table = table::<TestScalar>([borrowed_bigint("column_name", [7, 11], &alloc)]);
        let table_map = indexmap! { table_ref => source_table.clone() };

        let mut first_round_builder = FirstRoundBuilder::new(source_table.num_rows());
        let first_round_table = plan
            .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
            .unwrap();
        assert_eq!(
            OwnedTable::from(&first_round_table),
            OwnedTable::from(&source_table)
        );

        let mut final_round_builder = FinalRoundBuilder::new(1, VecDeque::new());
        let final_round_table = plan
            .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
            .unwrap();
        assert_eq!(
            OwnedTable::from(&final_round_table),
            OwnedTable::from(&source_table)
        );
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
