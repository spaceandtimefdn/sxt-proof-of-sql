use crate::base::commitment::{ColumnCommitmentsMismatch, ColumnCommitmentMetadataMismatch};

#[test]
fn column_commitments_mismatch_display_messages() {
    // NumColumns
    let err = ColumnCommitmentsMismatch::NumColumns;
    assert_eq!(format!("{}"), "commitments with different column counts cannot operate with each other");
    
    // Ident
    let err = ColumnCommitmentsMismatch::Ident {
        id_a: "col_a".to_string(),
        id_b: "col_b".to_string(),
    };
    assert_eq!(format!("{}"), "column with ident col_a cannot operate with column with ident col_b");
}

#[test]
fn column_commitments_mismatch_debug_formatting() {
    let err = ColumnCommitmentsMismatch::NumColumns;
    assert!(format!("{:?}").contains("NumColumns"));
    
    let err2 = ColumnCommitmentsMismatch::Ident {
        id_a: "a".to_string(),
        id_b: "b".to_string(),
    };
    assert!(format!("{:?}").contains("Ident"));
}

#[test]
fn column_commitments_mismatch_equality() {
    let err1 = ColumnCommitmentsMismatch::NumColumns;
    let err2 = ColumnCommitmentsMismatch::NumColumns;
    let err3 = ColumnCommitmentsMismatch::Ident {
        id_a: "x".to_string(),
        id_b: "y".to_string(),
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
