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
    struct Source {
        value: usize,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Converted {
        value: usize,
        was_from_ref: bool,
    }

    impl From<&Source> for Converted {
        fn from(source: &Source) -> Self {
            Self {
                value: source.value,
                was_from_ref: true,
            }
        }
    }

    #[test]
    fn ref_into_uses_reference_conversion_without_consuming_source() {
        let source = Source { value: 37 };

        let converted: Converted = source.ref_into();

        assert_eq!(
            converted,
            Converted {
                value: 37,
                was_from_ref: true,
            }
        );
        assert_eq!(source.value, 37);
    }
}
