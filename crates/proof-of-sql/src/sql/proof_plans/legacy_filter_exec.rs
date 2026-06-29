use super::fold_vals;
use crate::{
    base::{
        database::{
            filter_util::filter_columns, Column, ColumnField, ColumnRef, LiteralValue, Table,
            TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, HonestProver, ProofPlan, ProverEvaluate,
            ProverHonestyMarker, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, ProofExpr, TableExpr},
        proof_gadgets::{final_round_evaluate_filter, verify_evaluate_filter},
    },
    utils::log,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::marker::PhantomData;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table> WHERE <where_clause>
/// ```
///
/// This differs from the [`LegacyFilterExec`] in that the result is not a sparse table.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OstensibleLegacyFilterExec<H: ProverHonestyMarker> {
    aliased_results: Vec<AliasedDynProofExpr>,
    table: TableExpr,
    /// The boolean expression used to filter rows from the table.
    where_clause: DynProofExpr,
    phantom: PhantomData<H>,
}

impl<H: ProverHonestyMarker> OstensibleLegacyFilterExec<H> {
    /// Creates a new filter expression.
    pub fn new(
        aliased_results: Vec<AliasedDynProofExpr>,
        table: TableExpr,
        where_clause: DynProofExpr,
    ) -> Self {
        Self {
            aliased_results,
            table,
            where_clause,
            phantom: PhantomData,
        }
    }

    /// Get the aliased results
    pub fn aliased_results(&self) -> &[AliasedDynProofExpr] {
        &self.aliased_results
    }

    /// Get the table expression
    pub fn table(&self) -> &TableExpr {
        &self.table
    }

    /// Get the where clause expression
    pub fn where_clause(&self) -> &DynProofExpr {
        &self.where_clause
    }
}

impl<H: ProverHonestyMarker> ProofPlan for OstensibleLegacyFilterExec<H>
where
    OstensibleLegacyFilterExec<H>: ProverEvaluate,
{
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;

        let input_chi_eval = *chi_eval_map
            .get(&self.table.table_ref)
            .expect("Chi eval not found");
        let accessor = accessor
            .get(&self.table.table_ref)
            .cloned()
            .unwrap_or_else(|| [].into_iter().collect());
        // 1. selection
        let selection_eval =
            self.where_clause
                .verifier_evaluate(builder, &accessor, input_chi_eval.0, params)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| {
                    aliased_expr.expr.verifier_evaluate(
                        builder,
                        &accessor,
                        input_chi_eval.0,
                        params,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        // 3. filtered_columns
        let filtered_columns_evals =
            builder.try_consume_first_round_mle_evaluations(self.aliased_results.len())?;
        assert!(filtered_columns_evals.len() == self.aliased_results.len());

        let output_chi_eval = builder.try_consume_chi_evaluation()?;

        let c_fold_eval = alpha * fold_vals(beta, &columns_evals);
        let d_fold_eval = alpha * fold_vals(beta, &filtered_columns_evals);

        verify_evaluate_filter(
            builder,
            c_fold_eval,
            d_fold_eval,
            input_chi_eval.0,
            output_chi_eval.0,
            selection_eval,
        )?;
        Ok(TableEvaluation::new(
            filtered_columns_evals,
            output_chi_eval,
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| {
                ColumnField::new(aliased_expr.alias.clone(), aliased_expr.expr.data_type())
            })
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();

        for aliased_expr in &self.aliased_results {
            aliased_expr.expr.get_column_references(&mut columns);
        }

        self.where_clause.get_column_references(&mut columns);

        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref.clone()])
    }
}

/// Alias for a filter expression with a honest prover.
pub type LegacyFilterExec = OstensibleLegacyFilterExec<HonestProver>;

