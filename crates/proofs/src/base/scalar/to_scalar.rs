use crate::base::polynomial::ArkScalar;

/// Provides conversion to [Scalar].
///
/// This conversion is especially important for proofs.
/// Any data type we want to support will need to be able to convert to Scalar.
/// So, this trait may be used as a bound for supported data types.
///
/// We could just use rust's [From] and [Into] traits.
/// However, some types we want to support are foreign, and since Scalar itself is foreign, we
/// won't be able to provide these conversions.
///
/// One solution would be to create a new-type around every foreign type we want to support.
/// The other is to provide a new conversion trait entirely.
///
/// The latter was chosen for two reasons:
/// 1. We can still create new-types if we want to, but we don't have to in simple cases.
/// 2. There may be already-existing conversions for Scalar on types we *don't* want to support.
/// A new trait allows us to be explicit about the types we want to support.
pub trait ToScalar {
    fn to_scalar(&self) -> ArkScalar;
}

impl ToScalar for ArkScalar {
    fn to_scalar(&self) -> ArkScalar {
        *self
    }
}

impl ToScalar for bool {
    fn to_scalar(&self) -> ArkScalar {
        if *self {
            ArkScalar::one()
        } else {
            ArkScalar::zero()
        }
    }
}

macro_rules! uint_to_scalar {
    ($tt:ty) => {
        impl ToScalar for $tt {
            fn to_scalar(&self) -> ArkScalar {
                ArkScalar::from(*self)
            }
        }
    };
}

macro_rules! int_to_scalar {
    ($it:ty, $ut:ty) => {
        impl ToScalar for $it {
            fn to_scalar(&self) -> ArkScalar {
                if *self >= 0 {
                    ArkScalar::from(*self as $ut)
                } else {
                    -ArkScalar::from(-(*self as i128) as $ut)
                }
            }
        }
    };
}

uint_to_scalar!(u8);
uint_to_scalar!(u16);
uint_to_scalar!(u32);
uint_to_scalar!(u64);
uint_to_scalar!(u128);
int_to_scalar!(i8, u8);
int_to_scalar!(i16, u16);
int_to_scalar!(i32, u32);
int_to_scalar!(i64, u64);
int_to_scalar!(i128, u128);

macro_rules! byte_array_to_scalar {
    ($it:ty) => {
        impl ToScalar for $it {
            fn to_scalar(&self) -> ArkScalar {
                if self.is_empty() {
                    return ArkScalar::zero();
                }

                let hash = blake3::hash(self);
                let mut bytes: [u8; 32] = hash.into();
                bytes[31] &= 0b00001111_u8;

                ArkScalar::from_canonical_bytes(bytes)
                    .expect("The remaining four bits from `bytes` should be 0.")
            }
        }
    };
}

byte_array_to_scalar!([u8]);
byte_array_to_scalar!(&[u8]);

macro_rules! string_to_scalar {
    ($tt:ty) => {
        impl ToScalar for $tt {
            fn to_scalar(&self) -> ArkScalar {
                self.as_bytes().to_scalar()
            }
        }
    };
}

string_to_scalar!(&str);
string_to_scalar!(String);
string_to_scalar!(&String);

#[cfg(test)]
mod tests {
    use super::*;
    use byte_slice_cast::AsByteSlice;
    use rand::{
        distributions::{Distribution, Uniform},
        rngs::StdRng,
    };
    use rand_core::SeedableRng;
    use std::collections::HashSet;

    #[test]
    fn we_can_convert_unsigned_integers_to_scalars() {
        assert_eq!(0_u8.to_scalar(), ArkScalar::from(0_u8));
        assert_eq!(1_u16.to_scalar(), ArkScalar::from(1_u16));
        assert_eq!(1234_u32.to_scalar(), ArkScalar::from(1234_u32));
        assert_eq!(u64::MAX.to_scalar(), ArkScalar::from(u64::MAX));
    }

