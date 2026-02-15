//! This module contains the implementation of the `PermutationCheckTestPlan` struct. This struct
//! is used to check whether the permutation check gadgets work correctly.
use super::permutation_check::{final_round_evaluate_permutation_check, verify_permutation_check};
use crate::{
    base::{
        database::{
            table_utility::table_with_row_count, ColumnField, ColumnRef, LiteralValue, Table,
            TableEvaluation, TableOptions, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_gadgets::permutation_check::first_round_evaluate_permutation_check,
    },
};
use bumpalo::{
    collections::{vec::Vec as BumpVec, CollectIn},
    Bump,
};
use serde::Serialize;
use sqlparser::ast::Ident;

#[derive(Debug, Serialize)]
pub struct PermutationCheckTestPlan {
    pub source_table: TableRef,
    pub source_columns: Vec<ColumnRef>,
    pub permutation: Vec<usize>,
}

impl ProverEvaluate for PermutationCheckTestPlan {
    #[doc = "Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        // Check that the source columns belong to the source table
        for col_ref in &self.source_columns {
            assert_eq!(self.source_table, col_ref.table_ref(), "Table not found");
        }
        let source_table: &Table<'a, S> =
            table_map.get(&self.source_table).expect("Table not found");
        let source_columns = self
            .source_columns
            .iter()
            .map(|col_ref| {
                let col = *(source_table
                    .inner_table()
                    .get(&col_ref.column_id())
                    .expect("Column not found in table"));
                builder.produce_intermediate_mle(col);
                col
            })
            .collect_in::<BumpVec<_>>(alloc);
        // Produce chi evaluation length
        builder.produce_chi_evaluation_length(source_table.num_rows());
        first_round_evaluate_permutation_check(
            builder,
            alloc,
            alloc.alloc_slice_fill_copy(source_table.num_rows(), true),
            &source_columns,
            &self.permutation,
        );
        Ok(table_with_row_count([], 0))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        // Check that the source columns belong to the source table
        for col_ref in &self.source_columns {
            assert_eq!(self.source_table, col_ref.table_ref(), "Table not found");
        }
        // Get the table from the map using the table reference
        let source_table: &Table<'a, S> =
            table_map.get(&self.source_table).expect("Table not found");
        let source_columns = self
            .source_columns
            .iter()
            .map(|col_ref| {
                let col = *(source_table
                    .inner_table()
                    .get(&col_ref.column_id())
                    .expect("Column not found in table"));
                col
            })
            .collect_in::<BumpVec<_>>(alloc);
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        // Perform final permutation check
        final_round_evaluate_permutation_check(
            builder,
            alloc,
            alpha,
            beta,
            alloc.alloc_slice_fill_copy(source_table.num_rows(), true),
            &source_columns,
            &self.permutation,
        );
        Ok(table_with_row_count([], 0))
    }
}

impl ProofPlan for PermutationCheckTestPlan {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        Vec::<ColumnField>::new()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.source_columns
            .iter()
            .chain(self.source_columns.iter())
            .cloned()
            .collect()
    }

    #[doc = "Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.source_table.clone()}
    }

    #[doc = "Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        _params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        // Get the challenges from the builder
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let num_columns = self.source_columns.len();
        // Get the columns
        let column_evals = builder.try_consume_first_round_mle_evaluations(num_columns)?;
        // Get the chi evaluations
        let chi_eval = builder.try_consume_chi_evaluation()?.0;
        // Evaluate the verifier
        verify_permutation_check(builder, alpha, beta, chi_eval, &column_evals)?;
        Ok(TableEvaluation::new(vec![], (S::ZERO, 0)))
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::database::{table_utility::*, ColumnType, TableTestAccessor},
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;

    #[test]
    fn we_can_do_minimal_permutation_check() {
        let alloc = Bump::new();
        let source_table = table([borrowed_bigint("a", [1, 2, 3], &alloc)]);
        let permutation = vec![1usize, 2, 0];
        let source_table_ref = TableRef::new("sxt", "source_table");
        let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        let plan = PermutationCheckTestPlan {
            source_table: source_table_ref.clone(),
            source_columns: vec![ColumnRef::new(
                source_table_ref,
                "a".into(),
                ColumnType::BigInt,
            )],
            permutation,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_ok());
    }

    #[test]
    fn we_can_do_permutation_check() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2, 3], &alloc),
            borrowed_varchar("b", ["Space", "and", "Time"], &alloc),
            borrowed_boolean("c", [true, false, true], &alloc),
            borrowed_bigint("d", [5, 6, 7], &alloc),
        ]);
        let source_table_ref = TableRef::new("sxt", "source_table");
        let permutation = vec![1usize, 2, 0];
        let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        let plan = PermutationCheckTestPlan {
            source_table: source_table_ref.clone(),
            source_columns: vec![
                ColumnRef::new(source_table_ref.clone(), "a".into(), ColumnType::BigInt),
                ColumnRef::new(source_table_ref.clone(), "b".into(), ColumnType::VarChar),
                ColumnRef::new(source_table_ref, "c".into(), ColumnType::Boolean),
            ],
            permutation,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_ok());
    }

    #[test]
    fn we_can_do_permutation_check_when_tables_have_no_rows() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [0_i64; 0], &alloc),
            borrowed_varchar("b", [""; 0], &alloc),
            borrowed_boolean("c", [true; 0], &alloc),
            borrowed_bigint("d", [0_i64; 0], &alloc),
        ]);
        let source_table_ref = TableRef::new("sxt", "source_table");
        let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        let plan = PermutationCheckTestPlan {
            source_table: source_table_ref.clone(),
            source_columns: vec![
                ColumnRef::new(source_table_ref.clone(), "a".into(), ColumnType::BigInt),
                ColumnRef::new(source_table_ref.clone(), "b".into(), ColumnType::VarChar),
                ColumnRef::new(source_table_ref, "c".into(), ColumnType::Boolean),
            ],
            permutation: vec![],
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_ok());
    }

    #[test]
    #[should_panic(expected = "The number of columns should be greater than 0")]
    fn we_cannot_do_permutation_check_if_there_are_no_columns_in_the_tables() {
        let source_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(5) },
        )
        .unwrap();
        let source_table_ref = TableRef::new("sxt", "source_table");
        let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        let plan = PermutationCheckTestPlan {
            source_table: source_table_ref,
            source_columns: vec![],
            permutation: vec![0, 1, 2, 3, 4],
        };
        let _verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
    }

    #[test]
    #[should_panic(expected = "The number of columns should be greater than 0")]
    fn we_cannot_do_permutation_check_if_there_are_neither_rows_nor_columns_in_the_tables() {
        let source_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(0) },
        )
        .unwrap();
        let source_table_ref = TableRef::new("sxt", "source_table");
        let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        let plan = PermutationCheckTestPlan {
            source_table: source_table_ref,
            source_columns: vec![],
            permutation: vec![],
        };
        let _verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
    }

    #[test]
    #[should_panic(expected = "The number of columns should be greater than 0")]
    fn we_cannot_do_permutation_check_if_no_column_is_selected() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2], &alloc),
            borrowed_bigint("b", [3, 4], &alloc),
        ]);
        let source_table_ref = TableRef::new("sxt", "source_table");
        let accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        let plan = PermutationCheckTestPlan {
            source_table: source_table_ref,
            source_columns: vec![],
            permutation: vec![],
        };
        let _verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
    }
}
