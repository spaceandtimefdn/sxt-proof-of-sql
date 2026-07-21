#![expect(clippy::module_inception)]

use crate::base::scalar::ScalarConversionError;
use alloc::string::String;
use bnum::types::U256;
use core::ops::Sub;
use num_bigint::BigInt;

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
    + core::convert::TryInto <bool>
    + core::convert::TryInto <u8>
    + core::convert::TryInto <i8>
    + core::convert::TryInto <i16>
    + core::convert::TryInto <i32>
    + core::convert::TryInto <i64>
    + core::convert::TryInto <i128>
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

    /// Converts a `[u64; 4]` array (in non-Montgomery form) to a `Scalar`.
    ///
    /// This is a base conversion used for many other conversions. A `Scalar` is a field element
    /// (number mod some prime) with slightly less than 256 bits. Usually, `Scalar`s are internally
    /// stored in Montgomery form, so this conversion is non-free.
    fn from_limbs(val: [u64; 4]) -> Self;

    /// Converts a `Scalar` to a `[u64; 4]` array (in non-Montgomery form).
    ///
    /// This is a base conversion used for many other conversions. A `Scalar` is a field element
    /// (number mod some prime) with slightly less than 256 bits. Usually, `Scalar`s are internally
    /// stored in Montgomery form, so this conversion is non-free.
    fn to_limbs(&self) -> [u64; 4];

    /// Converts a string to a `Scalar` using a hash function.
    ///
    /// This conversion hashes the string in order to convert it to 256 bits which are ultimately
    /// converted into the `Scalar` itself. This is different from a parsing method.
    ///
    /// # Default Implementation
    ///
    /// The default implementation uses [`ScalarExt::from_byte_slice_via_hash`] to convert the string
    /// bytes to a scalar.
    fn from_str_via_hash(val: &str) -> Self
    where
        Self: Sized,
    {
        use crate::base::scalar::ScalarExt;
        Self::from_byte_slice_via_hash(val.as_bytes())
    }
}
