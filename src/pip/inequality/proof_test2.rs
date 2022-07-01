use crate::base::proof::{Commitment, PIPProof, Transcript};
use crate::pip::inequality::InequalityProof;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_inequality() {
    let a = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
        Scalar::from(2_u32),
    ];
    let b = vec![
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
        Scalar::from(4_u32),
    ];

    let output = vec![
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
        Scalar::from(1_u32),
    ];

    let c_a = Commitment::from(&a[..]);
    let c_b = Commitment::from(&b[..]);

    let mut transcript = Transcript::new(b"inequalitytest");
    let inequalityproof =
        InequalityProof::create(&mut transcript, &[&a, &b], &[&output], &[c_a, c_b]);

    let mut transcript = Transcript::new(b"inequalitytest");
    assert!(inequalityproof.verify(&mut transcript, &[c_a, c_b]).is_ok());
}
