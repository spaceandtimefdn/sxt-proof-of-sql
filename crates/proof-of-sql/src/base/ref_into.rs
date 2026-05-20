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
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct RefSource {
        value: usize,
    }

    impl From<&RefSource> for usize {
        fn from(source: &RefSource) -> Self {
            source.value
        }
    }

    #[test]
    fn ref_into_uses_reference_based_conversion() {
        let source = RefSource { value: 42 };

        assert_eq!(source.ref_into(), 42);
        assert_eq!(source, RefSource { value: 42 });
    }

    #[test]
    fn ref_into_allows_explicit_target_types() {
        let source = RefSource { value: 7 };
        let converted: usize = source.ref_into();

        assert_eq!(converted, 7);
    }
}