    #[test]
    fn we_can_convert_signed_positive_integers_to_scalars() {
        assert_eq!(0_i8.to_scalar(), ArkScalar::from(0_u8));
        assert_eq!(1_i16.to_scalar(), ArkScalar::from(1_u16));
        assert_eq!(1234_i32.to_scalar(), ArkScalar::from(1234_u32));
        assert_eq!(i64::MAX.to_scalar(), ArkScalar::from(i64::MAX as u64));
    }

    #[test]
    fn we_can_convert_signed_negative_integers_to_scalars() {
        assert_eq!((-1_i16).to_scalar(), -ArkScalar::from(1_i16 as u16));
        assert_eq!((-1234_i32).to_scalar(), -ArkScalar::from(1234_i32 as u32));
        assert_eq!((-i64::MAX).to_scalar(), -ArkScalar::from(i64::MAX as u64));
        assert_eq!(i64::MIN.to_scalar(), -ArkScalar::from(1_u64 << 63));
    }

    #[test]
    fn the_empty_string_will_be_mapped_to_the_zero_scalar() {
        assert_eq!("".to_scalar(), ArkScalar::zero());
        assert_eq!(<&str>::default().to_scalar(), ArkScalar::zero());
    }

    #[test]
    fn two_different_strings_map_to_different_scalars() {
        let s = "abc12";
        assert_ne!(s.to_scalar(), ArkScalar::zero());
        assert_ne!(s.to_scalar(), "abc123".to_scalar());
    }

    #[test]
    fn the_empty_buffer_will_be_mapped_to_the_zero_scalar() {
        assert_eq!([].to_scalar(), ArkScalar::zero());
        assert_eq!([].to_scalar(), ArkScalar::zero());
        assert_eq!(Vec::default().to_scalar(), ArkScalar::zero());
        assert_eq!(<Vec<u8>>::default().to_scalar(), ArkScalar::zero());
    }

    #[test]
    fn byte_arrays_with_the_same_content_but_different_types_map_to_different_scalars() {
        let array = [1_u8, 2_u8, 34_u8];
        assert_ne!(array.as_byte_slice().to_scalar(), ArkScalar::zero());
        assert_ne!(
            array.as_byte_slice().to_scalar(),
            [1_u32, 2_u32, 34_u32].as_byte_slice().to_scalar()
        );
    }

    #[test]
    fn strings_of_arbitrary_size_map_to_different_scalars() {
        let mut prev_scalars = HashSet::new();
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(1, 100);

        for _ in 0..100 {
            let s = dist.sample(&mut rng).to_string()
                + "testing string to scalar"
                    .repeat(dist.sample(&mut rng))
                    .as_str();
            assert!(prev_scalars.insert(s.as_str().to_scalar()));
        }
    }

    #[test]
    fn byte_arrays_of_arbitrary_size_map_to_different_scalars() {
        let mut prev_scalars = HashSet::new();
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(1, 100);

        for _ in 0..100 {
            let v = (0..dist.sample(&mut rng))
                .map(|_v| (dist.sample(&mut rng) % 255) as u8)
                .collect::<Vec<u8>>();
            assert!(prev_scalars.insert((v[..]).to_scalar()));
        }
    }

    #[test]
    fn the_string_hash_implementation_uses_the_full_range_of_bits() {
        let max_iters = 20;
        let mut rng = StdRng::from_seed([0u8; 32]);
        let dist = Uniform::new(1, i32::MAX);

        for i in 0..252 {
            let mut curr_iters = 0;
            let mut bset = HashSet::new();

            loop {
                let s = dist.sample(&mut rng).to_string().as_str().to_scalar();
                let bytes = s.as_bytes();

                let is_ith_bit_set = bytes[i / 8] & (1 << (i % 8)) != 0;

                bset.insert(is_ith_bit_set);

                if bset == HashSet::from([false, true]) {
                    break;
                }

                // this guarantees that, if the above test fails,
                // we'll be able to identify it's failing
                assert!(curr_iters <= max_iters);

                curr_iters += 1;
            }
        }
    }
}
