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
    utils::log,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Source [`ProofPlan`] for (sub)queries with table source such as `SELECT col from tab;`
/// Inspired by `DataFusion` data source [`ExecutionPlan`]s such as [`ArrowExec`] and [`CsvExec`].
/// Note that we only need to load the columns we use.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TableExec {
    /// Table reference
    table_ref: TableRef,
    /// Schema of the table
    schema: Vec<ColumnField>,
}

impl TableExec {
    /// Creates a new [`TableExec`].
    #[must_use]
    pub fn new(table_ref: TableRef, schema: Vec<ColumnField>) -> Self {
        Self { table_ref, schema }
    }

    /// Get the table reference
    #[must_use]
    pub fn table_ref(&self) -> &TableRef {
        &self.table_ref
    }

    /// Get the schema
    #[must_use]
    pub fn schema(&self) -> &[ColumnField] {
        &self.schema
    }
}

impl ProofPlan for TableExec {
    #[expect(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let column_evals = self
            .schema
            .iter()
            .map(|field| {
                *accessor
                    .get(self.table_ref())
                    .expect("Table does not exist")
                    .get(&field.name())
                    .expect("Column does not exist")
            })
            .collect::<Vec<_>>();
        let chi_eval = *chi_eval_map
            .get(&self.table_ref)
            .expect("Chi eval not found");
        Ok(TableEvaluation::new(column_evals, chi_eval))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.schema.clone()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.schema
            .iter()
            .map(|field| ColumnRef::new(self.table_ref.clone(), field.name(), field.data_type()))
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.table_ref.clone()}
    }
}

impl ProverEvaluate for TableExec {
    #[tracing::instrument(name = "TableExec::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let first_round_table = table_map
            .get(&self.table_ref)
            .expect("Table not found")
            .clone();

        log::log_memory_usage("End");

        Ok(first_round_table)
    }

    #[tracing::instrument(name = "TableExec::final_round_evaluate", level = "debug", skip_all)]
    #[expect(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let final_round_table = table_map
            .get(&self.table_ref)
            .expect("Table not found")
            .clone();

        log::log_memory_usage("End");

        Ok(final_round_table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{Column, ColumnType},
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::collections::VecDeque;

    fn schema() -> Vec<ColumnField> {
        vec![
            ColumnField::new("height".into(), ColumnType::BigInt),
            ColumnField::new("score".into(), ColumnType::Int),
        ]
    }

    fn table_ref() -> TableRef {
        TableRef::new("sxt", "players")
    }

    #[test]
    fn table_exec_exposes_schema_and_references() {
        let table_ref = table_ref();
        let plan = TableExec::new(table_ref.clone(), schema());

        assert_eq!(plan.table_ref(), &table_ref);
        assert_eq!(plan.schema(), schema());
        assert_eq!(plan.get_column_result_fields(), schema());
        assert_eq!(plan.get_table_references(), indexset! {table_ref.clone()});
        assert_eq!(
            plan.get_column_references(),
            indexset! {
                ColumnRef::new(table_ref.clone(), Ident::new("height"), ColumnType::BigInt),
                ColumnRef::new(table_ref, Ident::new("score"), ColumnType::Int),
            }
        );
    }

    #[test]
    fn table_exec_verifier_uses_schema_order_and_table_chi() {
        let table_ref = table_ref();
        let plan = TableExec::new(table_ref.clone(), schema());
        let mut table_accessors = IndexMap::default();
        table_accessors.insert(Ident::new("score"), TestScalar::from(12_u8));
        table_accessors.insert(Ident::new("height"), TestScalar::from(42_u8));
        let mut accessor = IndexMap::default();
        accessor.insert(table_ref.clone(), table_accessors);
        let mut chi_eval_map = IndexMap::default();
        chi_eval_map.insert(table_ref, (TestScalar::from(7_u8), 3));
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let evaluation = plan
            .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
            .unwrap();

        assert_eq!(
            evaluation.column_evals(),
            &[TestScalar::from(42_u8), TestScalar::from(12_u8)]
        );
        assert_eq!(evaluation.chi(), (TestScalar::from(7_u8), 3));
    }

    #[test]
    fn table_exec_prover_rounds_return_the_referenced_table() {
        let table_ref = table_ref();
        let plan = TableExec::new(table_ref.clone(), schema());
        let table: Table<TestScalar> = Table::try_from_iter([
            (Ident::new("height"), Column::BigInt(&[10, 20, 30])),
            (Ident::new("score"), Column::Int(&[1, 2, 3])),
        ])
        .unwrap();
        let mut table_map = IndexMap::default();
        table_map.insert(table_ref, table.clone());
        let alloc = Bump::new();

        let mut first_round_builder = FirstRoundBuilder::new(0);
        assert_eq!(
            plan.first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
                .unwrap(),
            table
        );

        let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
        assert_eq!(
            plan.final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
                .unwrap(),
            table
        );
    }
}
