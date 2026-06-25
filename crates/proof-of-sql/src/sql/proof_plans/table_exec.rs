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
    use super::TableExec;
    use crate::{
        base::database::{ColumnField, ColumnType, TableRef},
        sql::proof::ProofPlan,
    };
    use sqlparser::ast::Ident;

    fn make_exec() -> TableExec {
        TableExec::new(
            TableRef::new("s", "t"),
            alloc::vec![ColumnField::new(Ident::new("col"), ColumnType::BigInt)],
        )
    }

    #[test]
    fn table_ref_returns_correct_ref() {
        let e = make_exec();
        assert_eq!(e.table_ref().table_id().value.as_str(), "t");
    }

    #[test]
    fn schema_returns_columns() {
        let e = make_exec();
        assert_eq!(e.schema().len(), 1);
    }

    #[test]
    fn empty_schema_creates_zero_columns() {
        let e = TableExec::new(TableRef::new("", "t"), alloc::vec![]);
        assert!(e.schema().is_empty());
    }

    #[test]
    fn get_column_result_fields_matches_schema() {
        let e = make_exec();
        assert_eq!(e.get_column_result_fields().len(), 1);
    }

    #[test]
    fn get_table_references_contains_the_table() {
        let e = make_exec();
        assert_eq!(e.get_table_references().len(), 1);
    }

    #[test]
    fn equality_holds_for_same_values() {
        let a = make_exec();
        let b = make_exec();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e = make_exec();
        assert!(alloc::format!("{e:?}").contains("TableExec"));
    }
}
