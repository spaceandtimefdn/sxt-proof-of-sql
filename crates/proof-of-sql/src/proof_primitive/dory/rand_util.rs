#[cfg(test)]
use super::{G1Affine, G2Affine, PublicParameters, F};
#[cfg(test)]
use ark_std::{
    rand::{rngs::StdRng, Rng, SeedableRng},
    UniformRand,
};
#[cfg(test)]
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

#[cfg(test)]
/// Create a random number generator for testing.
#[must_use]
pub fn test_rng() -> impl Rng {
    ark_std::test_rng()
}

#[cfg(test)]
static SHARED_TEST_PUBLIC_PARAMETERS: OnceLock<Mutex<HashMap<usize, &'static PublicParameters>>> =
    OnceLock::new();

/// Returns a process-shared, lazily-initialized [`PublicParameters`] for the given
/// `max_nu`, generated with [`test_rng`]. The output is bit-identical to
/// `PublicParameters::test_rand(max_nu, &mut test_rng())` (because [`test_rng`] is a
/// deterministic seeded RNG), but is constructed at most once per `max_nu` and then
/// reused across every test in the process. `PublicParameters::test_rand` for the
/// same handful of `max_nu` values is currently called dozens of times per run and
/// dominates the test suite's wall time; this collapses those into a single
/// construction per `max_nu`.
///
/// The returned reference has a `'static` lifetime, suitable for constructing a
/// `ProverSetup<'static>` and a `VerifierSetup`. The underlying [`PublicParameters`]
/// is intentionally leaked, but only once per `max_nu` per process lifetime.
///
/// # Panics
/// Panics only if the internal cache mutex is poisoned (i.e., another thread
/// panicked while holding it), which would only happen on a prior test panic.
#[cfg(test)]
#[must_use]
pub fn shared_test_public_parameters(max_nu: usize) -> &'static PublicParameters {
    let cache = SHARED_TEST_PUBLIC_PARAMETERS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache
        .lock()
        .expect("shared_test_public_parameters cache poisoned");
    *guard.entry(max_nu).or_insert_with(|| {
        Box::leak(Box::new(PublicParameters::test_rand(
            max_nu,
            &mut test_rng(),
        )))
    })
}

/// Create a random number generator for testing with a specific seed.
#[cfg(test)]
pub fn test_seed_rng(seed: [u8; 32]) -> impl Rng {
    StdRng::from_seed(seed)
}

#[cfg(test)]
/// Creates two vectors of random G1 and G2 elements with length 2^nu.
pub fn rand_G_vecs<R>(nu: usize, rng: &mut R) -> (Vec<G1Affine>, Vec<G2Affine>)
where
    R: ark_std::rand::Rng + ?Sized,
{
    core::iter::repeat_with(|| (G1Affine::rand(rng), G2Affine::rand(rng)))
        .take(1 << nu)
        .unzip()
}

/// Creates two vectors of random F elements with length 2^nu.
#[cfg(test)]
pub fn rand_F_vecs<R>(nu: usize, rng: &mut R) -> (Vec<F>, Vec<F>)
where
    R: ark_std::rand::Rng + ?Sized,
{
    core::iter::repeat_with(|| (F::rand(rng), F::rand(rng)))
        .take(1 << nu)
        .unzip()
}

/// Creates two vectors of random F elements with length 2^nu.
#[cfg(test)]
pub fn rand_F_tensors<R>(nu: usize, rng: &mut R) -> (Vec<F>, Vec<F>)
where
    R: ark_std::rand::Rng + ?Sized,
{
    core::iter::repeat_with(|| (F::rand(rng), F::rand(rng)))
        .take(nu)
        .unzip()
}

#[test]
fn we_can_create_rand_G_vecs() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        assert_eq!(Gamma_1.len(), 1 << nu);
        assert_eq!(Gamma_2.len(), 1 << nu);
    }
}

#[test]
fn we_can_create_different_rand_G_vecs_consecutively_from_the_same_rng() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_2, Gamma_2_2) = rand_G_vecs(nu, &mut rng);
        assert_ne!(Gamma_1, Gamma_1_2);
        assert_ne!(Gamma_2, Gamma_2_2);
    }
}

#[test]
fn we_can_create_the_same_rand_G_vecs_from_the_same_seed() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([1; 32]);
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_2, Gamma_2_2) = rand_G_vecs(nu, &mut rng_2);
        assert_eq!(Gamma_1, Gamma_1_2);
        assert_eq!(Gamma_2, Gamma_2_2);
    }
}

#[test]
fn we_can_create_different_rand_G_vecs_from_different_seeds() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([2; 32]);
    for nu in 0..5 {
        let (Gamma_1, Gamma_2) = rand_G_vecs(nu, &mut rng);
        let (Gamma_1_2, Gamma_2_2) = rand_G_vecs(nu, &mut rng_2);
        assert_ne!(Gamma_1, Gamma_1_2);
        assert_ne!(Gamma_2, Gamma_2_2);
    }
}

#[test]
fn we_can_create_rand_F_vecs() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        assert_eq!(s1.len(), 1 << nu);
        assert_eq!(s2.len(), 1 << nu);
        assert_ne!(s1, s2);
    }
}

#[test]
fn we_can_create_different_rand_F_vecs_consecutively_from_the_same_rng() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (s1_2, s2_2) = rand_F_vecs(nu, &mut rng);
        assert_ne!(s1, s1_2);
        assert_ne!(s2, s2_2);
    }
}

#[test]
fn we_can_create_the_same_rand_F_vecs_from_the_same_seed() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([1; 32]);
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (s1_2, s2_2) = rand_F_vecs(nu, &mut rng_2);
        assert_eq!(s1, s1_2);
        assert_eq!(s2, s2_2);
    }
}

#[test]
fn we_can_create_different_rand_F_vecs_from_different_seeds() {
    let mut rng = test_seed_rng([1; 32]);
    let mut rng_2 = test_seed_rng([2; 32]);
    for nu in 0..5 {
        let (s1, s2) = rand_F_vecs(nu, &mut rng);
        let (s1_2, s2_2) = rand_F_vecs(nu, &mut rng_2);
        assert_ne!(s1, s1_2);
        assert_ne!(s2, s2_2);
    }
}
