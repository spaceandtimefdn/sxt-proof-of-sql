//! Module holding the `RefInto` trait.

/// A reference-to-value conversion that does not consume the input value.
///
/// This is automatically implemented for all `S` where `&S` implements `Into<T>`.
///
/// This is primarily useful when defining subtraits. For example, here is a trait that requires
/// the implementation of conversions to and from `usize` for both values and references:
/// ```ignore
/// pub trait SubTrait: From<usize> + Into<usize> + for<'a> From<&'a usize> + RefInto<usize> {
///     ...
/// }
/// ```
pub trait RefInto<T> {
    /// Converts a reference to this type into the (usually inferred) input type.
    fn ref_into(&self) -> T;
}
impl<T, S> RefInto<T> for S
where
    for<'a> &'a S: Into<T>,
{
    fn ref_into(&self) -> T {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::RefInto;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_scalar_ref_into_u64_4_does_not_panic() {
        let s = TestScalar::from(0i32);
        let _: [u64; 4] = s.ref_into();
    }

    #[test]
    fn zero_scalar_ref_into_is_all_zeros() {
        let s = TestScalar::from(0i32);
        let limbs: [u64; 4] = s.ref_into();
        assert_eq!(limbs, [0u64; 4]);
    }

    #[test]
    fn nonzero_scalar_ref_into_is_nonzero() {
        let s = TestScalar::from(1i32);
        let limbs: [u64; 4] = s.ref_into();
        assert_ne!(limbs, [0u64; 4]);
    }

    #[test]
    fn distinct_scalars_produce_distinct_limbs() {
        let a = TestScalar::from(1i32);
        let b = TestScalar::from(2i32);
        let limbs_a: [u64; 4] = a.ref_into();
        let limbs_b: [u64; 4] = b.ref_into();
        assert_ne!(limbs_a, limbs_b);
    }
}
