use super::range_check::{
    final_round_evaluate_range_check, first_round_evaluate_range_check,
    verifier_evaluate_range_check,
};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, ColumnType, LiteralValue, Table, TableEvaluation, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use bumpalo::Bump;
use serde::Serialize;
use sqlparser::ast::Ident;

#[derive(Debug, Serialize)]
// A test plan for performing range checks on a specified column.
struct RangeCheckTestPlan {
    // The column reference for the range check test.
    column: ColumnRef,
}

macro_rules! handle_column_with_match {
    ($col:expr, $fn_name:ident, $builder:expr, $alloc:expr) => {
        match $col.column_type() {
            ColumnType::BigInt => {
                let slice = $col
                    .as_bigint()
                    .expect("column_type() is BigInt, but as_bigint() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Int => {
                let slice = $col
                    .as_int()
                    .expect("column_type() is Int, but as_int() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::SmallInt => {
                let slice = $col
                    .as_smallint()
                    .expect("column_type() is SmallInt, but as_smallint() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::TinyInt => {
                let slice = $col
                    .as_tinyint()
                    .expect("column_type() is TinyInt, but as_tinyint() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Uint8 => {
                let slice = $col
                    .as_uint8()
                    .expect("column_type() is Uint8, but as_uint8() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Int128 => {
                let slice = $col
                    .as_int128()
                    .expect("column_type() is Int128, but as_int128() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Decimal75(_precision, _scale) => {
                let slice = $col
                    .as_decimal75()
                    .expect("column_type() is Decimal75, but as_decimal75() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::Scalar => {
                let slice = $col
                    .as_scalar()
                    .expect("column_type() is Scalar, but as_scalar() was None");
                $fn_name($builder, slice, $alloc);
            }
            ColumnType::TimestampTZ(_tu, _tz) => {
                let slice = $col
                    .as_timestamptz()
                    .expect("column_type() is TimestampTZ, but as_timestamptz() was None");
                $fn_name($builder, slice, $alloc);
            }
            _ => {
                panic!("Unsupported column type in handle_column_with_match");
            }
        }
    };
}

impl ProverEvaluate for RangeCheckTestPlan {
    #[doc = " Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        builder.request_post_result_challenges(1);
        builder.update_range_length(256);

        let table = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");

        // Extract the column data
        let col = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table");

        handle_column_with_match!(col, first_round_evaluate_range_check, builder, alloc);

        // Return a clone of the same table
        Ok(table.clone())
    }

    // extract data to test on from here, feed it into range check
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let table = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        let col = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table");

        handle_column_with_match!(col, final_round_evaluate_range_check, builder, alloc);

        Ok(table.clone())
    }
}

impl ProofPlan for RangeCheckTestPlan {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new(
            self.column.column_id(),
            *self.column.column_type(),
        )]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column.clone()}
    }

    #[doc = " Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
    }

    #[doc = " Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        _params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let input_column_eval = accessor[&self.column.table_ref()][&self.column.column_id()];
        let chi_n_eval = chi_eval_map[&self.column.table_ref()];

        verifier_evaluate_range_check(builder, input_column_eval, chi_n_eval.0)?;

        Ok(TableEvaluation::new(
            vec![accessor[&self.column.table_ref()][&self.column.column_id()]],
            chi_eval_map[&self.column.table_ref()],
        ))
    }
}

