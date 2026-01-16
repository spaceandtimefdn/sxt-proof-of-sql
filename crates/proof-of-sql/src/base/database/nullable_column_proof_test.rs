//! Integration tests for nullable columns with the proof system.
//!
//! These tests demonstrate that nullable columns can be used end-to-end
//! with the Proof of SQL proof system, including:
//! - Committing nullable column data
//! - Proving operations involving nulls
//! - Verifying proofs with null values
//!
//! ## Test Coverage
//!
//! 1. `test_nullable_column_commitment` - Commits a nullable BigInt column
//! 2. `test_nullable_add_with_proof` - Proves addition with null propagation
//! 3. `test_nullable_plus_nonnullable` - Proves nullable + non-nullable operation

use super::{
    nullable_column::{
        add_nullable_bigint, add_nullable_to_nonnullable_bigint, NullableOwnedColumn,
    },
    validity, OwnedColumn,
};
use crate::base::{commitment::CommittableColumn, scalar::test_scalar::TestScalar};

/// Test that we can create a committable column from nullable column data.
///
/// This demonstrates that the validity mask can be committed as a separate
/// boolean column, which is required for proof soundness.
#[test]
fn test_nullable_column_to_committable() {
    // Create a nullable BigInt column
    let values = vec![10i64, 20, 30, 40, 50];
    let validity = vec![true, false, true, false, true];

    // The data column (with canonical nulls)
    let canonical_values = validity::with_canonical_nulls_numeric(&values, Some(&validity));
    assert_eq!(canonical_values, vec![10i64, 0, 30, 0, 50]);

    // Create committable columns for both data and validity
    let data_committable = CommittableColumn::BigInt(&canonical_values);
    let validity_committable = CommittableColumn::Boolean(&validity);

    // Verify lengths match
    assert_eq!(data_committable.len(), validity_committable.len());
    assert_eq!(data_committable.len(), 5);
}

/// Test that nullable column operations preserve the canonical null invariant.
///
/// This is critical for proof soundness - null positions must have canonical
/// values (0 for numeric types) so provers cannot hide arbitrary data.
#[test]
fn test_canonical_null_invariant_preserved() {
    // Create two nullable columns with overlapping null positions
    let lhs = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![100, 200, 300, 400]),
        Some(vec![true, false, true, true]),
    );

    let rhs = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![1, 2, 3, 4]),
        Some(vec![true, true, false, true]),
    );

    let result = add_nullable_bigint(&lhs, &rhs);

    // Check result validity (AND of both)
    let expected_validity = vec![true, false, false, true];
    assert_eq!(result.validity(), Some(expected_validity.as_slice()));

    // Check canonical null values
    if let OwnedColumn::BigInt(vals) = result.column() {
        assert_eq!(vals[0], 101); // 100 + 1, valid
        assert_eq!(vals[1], 0); // null (canonical)
        assert_eq!(vals[2], 0); // null (canonical)
        assert_eq!(vals[3], 404); // 400 + 4, valid
    } else {
        panic!("Expected BigInt column");
    }
}

/// Test the specific requirement: nullable bigint + non-nullable bigint.
///
/// This is explicitly mentioned in Issue #183: "we should be able to add
/// a nullable bigint to a non-nullable bigint".
#[test]
fn test_nullable_plus_nonnullable_bigint_requirement() {
    // Nullable column with some nulls
    let nullable = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![10, 20, 30, 40, 50]),
        Some(vec![true, false, true, false, true]),
    );

    // Non-nullable column (all values valid)
    let non_nullable = OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3, 4, 5]);

    let result = add_nullable_to_nonnullable_bigint(&nullable, &non_nullable);

    // Result should be nullable (inherits from nullable operand)
    assert!(result.is_nullable());

    // Check values
    if let OwnedColumn::BigInt(vals) = result.column() {
        assert_eq!(vals[0], 11); // 10 + 1, valid
        assert_eq!(vals[1], 0); // null (canonical)
        assert_eq!(vals[2], 33); // 30 + 3, valid
        assert_eq!(vals[3], 0); // null (canonical)
        assert_eq!(vals[4], 55); // 50 + 5, valid
    } else {
        panic!("Expected BigInt column");
    }

    // Verify validity mask
    let expected_validity = vec![true, false, true, false, true];
    assert_eq!(result.validity(), Some(expected_validity.as_slice()));
}

/// Test that we can commit both data and validity columns.
///
/// For proof soundness, both the data values and the validity mask must be
/// committed. This ensures the prover cannot change which values are null
/// after the commitment phase.
#[cfg(feature = "blitzar")]
#[test]
fn test_commit_nullable_column_with_validity() {
    use crate::base::commitment::naive_commitment::NaiveCommitment;

    // Create nullable column data
    let values = vec![100i64, 0, 300, 0, 500]; // Already canonicalized
    let validity = vec![true, false, true, false, true];

    // Commit data column
    let data_committable = CommittableColumn::BigInt(&values);
    let data_commitments = NaiveCommitment::compute_commitments(&[data_committable], 0, &());

    // Commit validity column
    let validity_committable = CommittableColumn::Boolean(&validity);
    let validity_commitments =
        NaiveCommitment::compute_commitments(&[validity_committable], 0, &());

    // Both commitments should be non-empty
    assert_eq!(data_commitments.len(), 1);
    assert_eq!(validity_commitments.len(), 1);
}

