#[cfg(test)]
use super::{G1Affine, G2Affine, F};
#[cfg(test)]
use ark_std::{
    rand::{rngs::StdRng, Rng, SeedableRng},
    UniformRand,
};
#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
/// Create a random number generator for testing.
#[must_use]
pub fn test_rng() -> impl Rng {
    ark_std::test_rng()
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

#[cfg(test)]
fn seeded_G_vecs_from_seed_1() -> &'static [(Vec<G1Affine>, Vec<G2Affine>)] {
    static SEEDED_G_VECS: OnceLock<Vec<(Vec<G1Affine>, Vec<G2Affine>)>> = OnceLock::new();
    SEEDED_G_VECS
        .get_or_init(|| {
            let mut rng = test_seed_rng([1; 32]);
            (0..5).map(|nu| rand_G_vecs(nu, &mut rng)).collect()
        })
        .as_slice()
}

#[cfg(test)]
fn seeded_F_vecs_from_seed_1() -> &'static [(Vec<F>, Vec<F>)] {
    static SEEDED_F_VECS: OnceLock<Vec<(Vec<F>, Vec<F>)>> = OnceLock::new();
    SEEDED_F_VECS
        .get_or_init(|| {
            let mut rng = test_seed_rng([1; 32]);
            (0..5).map(|nu| rand_F_vecs(nu, &mut rng)).collect()
        })
        .as_slice()
}

#[test]
fn we_can_create_rand_G_vecs() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (gamma_1, gamma_2) = rand_G_vecs(nu, &mut rng);
        assert_eq!(gamma_1.len(), 1 << nu);
        assert_eq!(gamma_2.len(), 1 << nu);
    }
}

#[test]
fn we_can_create_different_rand_G_vecs_consecutively_from_the_same_rng() {
    let mut rng = test_rng();
    for nu in 0..5 {
        let (gamma_1, gamma_2) = rand_G_vecs(nu, &mut rng);
        let (gamma_1_2, gamma_2_2) = rand_G_vecs(nu, &mut rng);
        assert_ne!(gamma_1, gamma_1_2);
        assert_ne!(gamma_2, gamma_2_2);
    }
}

#[test]
fn we_can_create_the_same_rand_G_vecs_from_the_same_seed() {
    let mut same_seed_rng = test_seed_rng([1; 32]);
    for (nu, (gamma_1, gamma_2)) in seeded_G_vecs_from_seed_1().iter().enumerate() {
        let (same_gamma_1, same_gamma_2) = rand_G_vecs(nu, &mut same_seed_rng);
        assert_eq!(gamma_1, &same_gamma_1);
        assert_eq!(gamma_2, &same_gamma_2);
    }
}

#[test]
fn we_can_create_different_rand_G_vecs_from_different_seeds() {
    let mut different_seed_rng = test_seed_rng([2; 32]);
    for (nu, (gamma_1, gamma_2)) in seeded_G_vecs_from_seed_1().iter().enumerate() {
        let (different_gamma_1, different_gamma_2) = rand_G_vecs(nu, &mut different_seed_rng);
        assert_ne!(gamma_1, &different_gamma_1);
        assert_ne!(gamma_2, &different_gamma_2);
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
    let mut same_seed_rng = test_seed_rng([1; 32]);
    for (nu, (s1, s2)) in seeded_F_vecs_from_seed_1().iter().enumerate() {
        let (same_s1, same_s2) = rand_F_vecs(nu, &mut same_seed_rng);
        assert_eq!(s1, &same_s1);
        assert_eq!(s2, &same_s2);
    }
}

#[test]
fn we_can_create_different_rand_F_vecs_from_different_seeds() {
    let mut different_seed_rng = test_seed_rng([2; 32]);
    for (nu, (s1, s2)) in seeded_F_vecs_from_seed_1().iter().enumerate() {
        let (different_s1, different_s2) = rand_F_vecs(nu, &mut different_seed_rng);
        assert_ne!(s1, &different_s1);
        assert_ne!(s2, &different_s2);
    }
}
