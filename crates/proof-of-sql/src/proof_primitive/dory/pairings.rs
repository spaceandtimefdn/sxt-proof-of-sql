use crate::{base::if_rayon, utils::log};
#[cfg(feature = "rayon")]
use ark_ec::pairing::MillerLoopOutput;
use ark_ec::pairing::{Pairing, PairingOutput};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_impl simply because tracing doesn't work well with threading.
pub fn pairing<P: Pairing>(
    p: impl Into<P::G1Prepared>,
    q: impl Into<P::G2Prepared>,
) -> PairingOutput<P> {
    Pairing::pairing(p, q)
}
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing<P: Pairing>(
    a: impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
    b: impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
) -> PairingOutput<P> {
    log::log_memory_usage("Start");

    multi_pairing_impl(a, b)
}
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_2_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing_2<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (PairingOutput<P>, PairingOutput<P>) {
    log::log_memory_usage("Start");

    multi_pairing_2_impl((a0, b0), (a1, b1))
}
#[tracing::instrument(level = "debug", skip_all)]
// This is a wrapper around multi_pairing_4_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing_4<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a2, b2): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a3, b3): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
) {
    log::log_memory_usage("Start");

    multi_pairing_4_impl((a0, b0), (a1, b1), (a2, b2), (a3, b3))
}
/// # Panics
/// This function may panic if the final exponentiation fails due to invalid inputs, or if the multi-pairing operation encounters an error with the provided elements.
fn multi_pairing_impl<P: Pairing>(
    a: impl IntoIterator<Item = impl Into<P::G1Prepared> + Send>,
    b: impl IntoIterator<Item = impl Into<P::G2Prepared> + Send>,
) -> PairingOutput<P> {
    if_rayon!(
        {
            let a: Vec<_> = a.into_iter().collect();
            let b: Vec<_> = b.into_iter().collect();
            Pairing::final_exponentiation(MillerLoopOutput(
                a.into_par_iter()
                    .zip(b)
                    .map(|(x, y)| P::miller_loop(x, y).0)
                    .product(),
            ))
            .unwrap()
        },
        Pairing::multi_pairing(a, b)
    )
}
fn multi_pairing_2_impl<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (PairingOutput<P>, PairingOutput<P>) {
    if_rayon!(
        rayon::join(|| multi_pairing_impl(a0, b0), || multi_pairing_impl(a1, b1)),
        (multi_pairing_impl(a0, b0), multi_pairing_impl(a1, b1))
    )
}
fn multi_pairing_4_impl<P: Pairing>(
    (a0, b0): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a1, b1): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a2, b2): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
    (a3, b3): (
        impl IntoIterator<Item = impl Into<P::G1Prepared> + Send> + Send,
        impl IntoIterator<Item = impl Into<P::G2Prepared> + Send> + Send,
    ),
) -> (
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
    PairingOutput<P>,
) {
    let ((c0, c1), (c2, c3)) = if_rayon!(
        rayon::join(
            || multi_pairing_2_impl((a0, b0), (a1, b1)),
            || multi_pairing_2_impl((a2, b2), (a3, b3)),
        ),
        (
            multi_pairing_2_impl((a0, b0), (a1, b1)),
            multi_pairing_2_impl((a2, b2), (a3, b3)),
        )
    );
    (c0, c1, c2, c3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitive::dory::{test_rng, G1Affine, G2Affine};
    use ark_bls12_381::Bls12_381;
    use ark_std::UniformRand;

    #[test]
    fn multi_pairing_matches_single_pairing_for_one_pair() {
        let mut rng = test_rng();
        let g1 = G1Affine::rand(&mut rng);
        let g2 = G2Affine::rand(&mut rng);

        assert_eq!(
            multi_pairing::<Bls12_381>([g1], [g2]),
            pairing::<Bls12_381>(g1, g2)
        );
    }

    #[test]
    fn multi_pairing_2_matches_individual_multi_pairings() {
        let mut rng = test_rng();
        let g1 = [
            G1Affine::rand(&mut rng),
            G1Affine::rand(&mut rng),
            G1Affine::rand(&mut rng),
            G1Affine::rand(&mut rng),
        ];
        let g2 = [
            G2Affine::rand(&mut rng),
            G2Affine::rand(&mut rng),
            G2Affine::rand(&mut rng),
            G2Affine::rand(&mut rng),
        ];

        let left = (g1[0..2].iter().copied(), g2[0..2].iter().copied());
        let right = (g1[2..4].iter().copied(), g2[2..4].iter().copied());

        assert_eq!(
            multi_pairing_2::<Bls12_381>(left.clone(), right.clone()),
            (
                multi_pairing::<Bls12_381>(left.0, left.1),
                multi_pairing::<Bls12_381>(right.0, right.1)
            )
        );
    }

    #[test]
    fn multi_pairing_4_matches_individual_multi_pairings() {
        let mut rng = test_rng();
        let g1 = [
            G1Affine::rand(&mut rng),
            G1Affine::rand(&mut rng),
            G1Affine::rand(&mut rng),
            G1Affine::rand(&mut rng),
        ];
        let g2 = [
            G2Affine::rand(&mut rng),
            G2Affine::rand(&mut rng),
            G2Affine::rand(&mut rng),
            G2Affine::rand(&mut rng),
        ];

        let pairings = [
            ([g1[0]], [g2[0]]),
            ([g1[1]], [g2[1]]),
            ([g1[2]], [g2[2]]),
            ([g1[3]], [g2[3]]),
        ];

        assert_eq!(
            multi_pairing_4::<Bls12_381>(
                (pairings[0].0, pairings[0].1),
                (pairings[1].0, pairings[1].1),
                (pairings[2].0, pairings[2].1),
                (pairings[3].0, pairings[3].1),
            ),
            (
                multi_pairing::<Bls12_381>(pairings[0].0, pairings[0].1),
                multi_pairing::<Bls12_381>(pairings[1].0, pairings[1].1),
                multi_pairing::<Bls12_381>(pairings[2].0, pairings[2].1),
                multi_pairing::<Bls12_381>(pairings[3].0, pairings[3].1)
            )
        );
    }
}