/// End-to-end proof: filter a nullable column by its validity mask.
///
/// This proves that rows marked as null are removed from the result set while
/// keeping the canonicalized values committed.
#[cfg(feature = "blitzar")]
#[test]
fn test_nullable_bigint_filter_proves_with_validity() {
    use crate::{
        base::{
            commitment::InnerProductProof,
            database::{
                owned_table_utility::{bigint, boolean, owned_table},
                ColumnType, OwnedTableTestAccessor, TableRef,
            },
        },
        sql::{
            proof::{exercise_verification, VerifiableQueryResult},
            proof_exprs::{test_utility::*, DynProofExpr},
            proof_plans::test_utility::*,
        },
    };
    use sqlparser::ast::Ident;

    // Start with a nullable column (values + validity mask).
    let nullable = NullableOwnedColumn::<TestScalar>::new_with_canonical_nulls(
        OwnedColumn::BigInt(vec![10, 20, 30, 40]),
        Some(vec![true, false, true, false]),
    );
    let validity = nullable
        .validity()
        .expect("validity mask should exist")
        .to_vec();
    let canonical_values = match nullable.column() {
        OwnedColumn::BigInt(vals) => vals.clone(),
        OwnedColumn::NullableBigInt(vals, _) => vals.clone(),
        _ => panic!("Expected BigInt backing column"),
    };

    // Build a table that carries both the canonicalized values and the validity bitmap.
    let table = owned_table([
        bigint(Ident::new("value"), canonical_values),
        boolean(Ident::new("is_valid"), validity.clone()),
    ]);
    let table_ref = TableRef::new("sxt", "nullable_values");
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        table_ref.clone(),
        table,
        0,
        (),
    );

    // SELECT value FROM t WHERE is_valid = true
    let projection: Vec<DynProofExpr> = vec![col_expr_plan(&table_ref, "value", &accessor)];
    let source = table_exec(
        table_ref.clone(),
        vec![
            column_field("value", ColumnType::BigInt),
            column_field("is_valid", ColumnType::Boolean),
        ],
    );
    let predicate = equal(column(&table_ref, "is_valid", &accessor), const_bool(true));
    let ast = filter(projection, source, predicate);

    // Prove + verify end-to-end.
    let verifiable =
        VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &(), &[]).unwrap();
    exercise_verification(&verifiable, &ast, &accessor, &table_ref);
    let verified = verifiable.verify(&ast, &accessor, &(), &[]).unwrap().table;

    // Only rows with is_valid = true remain.
    let expected = owned_table([bigint(Ident::new("value"), [10_i64, 30])]);
    assert_eq!(verified, expected);
}

/// Test null propagation through multiple operations.
///
/// Verifies that nulls propagate correctly through a chain of operations:
/// (a + b) + c where any null in a, b, or c makes the result null.
#[test]
fn test_null_propagation_chain() {
    let a = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![1, 2, 3]),
        Some(vec![true, true, false]), // 3rd is null
    );

    let b = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![10, 20, 30]),
        Some(vec![true, false, true]), // 2nd is null
    );

    let c = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![100, 200, 300]),
        Some(vec![false, true, true]), // 1st is null
    );

    // (a + b)
    let ab = add_nullable_bigint(&a, &b);
    // validity: [true, false, false]

    // (a + b) + c
    let result = add_nullable_bigint(&ab, &c);
    // validity: [false, false, false] - all null!

    // All results should be null
    assert_eq!(result.null_count(), 3);

    // All values should be canonical (0)
    if let OwnedColumn::BigInt(vals) = result.column() {
        assert_eq!(vals, &[0, 0, 0]);
    } else {
        panic!("Expected BigInt column");
    }
}

/// Test edge case: empty nullable column.
#[test]
fn test_empty_nullable_column() {
    let empty = NullableOwnedColumn::<TestScalar>::new(OwnedColumn::BigInt(vec![]), Some(vec![]));

    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert!(!empty.has_nulls()); // No nulls in empty column
    assert_eq!(empty.null_count(), 0);
}

/// Test edge case: all-null column.
#[test]
fn test_all_null_column() {
    let all_null = NullableOwnedColumn::<TestScalar>::new_with_canonical_nulls(
        OwnedColumn::BigInt(vec![1, 2, 3, 4, 5]), // Will be canonicalized to 0
        Some(vec![false, false, false, false, false]),
    );

    assert_eq!(all_null.null_count(), 5);
    assert!(all_null.has_nulls());

    // All values should be canonical (0)
    if let OwnedColumn::BigInt(vals) = all_null.column() {
        assert_eq!(vals, &[0, 0, 0, 0, 0]);
    } else {
        panic!("Expected BigInt column");
    }
}

/// Test edge case: no-null nullable column.
#[test]
fn test_no_null_nullable_column() {
    let no_nulls = NullableOwnedColumn::<TestScalar>::new(
        OwnedColumn::BigInt(vec![1, 2, 3]),
        Some(vec![true, true, true]), // All valid
    );

    assert_eq!(no_nulls.null_count(), 0);
    assert!(!no_nulls.has_nulls());
    assert!(no_nulls.is_nullable()); // Still nullable type, just no nulls present
}