#[cfg(test)]
mod no_blitzar_tests {
    use super::*;
    use crate::base::{
        database::table_utility::*,
        map::indexset,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, Scalar},
    };
    use alloc::{collections::VecDeque, vec};

    fn run_range_check_plan_for_column<'a>(
        alloc: &'a Bump,
        column: (Ident, Column<'a, TestScalar>),
        column_type: ColumnType,
    ) {
        let table_ref = TableRef::new("sxt", "range_table");
        let column_id = column.0.clone();
        let table = table([column]);
        let mut table_map = IndexMap::default();
        table_map.insert(table_ref.clone(), table.clone());
        let plan = RangeCheckTestPlan {
            column: ColumnRef::new(table_ref, column_id, column_type),
        };

        let mut first_round_builder = FirstRoundBuilder::new(1);
        let first_round_table = plan
            .first_round_evaluate(&mut first_round_builder, alloc, &table_map, &[])
            .unwrap();
        assert_eq!(first_round_table, table);
        assert_eq!(first_round_builder.range_length(), 256);
        assert_eq!(first_round_builder.chi_evaluation_lengths(), &[256]);
        assert_eq!(first_round_builder.pcs_proof_mles().len(), 31);

        let mut post_result_challenges = VecDeque::new();
        post_result_challenges.push_back(TestScalar::from(7u64));
        let mut final_round_builder = FinalRoundBuilder::new(8, post_result_challenges);
        let final_round_table = plan
            .final_round_evaluate(&mut final_round_builder, alloc, &table_map, &[])
            .unwrap();
        assert_eq!(final_round_table, table);
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 33);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 33);
    }

    #[test]
    fn we_can_exercise_range_check_plan_without_blitzar_for_supported_column_types() {
        let alloc = Bump::new();
        run_range_check_plan_for_column(
            &alloc,
            borrowed_uint8("uint8", [0_u8, 1, u8::MAX], &alloc),
            ColumnType::Uint8,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_tinyint("tinyint", [0_i8, 1, i8::MAX], &alloc),
            ColumnType::TinyInt,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_smallint("smallint", [0_i16, 1, i16::MAX], &alloc),
            ColumnType::SmallInt,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_int("int", [0_i32, 1, i32::MAX], &alloc),
            ColumnType::Int,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_bigint("bigint", [0_i64, 1, i64::MAX], &alloc),
            ColumnType::BigInt,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_int128("int128", [0_i128, 1, i128::MAX], &alloc),
            ColumnType::Int128,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_scalar(
                "scalar",
                [TestScalar::ZERO, TestScalar::ONE, TestScalar::from(2u64)],
                &alloc,
            ),
            ColumnType::Scalar,
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_decimal75(
                "decimal75",
                74,
                0,
                [TestScalar::ZERO, TestScalar::ONE, TestScalar::from(2u64)],
                &alloc,
            ),
            ColumnType::Decimal75(Precision::new(74).unwrap(), 0),
        );
        run_range_check_plan_for_column(
            &alloc,
            borrowed_timestamptz(
                "time",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [0_i64, 1, i64::MAX],
                &alloc,
            ),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
        );
    }

    #[test]
    fn range_check_plan_reports_metadata_without_blitzar() {
        let table_ref = TableRef::new("sxt", "range_table");
        let column = ColumnRef::new(table_ref.clone(), "value".into(), ColumnType::BigInt);
        let plan = RangeCheckTestPlan {
            column: column.clone(),
        };

        assert_eq!(
            plan.get_column_result_fields(),
            vec![ColumnField::new("value".into(), ColumnType::BigInt)]
        );
        assert_eq!(plan.get_column_references(), indexset! {column});
        assert_eq!(plan.get_table_references(), indexset! {table_ref});
    }

    #[test]
    #[should_panic(expected = "Unsupported column type in handle_column_with_match")]
    fn range_check_plan_rejects_unsupported_column_types_without_blitzar() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "range_table");
        let column = borrowed_boolean("flag", [false, true], &alloc);
        let column_id = column.0.clone();
        let table = table([column]);
        let mut table_map = IndexMap::default();
        table_map.insert(table_ref.clone(), table);
        let plan = RangeCheckTestPlan {
            column: ColumnRef::new(table_ref, column_id, ColumnType::Boolean),
        };
        let mut first_round_builder = FirstRoundBuilder::new(1);

        let _ = plan.first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[]);
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{
                owned_table_utility::*, ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor,
            },
            math::decimal::Precision,
            posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        },
        proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;
    use num_bigint::BigUint;

    fn check_range(
        table_name: TableRef,
        col_name: &str,
        col_type: ColumnType,
        accessor: &OwnedTableTestAccessor<InnerProductProof>,
    ) {
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(table_name, col_name.into(), col_type),
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&ast, accessor, &(), &[]).unwrap();
        assert!(verifiable_res.verify(&ast, accessor, &(), &[]).is_ok());
    }

    #[test]
    fn we_can_prove_ranges_on_mixed_column_types() {
        let data = owned_table([
            uint8("uint8", [0, u8::MAX]),
            tinyint("tinyint", [0, i8::MAX]),
            smallint("smallint", [0, i16::MAX]),
            int("int", [0, i32::MAX]),
            bigint("bigint", [0, i64::MAX]),
            int128("int128", [0, i128::MAX]),
            timestamptz(
                "times",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [0, i64::MAX],
            ),
            decimal75(
                "decimal75",
                74,
                0,
                [
                    Curve25519Scalar::ZERO,
                    // 2^248 - 1
                    Curve25519Scalar::from_bigint(
                        (BigUint::from(2u8).pow(248) - BigUint::from(1u8))
                            .to_u64_digits()
                            .try_into()
                            .unwrap(),
                    ),
                ],
            ),
        ]);

        let t: TableRef = "sxt.t".parse().unwrap();
        let accessor =
            OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());

        check_range(t.clone(), "uint8", ColumnType::Uint8, &accessor);
        check_range(t.clone(), "tinyint", ColumnType::TinyInt, &accessor);
        check_range(t.clone(), "smallint", ColumnType::SmallInt, &accessor);
        check_range(t.clone(), "int", ColumnType::Int, &accessor);
        check_range(t.clone(), "bigint", ColumnType::BigInt, &accessor);
        check_range(t.clone(), "int128", ColumnType::Int128, &accessor);
        check_range(
            t.clone(),
            "decimal75",
            ColumnType::Decimal75(Precision::new(74).unwrap(), 0),
            &accessor,
        );
        check_range(
            t,
            "times",
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
            &accessor,
        );
    }

    #[test]
    #[should_panic(
        expected = "Range check failed, column contains values outside of the selected range"
    )]
    fn we_cannot_successfully_verify_invalid_range() {
        let data = owned_table([scalar("a", -2..254)]);
        let t = TableRef::new("sxt", "t");
        let accessor =
            OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t.clone(), "a".into(), ColumnType::Scalar),
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &[]).unwrap();
        let _ = verifiable_res.verify(&ast, &accessor, &(), &[]);
    }

    #[test]
    fn we_can_prove_a_range_check_with_range_up_to_boundary() {
        // 2^248 - 1
        let big_uint = BigUint::from(2u8).pow(248) - BigUint::from(1u8);
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0..2u32.pow(10))
                .map(|i| upper_bound - Curve25519Scalar::from(u64::from(i))) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t = TableRef::new("sxt", "t");
        let accessor =
            OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t.clone(), "a".into(), ColumnType::Scalar),
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &[]).unwrap();
        let res: Result<
            crate::sql::proof::QueryData<crate::base::scalar::MontScalar<ark_curve25519::FrConfig>>,
            crate::sql::proof::QueryError,
        > = verifiable_res.verify(&ast, &accessor, &(), &[]);

        if let Err(e) = res {
            panic!("Verification failed: {e}");
        }
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_prove_a_range_check_with_range_below_max_word_value() {
        // 2^248 - 1
        let big_uint = BigUint::from(2u8).pow(248) - BigUint::from(1u8);
        // Parse the number into a BigUint
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0u8..1)
                .map(|i| upper_bound - Curve25519Scalar::from(i)) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t = TableRef::new("sxt", "t");
        let accessor =
            OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t.clone(), "a".into(), ColumnType::Scalar),
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &[]).unwrap();
        let res: Result<
            crate::sql::proof::QueryData<crate::base::scalar::MontScalar<ark_curve25519::FrConfig>>,
            crate::sql::proof::QueryError,
        > = verifiable_res.verify(&ast, &accessor, &(), &[]);

        if let Err(e) = res {
            panic!("Verification failed: {e}");
        }
        assert!(res.is_ok());
    }

    #[test]
    #[should_panic(
        expected = "Range check failed, column contains values outside of the selected range"
    )]
    fn we_cannot_prove_a_range_check_equal_to_range_boundary() {
        // 2^248
        let big_uint = BigUint::from(2u8).pow(248);
        let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

        // Convert Vec<u64> to [u64; 4]
        let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

        let upper_bound = Curve25519Scalar::from_bigint(limbs);

        // Generate the test data
        let data: OwnedTable<Curve25519Scalar> = owned_table([scalar(
            "a",
            (0u16..2u16.pow(10))
                .map(|i| upper_bound - Curve25519Scalar::from(i)) // Count backward from 2^248
                .collect::<Vec<_>>(),
        )]);

        let t: TableRef = "sxt.t".parse().unwrap();
        let accessor =
            OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
        let ast = RangeCheckTestPlan {
            column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
        };
        let verifiable_res =
            VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &[]).unwrap();
        verifiable_res.verify(&ast, &accessor, &(), &[]).unwrap();
    }
}
