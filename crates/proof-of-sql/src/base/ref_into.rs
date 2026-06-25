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

    struct Wrapper(i32);

    impl<'a> From<&'a Wrapper> for i64 {
        fn from(w: &'a Wrapper) -> i64 {
            i64::from(w.0)
        }
    }

    impl<'a> From<&'a Wrapper> for i128 {
        fn from(w: &'a Wrapper) -> i128 {
            i128::from(w.0)
        }
    }

    #[test]
    fn ref_into_delegates_to_from_impl() {
        let w = Wrapper(42);
        let result: i64 = w.ref_into();
        assert_eq!(result, 42i64);
    }

    #[test]
    fn ref_into_works_with_negative_value() {
        let w = Wrapper(-100);
        let result: i64 = w.ref_into();
        assert_eq!(result, -100i64);
    }

    #[test]
    fn ref_into_does_not_consume_the_value() {
        let w = Wrapper(7);
        let r1: i64 = w.ref_into();
        let r2: i64 = w.ref_into();
        assert_eq!(r1, r2);
    }

    #[test]
    fn ref_into_zero_value() {
        let w = Wrapper(0);
        let result: i64 = w.ref_into();
        assert_eq!(result, 0i64);
    }

    #[test]
    fn ref_into_works_with_different_target_type() {
        let w = Wrapper(99);
        let result: i128 = w.ref_into();
        assert_eq!(result, 99i128);
    }

    #[test]
    fn ref_into_preserves_max_i32() {
        let w = Wrapper(i32::MAX);
        let result: i64 = w.ref_into();
        assert_eq!(result, i64::from(i32::MAX));
    }

    #[test]
    fn ref_into_preserves_min_i32() {
        let w = Wrapper(i32::MIN);
        let result: i64 = w.ref_into();
        assert_eq!(result, i64::from(i32::MIN));
    }
}
