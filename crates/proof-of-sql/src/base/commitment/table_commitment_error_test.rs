use crate::base::commitment::{NegativeRange, MixedLengthColumns, TableCommitmentFromColumnsError, AppendTableCommitmentError, TableCommitmentArithmeticError};

#[test]
fn negative_range_error_display() {
    let err = NegativeRange;
    assert_eq!(format!("{}"), "cannot create a TableCommitment with a negative range");
}

#[test]
fn mixed_length_columns_error_display() {
    let err = MixedLengthColumns;
    assert_eq!(format!("{}"), "cannot create a TableCommitment from columns of mixed length");
}

#[test]
fn table_commitment_error_debug_formatting() {
    let err = NegativeRange;
    assert!(format!("{:?}").contains("NegativeRange"));
    
    let err2 = MixedLengthColumns;
    assert!(format!("{:?}").contains("MixedLengthColumns"));
}

#[test]
fn table_commitment_error_equality() {
    let err1 = NegativeRange;
    let err2 = NegativeRange;
    assert_eq!(err1, err2);
    
    let err3 = MixedLengthColumns;
    assert_ne!(err1, err3);
}

#[test]
fn table_commitment_arithmetic_error_noncontiguous() {
    let err = TableCommitmentArithmeticError::NonContiguous;
    assert_eq!(format!("{}"), "cannot perform table commitment arithmetic for noncontiguous table commitments");
}

#[test]
fn table_commitment_arithmetic_error_debug() {
    let err = TableCommitmentArithmeticError::NonContiguous;
    assert!(format!("{:?}").contains("NonContiguous"));
}
