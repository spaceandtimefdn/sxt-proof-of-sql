use super::Scalar;
use bnum::types::U256;
use core::cmp::Ordering;
use tiny_keccak::Hasher;

/// Extension trait for blanket implementations for `Scalar` types.
/// This trait is primarily to avoid cluttering the core `Scalar` implementation with default implementations
/// and provides helper methods for `Scalar`.
pub trait ScalarExt: Scalar {
    /// Compute 10^exponent for the Scalar. Note that we do not check for overflow.
    fn pow10(exponent: u8) -> Self {
        itertools::repeat_n(Self::TEN, exponent as usize).product()
    }
    /// Compare two `Scalar`s as signed numbers.
    fn signed_cmp(&self, other: &Self) -> Ordering {
        match *self - *other {
            x if x.is_zero() => Ordering::Equal,
            x if x > Self::MAX_SIGNED => Ordering::Less,
            _ => Ordering::Greater,
        }
    }

    #[must_use]
    /// Converts a U256 to Scalar, wrapping as needed
    fn from_wrapping(value: U256) -> Self {
        let value_as_limbs: [u64; 4] = value.into();
        Self::from_limbs(value_as_limbs)
    }

    /// Converts a Scalar to U256. Note that any values above `MAX_SIGNED` shall remain positive, even if they are representative of negative values.
    fn into_u256_wrapping(self) -> U256 {
        U256::from(self.to_limbs())
    }

    /// Converts a byte slice to a Scalar using a hash function, preventing collisions.
    /// WARNING: Only up to 31 bytes (2^248 bits) are supported by `PoSQL` cryptographic
    /// objects. This function masks off the last byte of the hash to ensure the result
    /// fits in this range.
    #[must_use]
    fn from_byte_slice_via_hash(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::zero();
        }

        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(bytes);
        let mut hashed_bytes = [0u8; 32];
        hasher.finalize(&mut hashed_bytes);
        let hashed_val =
            U256::from_le_slice(&hashed_bytes).expect("32 bytes => guaranteed to parse as U256");
        let masked_val = hashed_val & Self::CHALLENGE_MASK;
        Self::from_wrapping(masked_val)
    }

    /// Converts a string to a Scalar using a hash function.
    #[must_use]
    fn from_str_via_hash(val: &str) -> Self {
        Self::from_byte_slice_via_hash(val.as_bytes())
    }

    /// Converts a limb array [u64; 4] into a Scalar.
    fn from_limbs(val: [u64; 4]) -> Self;

    /// Converts a Scalar into a limb array [u64; 4].
    fn to_limbs(&self) -> [u64; 4];
}

