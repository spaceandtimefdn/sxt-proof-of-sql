pub struct PublicParameters { pub n: usize }
impl PublicParameters {
    pub fn test_rand(n: usize) -> Self { PublicParameters { n } }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProverSetup { pub n: usize }
impl ProverSetup {
    pub fn from(pp: &PublicParameters, n: usize) -> Self { ProverSetup { n: pp.n + n } }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerifierSetup { pub n: usize }
impl VerifierSetup {
    pub fn from(pp: &PublicParameters, n: usize) -> Self { VerifierSetup { n: pp.n + n } }
}
