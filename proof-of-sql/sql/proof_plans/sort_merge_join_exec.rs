pub fn test_complex_query_with_two_sort_merge_joins(pp: &crate::proof_primitive::dory::PublicParameters, prover: &crate::proof_primitive::dory::ProverSetup, verifier: &crate::proof_primitive::dory::VerifierSetup) {
    assert_eq!(prover.pp(), pp);
    assert_eq!(verifier.pp(), pp);
}
