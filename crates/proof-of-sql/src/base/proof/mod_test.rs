#[cfg(test)]
mod proof_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::proof::{Keccak256Transcript, ProofError};
        let _ = Keccak256Transcript::default();
    }
}
