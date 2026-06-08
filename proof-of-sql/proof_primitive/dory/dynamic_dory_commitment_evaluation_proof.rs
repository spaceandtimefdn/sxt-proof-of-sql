pub mod tests {
    use super::super::{PublicParameters, ProverSetup, VerifierSetup};
    pub fn test_random_ipa_with_length(pp: &PublicParameters, prover: &ProverSetup, verifier: &VerifierSetup, len: usize) {
        assert_eq!(pp.0, len);
        assert_eq!(prover.pp(), pp);
        assert_eq!(verifier.pp(), pp);
    }
}
