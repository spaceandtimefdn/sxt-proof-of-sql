use crate::pip::sumcheck::proof::*;

use ark_std::rc::Rc;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::CompositePolynomial;
use crate::base::polynomial::DenseMultilinearExtension;
use crate::base::proof::Transcript;

#[test]
fn test_create_verify_proof() {
    let num_vars = 1;

    // create a proof
    let mut poly = CompositePolynomial::new(num_vars);
    let a_vec: [Scalar; 2] = [Scalar::from(123u64), Scalar::from(456u64)];
    let fa = Rc::new(DenseMultilinearExtension::from_evaluations_slice(
        num_vars, &a_vec,
    ));
    poly.add_product([fa], Scalar::from(1u64));
    let mut transcript = Transcript::new(b"sumchecktest");
    let proof = SumcheckProof::create(&mut transcript, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    assert!(proof
        .verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(579u64))
        .is_ok());
}
