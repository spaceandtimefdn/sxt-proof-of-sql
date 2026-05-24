use crate::base::commitment::{AppendColumnCommitmentsError, DuplicateIdents, NumColumnsMismatch};

#[test]
fn duplicate_idents_error_display() {
    let err = DuplicateIdents { id: "users".to_string() };
    assert_eq!(format!("{}"), "cannot create commitments with duplicate ident: users");
}

#[test]
fn duplicate_idents_debug_formatting() {
    let err = DuplicateIdents { id: "test".to_string() };
    assert!(format!("{:?}").contains("DuplicateIdents"));
}

#[test]
fn num_columns_mismatch_error_display() {
    let err = NumColumnsMismatch {
        left_columns: 5,
        right_columns: 3,
    };
    assert!(format!("{}").contains("5") && format!("{}").contains("3"));
}

#[test]
fn append_column_commitments_error_equality() {
    // NumColumnsMismatch comparison
    let err1 = NumColumnsMismatch { left_columns: 3, right_columns: 5 };
    let err2 = NumColumnsMismatch { left_columns: 3, right_columns: 5 };
    let err3 = NumColumnsMismatch { left_columns: 4, right_columns: 5 };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