impl ProverEvaluate for LegacyFilterExec {
    #[tracing::instrument(
        name = "LegacyFilterExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self
            .where_clause
            .first_round_evaluate(alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr.expr.first_round_evaluate(alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;

        // Compute filtered_columns and indexes
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias.clone())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(output_length);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "LegacyFilterExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        log::log_memory_usage("Start");
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self
            .where_clause
            .final_round_evaluate(builder, alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();

        // 2. columns
        let columns: Vec<_> = self
            .aliased_results
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // Compute filtered_columns
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);

        final_round_evaluate_filter::<S>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            selection,
            &filtered_columns,
            table.num_rows(),
            result_len,
        );
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results
                .iter()
                .map(|expr| expr.alias.clone())
                .zip(filtered_columns),
            TableOptions::new(Some(output_length)),
        )
        .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{
                owned_table_utility::{bigint, boolean, owned_table},
                table_utility::{borrowed_bigint, borrowed_boolean, table},
                ColumnType, OwnedTable,
            },
            map::{indexmap, IndexSet},
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::ProvableQueryResult,
            proof_exprs::{ColumnExpr, LiteralExpr},
        },
    };
    use alloc::{collections::VecDeque, vec};

    fn table_ref() -> TableRef {
        TableRef::new("sxt", "orders")
    }

    fn column_expr(table_ref: &TableRef, name: &str, data_type: ColumnType) -> DynProofExpr {
        DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
            table_ref.clone(),
            name.into(),
            data_type,
        )))
    }

    fn aliased_column(
        table_ref: &TableRef,
        name: &str,
        data_type: ColumnType,
    ) -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: column_expr(table_ref, name, data_type),
            alias: name.into(),
        }
    }

    fn plan(table_ref: &TableRef) -> LegacyFilterExec {
        LegacyFilterExec::new(
            vec![
                aliased_column(table_ref, "amount", ColumnType::BigInt),
                aliased_column(table_ref, "flag", ColumnType::Boolean),
            ],
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::try_new_equals(
                column_expr(table_ref, "amount", ColumnType::BigInt),
                DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(5))),
            )
            .unwrap(),
        )
    }

    #[test]
    fn we_can_read_legacy_filter_metadata_without_blitzar() {
        let table_ref = table_ref();
        let plan = plan(&table_ref);

        assert_eq!(plan.aliased_results().len(), 2);
        assert_eq!(plan.aliased_results()[0].alias.value, "amount");
        assert_eq!(plan.table().table_ref, table_ref);
        assert_eq!(plan.where_clause().data_type(), ColumnType::Boolean);
        assert_eq!(
            plan.get_column_result_fields(),
            vec![
                ColumnField::new("amount".into(), ColumnType::BigInt),
                ColumnField::new("flag".into(), ColumnType::Boolean),
            ]
        );
        assert_eq!(
            plan.get_column_references(),
            IndexSet::from_iter([
                ColumnRef::new(table_ref.clone(), "amount".into(), ColumnType::BigInt),
                ColumnRef::new(table_ref.clone(), "flag".into(), ColumnType::Boolean),
            ])
        );
        assert_eq!(
            plan.get_table_references(),
            IndexSet::from_iter([table_ref])
        );
    }

    #[test]
    fn we_can_evaluate_legacy_filter_rounds_without_blitzar() {
        let alloc = Bump::new();
        let table_ref = table_ref();
        let data = table::<TestScalar>([
            borrowed_bigint("amount", [1_i64, 5, 5, 9], &alloc),
            borrowed_boolean("flag", [true, false, true, false], &alloc),
        ]);
        let table_map = indexmap! {
            table_ref.clone() => data
        };
        let plan = plan(&table_ref);
        let fields = plan.get_column_result_fields();

        let mut first_round_builder = FirstRoundBuilder::new(4);
        let first_round_result: OwnedTable<TestScalar> = ProvableQueryResult::from(
            plan.first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
                .unwrap(),
        )
        .to_owned_table(&fields)
        .unwrap();
        let expected = owned_table::<TestScalar>([
            bigint("amount", [5_i64, 5]),
            boolean("flag", [false, true]),
        ]);
        assert_eq!(first_round_result, expected);
        assert_eq!(first_round_builder.pcs_proof_mles().len(), 2);

        let mut final_round_builder = FinalRoundBuilder::new(
            4,
            VecDeque::from([TestScalar::from(7_u64), TestScalar::from(13_u64)]),
        );
        let final_round_result: OwnedTable<TestScalar> = ProvableQueryResult::from(
            plan.final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
                .unwrap(),
        )
        .to_owned_table(&fields)
        .unwrap();

        assert_eq!(final_round_result, expected);
        assert!(final_round_builder.num_sumcheck_subpolynomials() >= 3);
        assert!(final_round_builder.pcs_proof_mles().len() >= 2);
    }
}
