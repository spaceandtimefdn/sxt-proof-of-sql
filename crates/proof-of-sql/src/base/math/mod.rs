//! This module defines math utilities used in Proof of SQL.
/// Handles parsing between decimal tokens received from the lexer into native `Decimal75` Proof of SQL type.
pub mod decimal;
#[cfg(test)]
mod decimal_error_test;
#[cfg(test)]
mod decimal_tests;
/// Module containing [I256] type.
pub mod i256;
#[cfg(test)]
mod i256_test;
mod log;
pub(crate) use log::log2_up;
mod big_decimal_ext;
pub use big_decimal_ext::BigDecimalExt;
/// Module providing permutation utilities for reordering data in proof computations.
pub(crate) mod permutation;
#[cfg(test)]
mod permutation_error_test;
