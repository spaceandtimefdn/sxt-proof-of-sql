use super::OstensibleFilterExec;
use crate::{
    base::{
        database::{
            filter_util::*, owned_table_utility::*, Column, LiteralValue, OwnedTableTestAccessor,
            Table, TableOptions, TableRef, TestAccessor,
        },
        map::IndexMap,
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProverEvaluate, ProverHonestyMarker, QueryError,
            VerifiableQueryResult,
        },
        proof_exprs::{
            test_utility::{cols_expr_plan, column, const_int128, equal, tab},
            ProofExpr,
        },
        proof_gadgets::final_round_filter_constraints,
        proof_plans::fold_columns,
    },
    utils::log,
};
use ark_ff::{One, Zero};
use blitzar::proof::InnerProductProof;
use bumpalo::Bump;

#[derive(Debug, PartialEq)]
struct Dishonest;
impl ProverHonestyMarker for Dishonest {}
type DishonestFilterExec = OstensibleFilterExec<Dishonest>;

impl ProverEvaluate for DishonestFilterExec {
    #[tracing::instrument(
        name = "DishonestFilterExec::first_round_evaluate",
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
            .get(&self.table().table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self
            .where_clause()
            .first_round_evaluate(alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        let output_length = selection.iter().filter(|b| **b).count();
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results()
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr.expr.first_round_evaluate(alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // Compute filtered_columns
        let (filtered_columns, _) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results()
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
        name = "DishonestFilterExec::final_round_evaluate",
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

        let table = table_map
            .get(&self.table().table_ref)
            .expect("Table not found");
        // 1. selection
        let selection_column: Column<'a, S> = self
            .where_clause()
            .final_round_evaluate(builder, alloc, table, params)?;
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");
        // 2. columns
        let columns: Vec<_> = self
            .aliased_results()
            .iter()
            .map(|aliased_expr| -> PlaceholderResult<Column<'a, S>> {
                aliased_expr
                    .expr
                    .final_round_evaluate(builder, alloc, table, params)
            })
            .collect::<PlaceholderResult<Vec<_>>>()?;
        // Compute filtered_columns
        let (filtered_columns, output_length) = filter_columns(alloc, &columns, selection);
        let filtered_columns = tamper_column(alloc, filtered_columns);
        // 3. Produce MLEs
        filtered_columns.iter().copied().for_each(|column| {
            builder.produce_intermediate_mle(column);
        });

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        let input_length = table.num_rows();

        let chi_n = alloc.alloc_slice_fill_copy(input_length, true);
        let chi_m = alloc.alloc_slice_fill_copy(output_length, true);

        let c_fold = alloc.alloc_slice_fill_copy(input_length, Zero::zero());
        fold_columns(c_fold, alpha, beta, &columns);
        let d_fold = alloc.alloc_slice_fill_copy(output_length, Zero::zero());
        fold_columns(d_fold, alpha, beta, &filtered_columns);

        let c_star = alloc.alloc_slice_copy(c_fold);
        slice_ops::add_const::<S, S>(c_star, One::one());
        slice_ops::batch_inversion(c_star);

        let d_star = alloc.alloc_slice_copy(d_fold);
        slice_ops::add_const::<S, S>(d_star, One::one());
        slice_ops::batch_inversion(d_star);

        builder.produce_intermediate_mle(c_star as &[_]);
        builder.produce_intermediate_mle(d_star as &[_]);

        final_round_filter_constraints(
            builder, c_star, d_star, selection, c_fold, d_fold, chi_n, chi_m,
        );
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results()
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

/// Tamper with the first element of the first column that is a Scalar. This could be changed for different types of tests.
fn tamper_column<'a, S: Scalar>(
    alloc: &'a Bump,
    mut columns: Vec<Column<'a, S>>,
) -> Vec<Column<'a, S>> {
    for column in &mut columns {
        if let Column::Scalar(tampered_column) = column {
            if !tampered_column.is_empty() {
                let tampered_column = alloc.alloc_slice_copy(tampered_column);
                // The following could be changed for different types of tests, but for the simplest one, we will simply increase the first element by 1.
                tampered_column[0] += S::one();
                *column = Column::Scalar(tampered_column);
                break;
            }
        }
    }
    columns
}

#[test]
fn we_fail_to_verify_a_basic_filter_with_a_dishonest_prover() {
    let data = owned_table([
        bigint("a", [101, 104, 105, 102, 105]),
        bigint("b", [1, 2, 3, 4, 5]),
        int128("c", [1, 2, 3, 4, 5]),
        varchar("d", ["1", "2", "3", "4", "5"]),
        scalar("e", [1, 2, 3, 4, 5]),
    ]);
    let t = TableRef::new("sxt", "t");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(t.clone(), data, 0);
    let expr = DishonestFilterExec::new(
        cols_expr_plan(&t, &["b", "c", "d", "e"], &accessor),
        tab(&t),
        equal(column(&t, "a", &accessor), const_int128(105_i128)),
    );
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &(), &[]).unwrap();
    assert!(matches!(
        res.verify(&expr, &accessor, &(), &[]),
        Err(QueryError::ProofError {
            source: ProofError::VerificationError { .. }
        })
    ));
}
