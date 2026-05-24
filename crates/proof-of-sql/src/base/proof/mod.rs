//! Contains the transcript protocol used to construct a proof,
//! as well as an error type which can occur when verification fails.
mod error;
pub use error::{PlaceholderError, PlaceholderResult, ProofError, ProofSizeMismatch};
#[cfg(test)]
mod error_test;

/// Contains an extension trait for `merlin::Transcript`, which is used to construct a proof.
#[cfg(any(test, feature = "blitzar"))]
mod merlin_transcript_core;
#[cfg(any(test, feature = "blitzar"))]
mod merlin_transcript_core_test;

mod transcript;
pub use transcript::Transcript;
#[cfg(test)]
mod transcript_test;

mod transcript_core;
#[cfg(test)]
mod transcript_core_test;

mod keccak256_transcript;
pub use keccak256_transcript::Keccak256Transcript;
#[cfg(test)]
mod keccak256_transcript_test;
#[cfg(test)]
mod mod_test;
