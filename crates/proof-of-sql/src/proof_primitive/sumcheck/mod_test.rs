#[cfg(test)]
mod sumcheck_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::proof_primitive::sumcheck::{SumcheckProof, ProverState};
        let _: Option<&SumcheckProof<blitzar::compute::Curve25519Scalar>> = None;
    }
}
