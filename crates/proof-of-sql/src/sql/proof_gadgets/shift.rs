use crate::{
    base::{proof::ProofError, scalar::Scalar, slice_ops},
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_plans::{fold_columns, fold_vals},
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use num_traits::{One, Zero};

/// Perform first round evaluation of downward shift.
pub(crate) fn first_round_evaluate_shift<S: Scalar>(
    builder: &mut FirstRoundBuilder<'_, S>,
    num_rows: usize,
) {
    builder.produce_chi_evaluation_length(num_rows + 1);
    builder.produce_rho_evaluation_length(num_rows);
    builder.produce_rho_evaluation_length(num_rows + 1);
}

/// Perform final round evaluation of downward shift.
pub(crate) fn final_round_evaluate_shift<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    column: &'a [S],
) -> &'a [S] {
    let shifted_column = alloc.alloc_slice_fill_with(column.len() + 1, |i| {
        if i == 0 {
            S::ZERO
        } else {
            column[i - 1]
        }
    });
    builder.produce_intermediate_mle(shifted_column as &[_]);
    final_round_evaluate_shift_base(builder, alloc, alpha, beta, column, shifted_column);
    shifted_column
}

/// Perform final round evaluation of downward shift.
///
/// # Panics
/// Panics if `column.len() != shifted_column.len() - 1` which should always hold for shifts.
fn final_round_evaluate_shift_base<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    column: &'a [S],
    shifted_column: &'a [S],
) {
    let num_rows = column.len();
    assert_eq!(
        num_rows + 1,
        shifted_column.len(),
        "Shifted column length mismatch"
    );
    let rho_plus_chi_n =
        alloc.alloc_slice_fill_with(num_rows, |i| S::from(i as u64 + 1_u64)) as &[_];
    let rho_n_plus_1 = alloc.alloc_slice_fill_with(num_rows + 1, |i| S::from(i as u64)) as &[_];
    let chi_n_plus_1 = alloc.alloc_slice_fill_copy(num_rows + 1, true);

    let c_fold = alloc.alloc_slice_fill_copy(num_rows, Zero::zero());
    fold_columns(c_fold, alpha, beta, &[rho_plus_chi_n, column]);
    let c_fold_extended = alloc.alloc_slice_fill_copy(num_rows + 1, Zero::zero());
    c_fold_extended[..num_rows].copy_from_slice(c_fold);
    let c_star = alloc.alloc_slice_copy(c_fold_extended);
    slice_ops::add_const::<S, S>(c_star, One::one());
    slice_ops::batch_inversion(c_star);

    let d_fold = alloc.alloc_slice_fill_copy(num_rows + 1, Zero::zero());
    fold_columns(d_fold, alpha, beta, &[rho_n_plus_1, shifted_column]);
    let d_star = alloc.alloc_slice_copy(d_fold);
    slice_ops::add_const::<S, S>(d_star, One::one());
    slice_ops::batch_inversion(d_star);

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(d_star as &[_]);

    // sum c_star - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );

    // c_star + c_fold * c_star - chi_n_plus_1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(c_star as &[_])]),
            (
                S::one(),
                vec![Box::new(c_fold_extended as &[_]), Box::new(c_star as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_n_plus_1 as &[_])]),
        ],
    );

    // d_star + d_fold * d_star - chi_n_plus_1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(d_star as &[_])]),
            (
                S::one(),
                vec![Box::new(d_fold as &[_]), Box::new(d_star as &[_])],
            ),
            (-S::one(), vec![Box::new(chi_n_plus_1 as &[_])]),
        ],
    );
}

