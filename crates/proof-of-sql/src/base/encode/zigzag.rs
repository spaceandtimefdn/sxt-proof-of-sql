use crate::base::{encode::U256, scalar::Scalar};

/// A trait for enabling zig-zag encoding
///
/// See <https://developers.google.com/protocol-buffers/docs/encoding#signed-ints>
/// for a descriptive reference.
pub trait ZigZag<T> {
    /// Encodes this ZigZag-enabled type into the type specified by implementation
    fn zigzag(&self) -> T;
}

/// Zigzag conversion from a generic Scalar to a [`ZigZag`] u256 integer
impl<S: Scalar> ZigZag<U256> for S {
    fn zigzag(&self) -> U256 {
        let mut x = U256::from(self.to_limbs());
        let mut y = U256::from((-*self).to_limbs());

        // we return the smallest ZigZag number between x and y
        // in case x is bigger than y, we return -y (encoded in the ZigZag format)
        // otherwise, we simply return x (also in the ZigZag format).
        if x.high > y.high || (x.high == y.high && x.low > y.low) {
            // y is smaller than x
            // we multiply y by 2
            y.high = (y.high << 1) | (y.low >> 127);
            y.low <<= 1;

            // then we subtract 1 from y
            let (low_val, carry_low) = y.low.overflowing_sub(1_u128);

            y.low = low_val;
            y.high -= u128::from(carry_low); // we should never expect overflow here

            // effectively encoding a ZigZag y
            y
        } else {
            // x is smaller than y
            // we multiply x by 2 (effectively encoding a ZigZag x)
            x.high = (x.high << 1) | (x.low >> 127);
            x.low <<= 1;

            x
        }
    }
}

/// Zigzag conversion from an u256 integer to a generic Scalar.
impl<S: Scalar> ZigZag<S> for U256 {
    fn zigzag(&self) -> S {
        // we need to divide self by 2 to remove the ZigZag encoding
        let mut zig_val = U256 {
            low: (self.low >> 1) | ((self.high & 1) << 127),
            high: self.high >> 1,
        };

        // verify if self is an odd or even number
        // in case it's an odd number, then scal represents the number `y`
        // otherwise, it represents the number x
        if self.low & 1 == 1 {
            // we need to sum 1 to zig_val.low to obtain the correct y value
            // in case of addition overflow, we also sum 1 to zig_val.high.
            let (low_val, carry_low) = zig_val.low.overflowing_add(1_u128);

            zig_val.low = low_val;
            zig_val.high += u128::from(carry_low); // we should never expect overflow here

            let limbs: [u64; 4] = (&zig_val).into();
            let scal = S::from_limbs(limbs);

            -scal
        } else {
            let limbs: [u64; 4] = (&zig_val).into();
            let scal = S::from_limbs(limbs);

            // return x
            scal
        }
    }
}
