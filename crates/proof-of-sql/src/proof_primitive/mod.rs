//! This module contains cryptographic proof primitives used in the proof system.
//!
//! It includes commitment schemes, sumcheck protocols, and inner product arguments
//! that form the building blocks for zero-knowledge proofs.
pub mod dory;
/// Central location for any code that requires the use of a dynamic matrix (for now, hyrax and dynamic dory).
pub(super) mod dynamic_matrix_utils;
/// Module implementing the sumcheck protocol for polynomial evaluation proofs.
pub(crate) mod sumcheck;

pub mod hyperkzg;

/// Module for inner product argument proofs used in commitment verification.
pub mod inner_product;