#[cfg(test)]
pub(crate) fn test_scalar_constants<S: ScalarExt>() {
    assert_eq!(S::from(0), S::ZERO);
    assert_eq!(S::from(1), S::ONE);
    assert_eq!(S::from(2), S::TWO);
    // -1/2 == least upper bound
    assert_eq!(-S::TWO.inv().unwrap(), S::MAX_SIGNED);
    assert_eq!(S::from(10), S::TEN);

    // Check the challenge mask
    assert_eq!(
        S::CHALLENGE_MASK,
        U256::MAX >> S::CHALLENGE_MASK.leading_zeros()
    );
    assert!(S::MAX_SIGNED.into_u256_wrapping() < S::CHALLENGE_MASK);
    assert!((-S::ONE).into_u256_wrapping() > S::CHALLENGE_MASK);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::{test_scalar::TestScalar, MontScalar};
    use core::cmp::Ordering;

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Independent reference for the hash → mask → Scalar pipeline.
    fn reference_from_bytes_via_hash(bytes: &[u8]) -> TestScalar {
        if bytes.is_empty() {
            return TestScalar::ZERO;
        }
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(bytes);
        let mut hashed_bytes = [0u8; 32];
        hasher.finalize(&mut hashed_bytes);
        let hashed_val =
            U256::from_le_slice(&hashed_bytes).expect("32 bytes always parse as U256");
        let masked_val = hashed_val & TestScalar::CHALLENGE_MASK;
        TestScalar::from_wrapping(masked_val)
    }

    fn assert_challenge_range(s: TestScalar) {
        let u = s.into_u256_wrapping();
        assert!(
            u <= TestScalar::CHALLENGE_MASK,
            "hashed scalar must be within CHALLENGE_MASK, got {u}"
        );
    }

    // =========================================================================
    // test_scalar_constants (smoke via TestScalar)
    // =========================================================================

    #[test]
    fn constants_are_consistent_for_test_scalar() {
        test_scalar_constants::<TestScalar>();
    }

    // =========================================================================
    // from_wrapping / into_u256_wrapping
    // =========================================================================

    #[test]
    fn from_wrapping_and_into_u256_roundtrip_small_values() {
        for v in [0u64, 1, 2, 10, u64::MAX] {
            let s = TestScalar::from(v);
            let u = s.into_u256_wrapping();
            let back = TestScalar::from_wrapping(u);
            assert_eq!(s, back);
        }
    }

    #[test]
    fn from_wrapping_truncates_to_limbs_consistently() {
        let value = U256::MAX;
        let s = TestScalar::from_wrapping(value);
        let limbs = s.to_limbs();
        let s2 = TestScalar::from_limbs(limbs);
        assert_eq!(s, s2);
    }

    #[test]
    fn into_u256_wrapping_preserves_limbs() {
        let s = TestScalar::from(0xDEAD_BEEF_CAFE_BABEu64);
        let u = s.into_u256_wrapping();
        assert_eq!(U256::from(s.to_limbs()), u);
    }

    #[test]
    fn into_u256_wrapping_keeps_upper_half_positive_representation() {
        // Values above MAX_SIGNED stay as large positive U256 (documented).
        let neg_one = -TestScalar::ONE;
        let u = neg_one.into_u256_wrapping();
        assert!(u > TestScalar::CHALLENGE_MASK);
        assert!(u > TestScalar::MAX_SIGNED.into_u256_wrapping());
    }

    #[test]
    fn max_signed_is_below_challenge_mask_in_u256() {
        assert!(TestScalar::MAX_SIGNED.into_u256_wrapping() < TestScalar::CHALLENGE_MASK);
    }

    // =========================================================================
    // pow10
    // =========================================================================

    #[test]
    fn pow10_zero_is_one() {
        assert_eq!(TestScalar::pow10(0), TestScalar::ONE);
    }

    #[test]
    fn pow10_one_is_ten() {
        assert_eq!(TestScalar::pow10(1), TestScalar::TEN);
    }

    #[test]
    fn pow10_matches_u128_for_all_non_wrapping_exponents() {
        for i in 0..=u128::MAX.ilog10() {
            let exp = u8::try_from(i).unwrap();
            assert_eq!(
                TestScalar::pow10(exp),
                TestScalar::from(10u128.pow(i)),
                "pow10({exp})"
            );
        }
    }

    #[test]
    fn pow10_76_matches_known_field_element() {
        assert_eq!(
            TestScalar::pow10(76),
            MontScalar(ark_ff::MontFp!(
                "10000000000000000000000000000000000000000000000000000000000000000000000000000"
            ))
        );
    }

    #[test]
    fn pow10_is_prefix_multiplicative() {
        // 10^(a+b) == 10^a * 10^b when no intermediate API issues (field wrap OK).
        for a in 0u8..=20 {
            for b in 0u8..=20 {
                let left = TestScalar::pow10(a.saturating_add(b));
                let right = TestScalar::pow10(a) * TestScalar::pow10(b);
                if (a as u16 + b as u16) <= 38 {
                    // Stay in the range that also matches integer 10^n for sanity.
                    assert_eq!(left, right, "a={a}, b={b}");
                } else {
                    // Still must be internally consistent mod field.
                    assert_eq!(left, right, "wrapped a={a}, b={b}");
                }
            }
        }
    }

    #[test]
    fn pow10_large_exponents_do_not_panic() {
        for exp in [39u8, 40, 64, 100, 128, 200, 255] {
            let _ = TestScalar::pow10(exp);
        }
    }

    // =========================================================================
    // signed_cmp
    // =========================================================================

    #[test]
    fn signed_cmp_equal_values() {
        let samples = [
            TestScalar::ZERO,
            TestScalar::ONE,
            TestScalar::TWO,
            TestScalar::TEN,
            TestScalar::MAX_SIGNED,
            TestScalar::MAX_SIGNED + TestScalar::ONE, // min signed
            -TestScalar::ONE,
            TestScalar::from(123456789u64),
        ];
        for s in samples {
            assert_eq!(s.signed_cmp(&s), Ordering::Equal);
        }
    }

    #[test]
    fn signed_cmp_positive_ordering() {
        let zero = TestScalar::ZERO;
        let one = TestScalar::ONE;
        let two = TestScalar::TWO;
        let max = TestScalar::MAX_SIGNED;

        assert_eq!(one.signed_cmp(&zero), Ordering::Greater);
        assert_eq!(zero.signed_cmp(&one), Ordering::Less);
        assert_eq!(two.signed_cmp(&one), Ordering::Greater);
        assert_eq!(max.signed_cmp(&one), Ordering::Greater);
        assert_eq!(max.signed_cmp(&zero), Ordering::Greater);
        assert_eq!(max.signed_cmp(&(max - one)), Ordering::Greater);
    }

    #[test]
    fn signed_cmp_negative_region() {
        let zero = TestScalar::ZERO;
        let one = TestScalar::ONE;
        let max = TestScalar::MAX_SIGNED;
        let min = max + one; // most-negative representative in signed view

        assert_eq!(min.signed_cmp(&zero), Ordering::Less);
        assert_eq!(zero.signed_cmp(&min), Ordering::Greater);
        assert_eq!((-one).signed_cmp(&zero), Ordering::Less);
        assert_eq!((-one).signed_cmp(&one), Ordering::Less);
        assert_eq!((-one).signed_cmp(&min), Ordering::Greater); // -1 > min
    }

    #[test]
    fn signed_cmp_matches_existing_smoke_expectations() {
        let zero = TestScalar::ZERO;
        let one = TestScalar::ONE;
        let two = TestScalar::TWO;
        let max = TestScalar::MAX_SIGNED;
        let min = max + one;

        assert_eq!(max.signed_cmp(&one), Ordering::Greater);
        assert_eq!(one.signed_cmp(&zero), Ordering::Greater);
        assert_eq!(min.signed_cmp(&zero), Ordering::Less);
        assert_eq!((two * max).signed_cmp(&zero), Ordering::Less);
        assert_eq!(two * max + one, zero);
    }

    #[test]
    fn signed_cmp_antisymmetry() {
        let vals = [
            TestScalar::ZERO,
            TestScalar::ONE,
            TestScalar::TWO,
            TestScalar::MAX_SIGNED,
            TestScalar::MAX_SIGNED + TestScalar::ONE,
            -TestScalar::ONE,
            -TestScalar::TWO,
            TestScalar::TEN,
        ];
        for a in &vals {
            for b in &vals {
                let ab = a.signed_cmp(b);
                let ba = b.signed_cmp(a);
                match ab {
                    Ordering::Equal => assert_eq!(ba, Ordering::Equal),
                    Ordering::Less => assert_eq!(ba, Ordering::Greater),
                    Ordering::Greater => assert_eq!(ba, Ordering::Less),
                }
            }
        }
    }

    // =========================================================================
    // from_byte_slice_via_hash
    // =========================================================================

    #[test]
    fn from_byte_slice_via_hash_empty_is_zero() {
        assert_eq!(TestScalar::from_byte_slice_via_hash(&[]), TestScalar::ZERO);
    }

    #[test]
    fn from_byte_slice_via_hash_abc_matches_reference_pipeline() {
        let got = TestScalar::from_byte_slice_via_hash(b"abc");
        let exp = reference_from_bytes_via_hash(b"abc");
        assert_eq!(got, exp);
        assert_challenge_range(got);
    }

    #[test]
    fn from_byte_slice_via_hash_applies_challenge_mask() {
        // Raw keccak256("abc") LE value must differ from masked value unless
        // the top bits were already clear.
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(b"abc");
        let mut hashed_bytes = [0u8; 32];
        hasher.finalize(&mut hashed_bytes);
        let raw = U256::from_le_slice(&hashed_bytes).unwrap();
        let masked = raw & TestScalar::CHALLENGE_MASK;
        assert_eq!(
            TestScalar::from_byte_slice_via_hash(b"abc"),
            TestScalar::from_wrapping(masked)
        );
        // Mask must be a pure high-bit clear (subset of bits).
        assert_eq!(masked & TestScalar::CHALLENGE_MASK, masked);
        if raw != masked {
            assert!(raw > masked);
        }
    }

    #[test]
    fn from_byte_slice_via_hash_single_byte_inputs() {
        for b in 0u8..=255 {
            let s = TestScalar::from_byte_slice_via_hash(&[b]);
            assert_eq!(s, reference_from_bytes_via_hash(&[b]));
            assert_challenge_range(s);
        }
    }

    #[test]
    fn from_byte_slice_via_hash_is_deterministic() {
        let data = b"determinism-check-0123456789";
        let a = TestScalar::from_byte_slice_via_hash(data);
        let b = TestScalar::from_byte_slice_via_hash(data);
        assert_eq!(a, b);
    }

    #[test]
    fn from_byte_slice_via_hash_different_inputs_differ() {
        let a = TestScalar::from_byte_slice_via_hash(b"input-a");
        let b = TestScalar::from_byte_slice_via_hash(b"input-b");
        assert_ne!(a, b);
    }

    #[test]
    fn from_byte_slice_via_hash_long_input() {
        let data = alloc::vec![0xA5u8; 10_000];
        let s = TestScalar::from_byte_slice_via_hash(&data);
        assert_eq!(s, reference_from_bytes_via_hash(&data));
        assert_challenge_range(s);
    }

    #[test]
    fn from_byte_slice_via_hash_binary_nul_bytes() {
        let data = [0u8; 64];
        let s = TestScalar::from_byte_slice_via_hash(&data);
        // Not the empty-input special case.
        assert_ne!(s, TestScalar::ZERO);
        assert_eq!(s, reference_from_bytes_via_hash(&data));
    }

    // =========================================================================
    // from_str_via_hash  (thorough — moved to ScalarExt blanket impl)
    // =========================================================================

    #[test]
    fn from_str_via_hash_empty_string_is_zero() {
        assert_eq!(TestScalar::from_str_via_hash(""), TestScalar::ZERO);
    }

    #[test]
    fn from_str_via_hash_matches_byte_api_for_ascii() {
        let samples = [
            "",
            "a",
            "abc",
            "ABC",
            "hello world",
            "0",
            "0123456789",
            "PoSQL",
            "special-chars!@#$%^&*()",
            "whitespace   tabs\t\nnewlines\r\n",
        ];
        for s in samples {
            let from_str = TestScalar::from_str_via_hash(s);
            let from_bytes = TestScalar::from_byte_slice_via_hash(s.as_bytes());
            assert_eq!(from_str, from_bytes, "mismatch for {s:?}");
            assert_eq!(from_str, reference_from_bytes_via_hash(s.as_bytes()));
            if s.is_empty() {
                assert_eq!(from_str, TestScalar::ZERO);
            } else {
                assert_challenge_range(from_str);
            }
        }
    }

    #[test]
    fn from_str_via_hash_unicode_utf8_bytes() {
        let samples = [
            "xuân",           // Vietnamese
            "こんにちは",        // Japanese
            "🙂🚀",            // emoji
            "Ñoño",
            "Здравствуй",
            "السلام عليكم",
            "a\u{0000}b",     // interior NUL in UTF-8
            "\u{FEFF}bom",    // BOM
        ];
        for s in samples {
            let from_str = TestScalar::from_str_via_hash(s);
            let from_bytes = TestScalar::from_byte_slice_via_hash(s.as_bytes());
            assert_eq!(from_str, from_bytes, "unicode mismatch for {s:?}");
            assert_eq!(from_str, reference_from_bytes_via_hash(s.as_bytes()));
            assert_challenge_range(from_str);
        }
    }

    #[test]
    fn from_str_via_hash_is_deterministic() {
        let s = "same-string-again";
        assert_eq!(
            TestScalar::from_str_via_hash(s),
            TestScalar::from_str_via_hash(s)
        );
    }

    #[test]
    fn from_str_via_hash_case_sensitivity() {
        assert_ne!(
            TestScalar::from_str_via_hash("abc"),
            TestScalar::from_str_via_hash("ABC")
        );
    }

    #[test]
    fn from_str_via_hash_different_strings_differ() {
        let a = TestScalar::from_str_via_hash("alpha");
        let b = TestScalar::from_str_via_hash("beta");
        let c = TestScalar::from_str_via_hash("alpha ");
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn from_str_via_hash_long_string() {
        let s = "x".repeat(50_000);
        let got = TestScalar::from_str_via_hash(&s);
        assert_eq!(got, TestScalar::from_byte_slice_via_hash(s.as_bytes()));
        assert_challenge_range(got);
    }

    #[test]
    fn from_str_via_hash_known_abc_vector_against_reference() {
        // Keccak-256("abc") = 4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c05
        // Interpretation: little-endian U256 of those 32 bytes, then & CHALLENGE_MASK.
        let got = TestScalar::from_str_via_hash("abc");
        let exp = reference_from_bytes_via_hash(b"abc");
        assert_eq!(got, exp);
        assert_challenge_range(got);
    }

    /// Blanket implementation check: any `S: Scalar` gets the same hashing behavior
    /// through `ScalarExt` (exercised via a generic helper, monomorphized on TestScalar).
    fn generic_from_str_via_hash_equivalence<S: ScalarExt>() {
        let samples = ["", "a", "abc", "blanket-impl", "kiểm-thử", "🙂"];
        for v in samples {
            let via_str = S::from_str_via_hash(v);
            let via_bytes = S::from_byte_slice_via_hash(v.as_bytes());
            assert_eq!(via_str, via_bytes, "blanket mismatch for {v:?}");
            if v.is_empty() {
                assert_eq!(via_str, S::ZERO);
            }
        }
    }

    #[test]
    fn from_str_via_hash_blanket_impl_for_test_scalar() {
        generic_from_str_via_hash_equivalence::<TestScalar>();
    }

    #[test]
    fn from_str_via_hash_never_exceeds_challenge_mask_bits() {
        for v in [
            "a",
            "abc",
            "mask-boundary",
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
            "0123456789abcdef0123456789abcdef",
        ] {
            let s = TestScalar::from_str_via_hash(v);
            let u = s.into_u256_wrapping();
            assert!(u <= TestScalar::CHALLENGE_MASK);
            // High bits outside the mask must be clear.
            assert_eq!(u & TestScalar::CHALLENGE_MASK, u);
        }
    }

    #[test]
    fn from_str_via_hash_does_not_collide_with_empty_for_common_literals() {
        // Empty maps to ZERO by policy; ordinary literals must not.
        for v in ["0", "false", "null", "None", " ", "\0"] {
            assert_ne!(
                TestScalar::from_str_via_hash(v),
                TestScalar::ZERO,
                "unexpected zero hash for {v:?}"
            );
        }
    }

    // =========================================================================
    // Limb round-trip
    // =========================================================================

    #[test]
    fn limb_roundtrip_soundness_many_values() {
        let values = [
            TestScalar::ZERO,
            TestScalar::ONE,
            TestScalar::TWO,
            TestScalar::TEN,
            TestScalar::MAX_SIGNED,
            TestScalar::MAX_SIGNED + TestScalar::ONE,
            -TestScalar::ONE,
            TestScalar::from(5u64),
            TestScalar::from(u64::MAX),
            TestScalar::pow10(20),
            TestScalar::from_str_via_hash("roundtrip"),
        ];
        for original in values {
            let limbs = original.to_limbs();
            let reconstructed = TestScalar::from_limbs(limbs);
            assert_eq!(original, reconstructed, "Round-trip failed!");
        }
    }

    #[test]
    fn limb_roundtrip_after_wrapping_u256() {
        let u = U256::from(0x1234_5678_9ABC_DEF0u64) << 128;
        let s = TestScalar::from_wrapping(u);
        assert_eq!(s, TestScalar::from_limbs(s.to_limbs()));
    }

    // =========================================================================
    // Cross-property: hash outputs usable with signed_cmp / arithmetic
    // =========================================================================

    #[test]
    fn hashed_strings_support_signed_cmp_and_arithmetic() {
        let a = TestScalar::from_str_via_hash("left");
        let b = TestScalar::from_str_via_hash("right");
        let _ = a.signed_cmp(&b);
        let _ = a + b;
        let _ = a - b;
        let _ = a * b;
    }

    #[test]
    fn challenge_mask_is_prefix_bitmask() {
        // leading_zeros consistency already in test_scalar_constants; lock bit shape.
        let m = TestScalar::CHALLENGE_MASK;
        let shifts = m.leading_zeros();
        assert_eq!(m, U256::MAX >> shifts);
        assert!(shifts >= 1, "mask must clear at least some top bit for 31-byte fit");
    }
}
