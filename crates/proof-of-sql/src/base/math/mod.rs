//! This module defines math utilities used in Proof of SQL.
/// Handles parsing between decimal tokens received from the lexer into native `Decimal75` Proof of SQL type.
pub mod decimal;
#[cfg(test)]
mod decimal_tests;
/// Module containing [I256] type.
pub mod i256;
mod log;
pub(crate) use log::log2_up;
mod big_decimal_ext;
pub use big_decimal_ext::BigDecimalExt;
/// Module providing permutation utilities for reordering data in proof computations.
pub(crate) mod permutation;

/// Returns whether `value` is a multiple of `rhs`, matching the standard
/// integer `is_multiple_of` zero-divisor semantics without requiring the
/// unstable library feature on this toolchain.
#[inline]
pub(crate) const fn is_multiple_of(value: usize, rhs: usize) -> bool {
    if rhs == 0 {
        value == 0
    } else {
        value % rhs == 0
    }
}

#[cfg(test)]
mod tests {
    use super::is_multiple_of;

    #[test]
    fn is_multiple_of_matches_zero_divisor_semantics() {
        assert!(is_multiple_of(0, 0));
        assert!(!is_multiple_of(1, 0));
        assert!(is_multiple_of(6, 3));
        assert!(!is_multiple_of(7, 3));
    }
}
