#![expect(clippy::module_inception)]

use crate::base::scalar::ScalarConversionError;
use alloc::string::String;
use bnum::types::U256;
use core::ops::Sub;
use num_bigint::BigInt;
use tiny_keccak::Hasher;

/// A trait for the scalar field used in Proof of SQL.
pub trait Scalar:
    Clone
    + core::fmt::Debug
    + core::fmt::Display
    + PartialEq
    + Default
    + Sync
    + Send
    + num_traits::One
    + core::iter::Sum
    + core::iter::Product
    + Sub<Output = Self>
    + Copy
    + core::ops::MulAssign
    + core::ops::AddAssign
    + num_traits::Zero
    + for<'a> core::convert::From<&'a Self> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a bool> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i8> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i16> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i32> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i64> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a i128> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a u8> // Required for `Column` to implement `MultilinearExtension`
    + for<'a> core::convert::From<&'a u64> // Required for `Column` to implement `MultilinearExtension`
    + core::convert::TryInto<bool>
    + core::convert::TryInto<u8>
    + core::convert::TryInto<i8>
    + core::convert::TryInto<i16>
    + core::convert::TryInto<i32>
    + core::convert::TryInto<i64>
    + core::convert::TryInto<i128>
    + core::convert::From<u8>
    + core::cmp::Ord
    + core::ops::Neg<Output = Self>
    + num_traits::Zero
    + core::ops::AddAssign
    + ark_serialize::CanonicalSerialize //This enables us to put `Scalar`s on the transcript
    + ark_std::UniformRand //This enables us to get `Scalar`s as challenges from the transcript
    + num_traits::Inv<Output = Option<Self>> // Note: `inv` should return `None` exactly when the element is zero.
    + core::ops::SubAssign
    + core::convert::From<i128>
    + core::convert::From<i64>
    + core::convert::From<i32>
    + core::convert::From<i16>
    + core::convert::From<i8>
    + core::convert::From<u64>
    + core::convert::From<bool>
    + core::convert::Into<BigInt>
    + TryFrom<BigInt, Error = ScalarConversionError>
{
    /// The value (p - 1) / 2. This is "mid-point" of the field - the "six" on the clock.
    /// It is the largest signed value that can be represented in the field with the natural embedding.
    const MAX_SIGNED: Self;
    /// The 0 (additive identity) element of the field.
    const ZERO: Self;
    /// The 1 (multiplicative identity) element of the field.
    const ONE: Self;
    /// 1 + 1
    const TWO: Self;
    /// 2 + 2 + 2 + 2 + 2
    const TEN: Self;
    /// 2^64
    const TWO_POW_64: Self;
    /// The value to mask the challenge with to ensure it is in the field.
    /// This one less than the largest power of 2 that is less than the field modulus.
    const CHALLENGE_MASK: U256;
    /// The largest n such that 2^n <=p
    const MAX_BITS: u8;
    /// A U256 representation of the largest signed value in the field.
    const MAX_SIGNED_U256: U256;

    /// Create a new Scalar from raw limbs [u64; 4]. The array is expected to be in non-montgomery form.
    fn from_limbs(val: [u64; 4]) -> Self;

    /// Convert this Scalar to raw limbs [u64; 4]. The array will be in non-montgomery form.
    fn to_limbs(&self) -> [u64; 4];

    /// Convert a string slice to a Scalar using a hash function.
    #[must_use]
    fn from_str_via_hash(val: &str) -> Self {
        if val.is_empty() {
            return Self::ZERO;
        }

        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(val.as_bytes());
        let mut hashed_bytes = [0u8; 32];
        hasher.finalize(&mut hashed_bytes);
        let hashed_val =
            U256::from_le_slice(&hashed_bytes).expect("32 bytes => guaranteed to parse as U256");
        let masked_val = hashed_val & Self::CHALLENGE_MASK;
        Self::from_limbs(masked_val.into())
    }
}
