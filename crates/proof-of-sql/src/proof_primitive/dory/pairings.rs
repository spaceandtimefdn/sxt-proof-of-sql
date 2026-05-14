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
    use super::{multi_pairing, multi_pairing_2, multi_pairing_4, pairing};
    use ark_bls12_381::{Bls12_381, Fr, G1Affine, G2Affine};
    use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};

    fn g1(multiplier: u64) -> G1Affine {
        (G1Affine::generator() * Fr::from(multiplier)).into_affine()
    }

    fn g2(multiplier: u64) -> G2Affine {
        (G2Affine::generator() * Fr::from(multiplier)).into_affine()
    }

    #[test]
    fn we_can_compute_a_single_pairing() {
        let p = g1(2);
        let q = g2(3);

        assert_eq!(
            pairing::<Bls12_381>(p, q),
            <Bls12_381 as Pairing>::pairing(p, q)
        );
    }

    #[test]
    fn we_can_compute_a_multi_pairing() {
        let left = [g1(2), g1(3), g1(5)];
        let right = [g2(7), g2(11), g2(13)];

        assert_eq!(
            multi_pairing::<Bls12_381>(left, right),
            <Bls12_381 as Pairing>::multi_pairing(left, right)
        );
    }

    #[test]
    fn we_can_compute_two_multi_pairings_together() {
        let first = ([g1(2), g1(3)], [g2(5), g2(7)]);
        let second = ([g1(11), g1(13), g1(17)], [g2(19), g2(23), g2(29)]);

        assert_eq!(
            multi_pairing_2::<Bls12_381>(first, second),
            (
                <Bls12_381 as Pairing>::multi_pairing(first.0, first.1),
                <Bls12_381 as Pairing>::multi_pairing(second.0, second.1),
            )
        );
    }

    #[test]
    fn we_can_compute_four_multi_pairings_together() {
        let first = ([g1(2)], [g2(3)]);
        let second = ([g1(5), g1(7)], [g2(11), g2(13)]);
        let third = ([g1(17), g1(19), g1(23)], [g2(29), g2(31), g2(37)]);
        let fourth = ([g1(41), g1(43)], [g2(47), g2(53)]);

        assert_eq!(
            multi_pairing_4::<Bls12_381>(first, second, third, fourth),
            (
                <Bls12_381 as Pairing>::multi_pairing(first.0, first.1),
                <Bls12_381 as Pairing>::multi_pairing(second.0, second.1),
                <Bls12_381 as Pairing>::multi_pairing(third.0, third.1),
                <Bls12_381 as Pairing>::multi_pairing(fourth.0, fourth.1),
            )
        );
    }
}