pub(crate) fn verify_shift<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    alpha: S,
    beta: S,
    column_eval: S,
    chi_n_eval: S,
) -> Result<(S, S), ProofError> {
    let chi_n_plus_1_eval = builder.try_consume_chi_evaluation()?.0;
    let shifted_column_eval = builder.try_consume_final_round_mle_evaluation()?;
    let rho_n_eval = builder.try_consume_rho_evaluation()?;
    let rho_n_plus_1_eval = builder.try_consume_rho_evaluation()?;
    let c_fold_eval = alpha * fold_vals(beta, &[rho_n_eval + chi_n_eval, column_eval]);
    let d_fold_eval = alpha * fold_vals(beta, &[rho_n_plus_1_eval, shifted_column_eval]);
    let c_star_eval = builder.try_consume_final_round_mle_evaluation()?;
    let d_star_eval = builder.try_consume_final_round_mle_evaluation()?;

    //sum c_star - d_star = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::ZeroSum,
        c_star_eval - d_star_eval,
        1,
    )?;

    // c_star + c_fold * c_star - chi_n_plus_1 = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        c_star_eval + c_fold_eval * c_star_eval - chi_n_plus_1_eval,
        2,
    )?;

    // d_star + d_fold * d_star - chi_n_plus_1 = 0
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        d_star_eval + d_fold_eval * d_star_eval - chi_n_plus_1_eval,
        2,
    )?;

    Ok((shifted_column_eval, chi_n_plus_1_eval))
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::{final_round_evaluate_shift_base, first_round_evaluate_shift, verify_shift};
    use crate::{
        base::{
            database::{
                table_utility::*, ColumnField, ColumnRef, ColumnType, LiteralValue, OwnedTable,
                Table, TableEvaluation, TableOptions, TableRef, TableTestAccessor, TestAccessor,
            },
            map::{indexset, IndexMap, IndexSet},
            proof::{PlaceholderResult, ProofError},
            scalar::Scalar,
        },
        sql::proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerifiableQueryResult,
            VerificationBuilder,
        },
    };
    use blitzar::proof::InnerProductProof;
    use bumpalo::Bump;
    use serde::Serialize;
    use sqlparser::ast::Ident;

    #[derive(Debug, Serialize)]
    pub struct ShiftTestPlan {
        pub column: ColumnRef,
        pub candidate_shifted_column: ColumnRef,
        /// The length can be wrong in the test plan and that should error out
        pub column_length: usize,
    }

    impl ProverEvaluate for ShiftTestPlan {
        #[doc = "Evaluate the query, modify `FirstRoundBuilder` and return the result."]
        fn first_round_evaluate<'a, S: Scalar>(
            &self,
            builder: &mut FirstRoundBuilder<'a, S>,
            _alloc: &'a Bump,
            _table_map: &IndexMap<TableRef, Table<'a, S>>,
            _params: &[LiteralValue],
        ) -> PlaceholderResult<Table<'a, S>> {
            builder.request_post_result_challenges(2);
            builder.produce_chi_evaluation_length(self.column_length);
            builder.produce_chi_evaluation_length(self.column_length + 1);
            // Evaluate the first round
            first_round_evaluate_shift(builder, self.column_length);
            // This is just a dummy table, the actual data is not used
            Ok(Table::try_new_with_options(
                IndexMap::default(),
                TableOptions { row_count: Some(0) },
            )
            .unwrap())
        }

        fn final_round_evaluate<'a, S: Scalar>(
            &self,
            builder: &mut FinalRoundBuilder<'a, S>,
            alloc: &'a Bump,
            table_map: &IndexMap<TableRef, Table<'a, S>>,
            _params: &[LiteralValue],
        ) -> PlaceholderResult<Table<'a, S>> {
            // Get the table from the map using the table reference
            let source_table: &Table<'a, S> = table_map
                .get(&self.column.table_ref())
                .expect("Table not found");
            let source_column: Vec<S> = source_table
                .inner_table()
                .get(&self.column.column_id())
                .expect("Column not found in table")
                .to_scalar();
            let alloc_source_column = alloc.alloc_slice_copy(&source_column);
            builder.produce_intermediate_mle(alloc_source_column as &[_]);

            let candidate_table = table_map
                .get(&self.candidate_shifted_column.table_ref())
                .expect("Table not found");
            let candidate_column: Vec<S> = candidate_table
                .inner_table()
                .get(&self.candidate_shifted_column.column_id())
                .expect("Column not found in table")
                .to_scalar();
            let alloc_candidate_column = alloc.alloc_slice_copy(&candidate_column);
            builder.produce_intermediate_mle(alloc_candidate_column as &[_]);
            let alpha = builder.consume_post_result_challenge();
            let beta = builder.consume_post_result_challenge();
            final_round_evaluate_shift_base(
                builder,
                alloc,
                alpha,
                beta,
                alloc_source_column,
                alloc_candidate_column,
            );
            // Return a dummy table
            Ok(Table::try_new_with_options(
                IndexMap::default(),
                TableOptions { row_count: Some(0) },
            )
            .unwrap())
        }
    }

    impl ProofPlan for ShiftTestPlan {
        fn get_column_result_fields(&self) -> Vec<ColumnField> {
            vec![]
        }

        fn get_column_references(&self) -> IndexSet<ColumnRef> {
            indexset! {self.column.clone(), self.candidate_shifted_column.clone()}
        }

        #[doc = "Return all the tables referenced in the Query"]
        fn get_table_references(&self) -> IndexSet<TableRef> {
            indexset! {self.column.table_ref(), self.candidate_shifted_column.table_ref()}
        }

        #[doc = "Form components needed to verify and proof store into `VerificationBuilder`"]
        fn verifier_evaluate<S: Scalar>(
            &self,
            builder: &mut impl VerificationBuilder<S>,
            _accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
            _result: Option<&OwnedTable<S>>,
            _chi_eval_map: &IndexMap<TableRef, S>,
            _params: &[LiteralValue],
        ) -> Result<TableEvaluation<S>, ProofError> {
            // Get the challenges from the builder
            let alpha = builder.try_consume_post_result_challenge()?;
            let beta = builder.try_consume_post_result_challenge()?;
            // Get the columns
            let column_eval = builder.try_consume_final_round_mle_evaluation()?;
            let chi_n_eval = builder.try_consume_chi_evaluation()?.0;
            // Evaluate the verifier
            verify_shift(builder, alpha, beta, column_eval, chi_n_eval)?;
            Ok(TableEvaluation::new(vec![], S::zero()))
        }
    }

    #[test]
    fn we_can_do_shift() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2, 3], &alloc),
            borrowed_varchar("b", ["Space", "and", "Time"], &alloc),
            borrowed_boolean("c", [true, false, true], &alloc),
        ]);
        let candidate_table = table([
            borrowed_bigint("c", [0, 1, 2, 3], &alloc),
            borrowed_varchar("d", ["", "Space", "and", "Time"], &alloc),
            borrowed_boolean("e", [false, true, false, true], &alloc),
        ]);
        let source_table_ref: TableRef = "sxt.source_table".parse().unwrap();
        let candidate_table_ref: TableRef = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref.clone(), candidate_table, 0);

        // BigInt column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref.clone(), "a".into(), ColumnType::BigInt),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref.clone(),
                "c".into(),
                ColumnType::BigInt,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        let res = verifiable_res.verify(&plan, &accessor, &(), &[]);
        assert!(res.is_ok());

        // Varchar column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref.clone(), "b".into(), ColumnType::VarChar),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref.clone(),
                "d".into(),
                ColumnType::VarChar,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        let res = verifiable_res.verify(&plan, &accessor, &(), &[]);
        assert!(res.is_ok());

        // Boolean column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref, "c".into(), ColumnType::Boolean),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref,
                "e".into(),
                ColumnType::Boolean,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        let res = verifiable_res.verify(&plan, &accessor, &(), &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn we_cannot_do_shift_if_candidate_is_incorrect() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2, 3], &alloc),
            borrowed_varchar("b", ["Space", "and", "Time"], &alloc),
            borrowed_boolean("c", [true, false, true], &alloc),
            borrowed_bigint("d", [5, 6, 7], &alloc),
        ]);
        let candidate_table = table([
            borrowed_bigint("c", [2, 1, 2, 3], &alloc),
            borrowed_varchar("d", ["The", "Space", "and", "Time"], &alloc),
            borrowed_boolean("e", [true, true, false, true], &alloc),
            borrowed_bigint("f", [0, 5, 6, 7], &alloc),
        ]);
        let source_table_ref: TableRef = "sxt.source_table".parse().unwrap();
        let candidate_table_ref: TableRef = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref.clone(), candidate_table, 0);

        // BigInt column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref.clone(), "a".into(), ColumnType::BigInt),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref.clone(),
                "c".into(),
                ColumnType::BigInt,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_err());

        // Varchar column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref.clone(), "b".into(), ColumnType::VarChar),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref.clone(),
                "d".into(),
                ColumnType::VarChar,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_err());

        // Boolean column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref.clone(), "c".into(), ColumnType::Boolean),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref.clone(),
                "e".into(),
                ColumnType::Boolean,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_err());

        // Success case: The last pair of columns is correct even though the others are not
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref, "d".into(), ColumnType::BigInt),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref,
                "f".into(),
                ColumnType::BigInt,
            ),
            column_length: 3,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&plan, &accessor, &(), &[]).is_ok());
    }

    #[should_panic(expected = "Shifted column length mismatch")]
    #[test]
    fn we_cannot_do_shift_if_column_length_is_wrong() {
        let alloc = Bump::new();
        let source_table = table([borrowed_bigint("a", [101, 102, 103, 104, 105, 106], &alloc)]);
        let candidate_table = table([borrowed_bigint(
            "a",
            [102, 101, 102, 103, 104, 105, 106, -102],
            &alloc,
        )]);
        let source_table_ref: TableRef = "sxt.source_table".parse().unwrap();
        let candidate_table_ref: TableRef = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref.clone(),
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref.clone(), candidate_table, 0);

        // BigInt column
        let plan = ShiftTestPlan {
            column: ColumnRef::new(source_table_ref, "a".into(), ColumnType::BigInt),
            candidate_shifted_column: ColumnRef::new(
                candidate_table_ref,
                "a".into(),
                ColumnType::BigInt,
            ),
            column_length: 7,
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &(), &[]).unwrap();
        let res = verifiable_res.verify(&plan, &accessor, &(), &[]);
        assert!(res.is_err());
    }
}
