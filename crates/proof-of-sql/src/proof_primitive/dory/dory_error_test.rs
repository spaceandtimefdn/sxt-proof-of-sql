use crate::proof_primitive::dory::DoryError;

#[test]
fn dory_error_display_messages() {
    // InvalidGeneratorsOffset
    let err = DoryError::InvalidGeneratorsOffset { offset: 5 };
    assert_eq!(format!("{}"), "invalid generators offset: 5");
    
    // VerificationError
    let err = DoryError::VerificationError;
    assert_eq!(format!("{}"), "verification error");
    
    // SmallSetup
    let err = DoryError::SmallSetup { actual: 10, required: 20 };
    assert_eq!(format!("{}"), "setup is too small: the setup is 10, but the proof requires a setup of size 20");
}

#[test]
fn dory_error_debug_formatting() {
    let err = DoryError::InvalidGeneratorsOffset { offset: 3 };
    assert!(format!("{:?}").contains("InvalidGeneratorsOffset"));
    
    let err2 = DoryError::VerificationError;
    assert!(format!("{:?}").contains("VerificationError"));
}

#[test]
fn dory_error_equality() {
    let err1 = DoryError::InvalidGeneratorsOffset { offset: 1 };
    let err2 = DoryError::InvalidGeneratorsOffset { offset: 1 };
    let err3 = DoryError::InvalidGeneratorsOffset { offset: 2 };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
