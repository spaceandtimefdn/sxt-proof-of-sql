use super::VecCommitmentExt;
use crate::base::{
    commitment::naive_commitment::NaiveCommitment,
    scalar::test_scalar::TestScalar,
};

#[test]
fn test_from_columns_empty_produces_empty_commitments() {
    let columns: Vec<&[TestScalar]> = vec![];
    let commitments =
        Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());
    assert!(commitments.is_empty());
}

#[test]
fn test_from_columns_single_column() {
    let data: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let columns: Vec<&[TestScalar]> = vec![data.as_slice()];
    let commitments =
        Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());
    assert_eq!(commitments.len(), 1);
}

#[test]
fn test_from_columns_multiple_columns() {
    let col1: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let col2: Vec<TestScalar> = vec![TestScalar::from(3u64), TestScalar::from(4u64)];
    let columns: Vec<&[TestScalar]> = vec![col1.as_slice(), col2.as_slice()];
    let commitments =
        Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());
    assert_eq!(commitments.len(), 2);
}

#[test]
fn test_commitments_with_offset_differ() {
    let data: Vec<TestScalar> = vec![TestScalar::from(1u64)];
    let columns: Vec<&[TestScalar]> = vec![data.as_slice()];
    let c0 = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());
    let c1 = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 1, &());
    // Different offsets should produce different commitments
    assert_ne!(c0, c1);
}

#[test]
fn test_extend_with_empty_does_not_change() {
    let data: Vec<TestScalar> = vec![TestScalar::from(5u64)];
    let columns: Vec<&[TestScalar]> = vec![data.as_slice()];
    let mut commitments =
        Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());
    let original_len = commitments.len();
    let empty: Vec<&[TestScalar]> = vec![];
    commitments.extend_columns_with_offset(&empty, 0, &());
    assert_eq!(commitments.len(), original_len);
}
