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
    use alloc::{vec, vec::Vec};

    #[derive(Debug, PartialEq, Eq)]
    struct WrappedU32(u32);

    impl From<&WrappedU32> for u32 {
        fn from(value: &WrappedU32) -> Self {
            value.0
        }
    }

    #[test]
    fn ref_into_uses_reference_conversion_without_consuming_source() {
        let wrapped = WrappedU32(42);

        let converted = RefInto::<u32>::ref_into(&wrapped);

        assert_eq!(converted, 42);
        assert_eq!(wrapped, WrappedU32(42));
    }

    #[test]
    fn ref_into_can_be_used_with_iterator_adapters() {
        let wrapped = [WrappedU32(1), WrappedU32(2), WrappedU32(3)];

        let converted: Vec<_> = wrapped.iter().map(RefInto::<u32>::ref_into).collect();

        assert_eq!(converted, vec![1, 2, 3]);
        assert_eq!(wrapped[0], WrappedU32(1));
    }
}
