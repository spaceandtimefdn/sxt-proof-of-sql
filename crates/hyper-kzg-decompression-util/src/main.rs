//! Decompress a stream of compressed `G1Affine` points.
//!
//! Reads a concatenation of compressed `ark_bn254::G1Affine` points from stdin, such as
//! the file produced by `powers-of-tau-util`, and writes the concatenation of the same
//! points in uncompressed form to stdout.
//!
//! The input is processed in batches of [`BATCH_BYTES`] bytes, so memory use stays
//! constant regardless of the input size.
//!
//! Usage:
//!
//! cargo run --release < <`compressed_file_path`> > <`uncompressed_file_path`>

mod decompress;
mod error;

use decompress::{decompress, COMPRESSED_SIZE};
use error::DecompressionResult;
use std::io::{self, Read, Write};

/// The number of compressed bytes read per batch. This must be a multiple of
/// [`COMPRESSED_SIZE`] so that only the final batch can contain a partial point.
const BATCH_BYTES: usize = COMPRESSED_SIZE << 20;

/// [`BATCH_BYTES`] as a `u64`. The assertion guarantees at compile time that the cast
/// cannot truncate.
const BATCH_BYTES_U64: u64 = {
    assert!(usize::BITS <= u64::BITS);
    BATCH_BYTES as u64
};

/// Read compressed `G1Affine` points from `reader` in batches of [`BATCH_BYTES`] bytes and
/// write them uncompressed to `writer`.
///
/// # Errors
///
/// This function returns an error if the input cannot be read, the output cannot be
/// written, the input length is not a multiple of the compressed point size, or any
/// point fails to deserialize or serialize. Batches before the failing one will already
/// have been written.
fn decompress_stream(mut reader: impl Read, mut writer: impl Write) -> DecompressionResult<()> {
    let mut compressed = Vec::with_capacity(BATCH_BYTES);
    loop {
        compressed.clear();
        let read = reader
            .by_ref()
            .take(BATCH_BYTES_U64)
            .read_to_end(&mut compressed)?;
        writer.write_all(&decompress(&compressed)?)?;
        if read < BATCH_BYTES {
            break;
        }
    }
    Ok(writer.flush()?)
}

fn main() -> DecompressionResult<()> {
    decompress_stream(io::stdin().lock(), io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;
    use error::DecompressionError;

    /// The compressed point at infinity: a zero x-coordinate with the infinity flag set.
    const COMPRESSED_IDENTITY: [u8; 32] = {
        let mut bytes = [0; 32];
        bytes[31] = 0x40;
        bytes
    };

    /// The uncompressed point at infinity: zero coordinates with the infinity flag set.
    const UNCOMPRESSED_IDENTITY: [u8; 64] = {
        let mut bytes = [0; 64];
        bytes[63] = 0x40;
        bytes
    };

    /// Decompress `compressed` through [`decompress_stream`] into a buffer.
    fn decompress_to_vec(compressed: &[u8]) -> DecompressionResult<Vec<u8>> {
        let mut uncompressed = Vec::new();
        decompress_stream(compressed, &mut uncompressed).map(|()| uncompressed)
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_can_decompress_a_stream() {
        let uncompressed = decompress_to_vec(&COMPRESSED_IDENTITY.repeat(3)).unwrap();
        assert_eq!(uncompressed, UNCOMPRESSED_IDENTITY.repeat(3));
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_can_decompress_an_empty_stream() {
        assert!(decompress_to_vec(&[]).unwrap().is_empty());
    }

    /// # Panics
    ///
    /// This function panics if the assertion fails.
    #[test]
    fn we_cannot_decompress_a_stream_with_invalid_length() {
        let mut compressed = COMPRESSED_IDENTITY.repeat(3);
        compressed.pop();
        assert!(matches!(
            decompress_to_vec(&compressed),
            Err(DecompressionError::InvalidLength)
        ));
    }
}
