//! Decompression of compressed `G1Affine` points.

use crate::error::{DecompressionError, DecompressionResult};
use ark_bn254::G1Affine;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

/// The size in bytes of a compressed `G1Affine` point.
pub(crate) const COMPRESSED_SIZE: usize = 32;

/// Decompress a single compressed `G1Affine` point into its uncompressed form.
///
/// # Errors
///
/// This function returns an error if the point fails to deserialize or serialize.
fn decompress_point(compressed: &[u8]) -> DecompressionResult<Vec<u8>> {
    let mut uncompressed = Vec::new();
    G1Affine::deserialize_compressed(compressed)?.serialize_uncompressed(&mut uncompressed)?;
    Ok(uncompressed)
}

/// Decompress a byte slice of concatenated compressed `G1Affine` points into the
/// concatenation of the same points in uncompressed form.
///
/// # Errors
///
/// This function returns an error if the length of the input is not a multiple of the
/// compressed point size or if any point fails to deserialize or serialize.
pub(crate) fn decompress(compressed: &[u8]) -> DecompressionResult<Vec<u8>> {
    let chunks = compressed
        .len()
        .is_multiple_of(COMPRESSED_SIZE)
        .then_some(compressed.par_chunks_exact(COMPRESSED_SIZE))
        .ok_or(DecompressionError::InvalidLength)?;
    let points = chunks
        .map(decompress_point)
        .collect::<DecompressionResult<Vec<_>>>()?;
    Ok(points.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::UniformRand;

    /// The size in bytes of a decompressed `G1Affine` point.
    const UNCOMPRESSED_SIZE: usize = 64;

    /// # Panics
    ///
    /// This function panics if the points cannot be generated.
    fn random_points(n: usize) -> Vec<G1Affine> {
        let mut rng = ark_std::test_rng();
        (0..n).map(|_| G1Affine::rand(&mut rng)).collect()
    }

    /// # Panics
    ///
    /// This function panics if the points cannot be serialized.
    fn compress_points(points: &[G1Affine]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for point in points {
            point.serialize_compressed(&mut bytes).unwrap();
        }
        bytes
    }

    /// # Panics
    ///
    /// This function panics if the points cannot be deserialized.
    fn deserialize_uncompressed_points(bytes: &[u8]) -> Vec<G1Affine> {
        bytes
            .chunks_exact(UNCOMPRESSED_SIZE)
            .map(|chunk| G1Affine::deserialize_uncompressed(chunk).unwrap())
            .collect()
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_can_decompress_a_point() {
        let point = random_points(1)[0];
        let uncompressed = decompress_point(&compress_points(&[point])).unwrap();
        assert_eq!(uncompressed.len(), UNCOMPRESSED_SIZE);
        assert_eq!(deserialize_uncompressed_points(&uncompressed), [point]);
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_can_decompress_the_identity_point() {
        let point = G1Affine::identity();
        let uncompressed = decompress_point(&compress_points(&[point])).unwrap();
        assert_eq!(deserialize_uncompressed_points(&uncompressed), [point]);
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_cannot_decompress_an_invalid_point() {
        assert!(matches!(
            decompress_point(&[0xff; COMPRESSED_SIZE]),
            Err(DecompressionError::Serialization(_))
        ));
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_cannot_decompress_a_truncated_point() {
        let compressed = compress_points(&random_points(1));
        assert!(matches!(
            decompress_point(&compressed[..COMPRESSED_SIZE - 1]),
            Err(DecompressionError::Serialization(_))
        ));
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_can_decompress_points() {
        let mut points = random_points(8);
        points.push(G1Affine::identity());
        let uncompressed = decompress(&compress_points(&points)).unwrap();
        assert_eq!(uncompressed.len(), points.len() * UNCOMPRESSED_SIZE);
        assert_eq!(deserialize_uncompressed_points(&uncompressed), points);
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_cannot_decompress_points_with_an_invalid_point() {
        let mut compressed = compress_points(&random_points(2));
        compressed[COMPRESSED_SIZE..].fill(0xff);
        assert!(matches!(
            decompress(&compressed),
            Err(DecompressionError::Serialization(_))
        ));
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_can_decompress_an_empty_input() {
        assert!(decompress(&[]).unwrap().is_empty());
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_cannot_decompress_input_with_invalid_length() {
        let mut compressed = compress_points(&random_points(2));
        compressed.pop();
        assert!(matches!(
            decompress(&compressed),
            Err(DecompressionError::InvalidLength)
        ));
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn point_sizes_match_the_serialized_sizes() {
        assert_eq!(COMPRESSED_SIZE, G1Affine::default().compressed_size());
        assert_eq!(UNCOMPRESSED_SIZE, G1Affine::default().uncompressed_size());
    }
}
