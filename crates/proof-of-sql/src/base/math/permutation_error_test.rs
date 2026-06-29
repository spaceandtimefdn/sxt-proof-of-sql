use super::PermutationError;

#[test]
fn permutation_error_display_messages() {
    // InvalidPermutation
    let err = PermutationError::InvalidPermutation {
        error: "duplicate elements".to_string(),
    };
    assert_eq!(format!("{}"), "Permutation is invalid duplicate elements");
    
    // PermutationSizeMismatch
    let err = PermutationError::PermutationSizeMismatch {
        permutation_size: 5,
        slice_length: 3,
    };
    assert_eq!(format!("{}"), "Application of a permutation to a slice with a different length 5 != 3");
}

#[test]
fn permutation_error_debug_formatting() {
    let err = PermutationError::InvalidPermutation {
        error: "test error".to_string(),
    };
    assert!(format!("{:?}").contains("InvalidPermutation"));
    
    let err2 = PermutationError::PermutationSizeMismatch {
        permutation_size: 10,
        slice_length: 20,
    };
    assert!(format!("{:?}").contains("PermutationSizeMismatch"));
}

#[test]
fn permutation_error_equality() {
    let err1 = PermutationError::InvalidPermutation {
        error: "same".to_string(),
    };
    let err2 = PermutationError::InvalidPermutation {
        error: "same".to_string(),
    };
    let err3 = PermutationError::InvalidPermutation {
        error: "different".to_string(),
    };
    let err4 = PermutationError::PermutationSizeMismatch {
        permutation_size: 5,
        slice_length: 5,
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
    assert_ne!(err1, err4);
}
