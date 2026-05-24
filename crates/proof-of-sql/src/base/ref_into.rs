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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Counter(u32);

    impl From<&Counter> for u32 {
        fn from(value: &Counter) -> Self {
            value.0
        }
    }

    impl From<&Counter> for String {
        fn from(value: &Counter) -> Self {
            value.0.to_string()
        }
    }

    #[test]
    fn it_uses_reference_based_into_for_custom_type() {
        let counter = Counter(42);
        let as_u32: u32 = counter.ref_into();
        assert_eq!(as_u32, 42u32);
    }

    #[test]
    fn it_can_target_multiple_output_types() {
        let counter = Counter(11);
        let as_string: String = counter.ref_into();
        assert_eq!(as_string, "11");
    }
}
