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

    #[derive(Debug, PartialEq, Eq)]
    struct Wrapper(u8);

    impl From<&Wrapper> for u16 {
        fn from(value: &Wrapper) -> Self {
            u16::from(value.0)
        }
    }

    #[test]
    fn ref_into_uses_reference_conversion_without_consuming_value() {
        let wrapper = Wrapper(7);

        assert_eq!(wrapper.ref_into(), 7_u16);
        assert_eq!(wrapper, Wrapper(7));
    }
}
