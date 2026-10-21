//! Errors for the decompression utility.

use crate::decompress::COMPRESSED_SIZE;
use ark_serialize::SerializationError;
use std::io::Error;
use thiserror::Error;

/// Errors that can occur while decompressing a stream of points.
#[derive(Error, Debug)]
pub(crate) enum DecompressionError {
    /// The input length is not a multiple of the compressed point size.
    #[error("Input length is not a multiple of the compressed point size {COMPRESSED_SIZE}")]
    InvalidLength,

    /// A point could not be decompressed or serialized in uncompressed form.
    #[error("Failed to decompress point")]
    Serialization(#[from] SerializationError),

    /// The input could not be read or the output could not be written.
    #[error("Failed to read input or write output")]
    Io(#[from] Error),
}

/// The result type for the decompression utility.
pub(crate) type DecompressionResult<T> = Result<T, DecompressionError>;
