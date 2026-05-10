/// Implements serde Serialize/Deserialize for types using [`ark_serialize`] with checked deserialization.
///
/// Uses [`serialize_compressed`] and [`deserialize_compressed`] which validates data on deserialization.
macro_rules! impl_serde_for_ark_serde_checked {
    ($t:ty) => {
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut bytes =
                    Vec::with_capacity(ark_serialize::CanonicalSerialize::compressed_size(self));
                ark_serialize::CanonicalSerialize::serialize_compressed(self, &mut bytes)
                    .map_err(serde::ser::Error::custom)?;
                bytes.serialize(serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                ark_serialize::CanonicalDeserialize::deserialize_compressed(
                    Vec::deserialize(deserializer)?.as_slice(),
                )
                .map_err(serde::de::Error::custom)
            }
        }
    };
}

/// Implements serde Serialize/Deserialize for types using [`ark_serialize`] with unchecked deserialization.
///
/// Uses [`serialize_compressed`] and [`deserialize_compressed_unchecked`] which skips validation.
/// This is faster but should only be used when the data source is trusted.
macro_rules! impl_serde_for_ark_serde_unchecked {
    ($t:ty) => {
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut bytes =
                    Vec::with_capacity(ark_serialize::CanonicalSerialize::compressed_size(self));
                ark_serialize::CanonicalSerialize::serialize_compressed(self, &mut bytes)
                    .map_err(serde::ser::Error::custom)?;
                bytes.serialize(serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                ark_serialize::CanonicalDeserialize::deserialize_compressed_unchecked(
                    Vec::deserialize(deserializer)?.as_slice(),
                )
                .map_err(serde::de::Error::custom)
            }
        }
    };
}

pub(crate) use impl_serde_for_ark_serde_checked;
pub(crate) use impl_serde_for_ark_serde_unchecked;

#[cfg(test)]
mod tests {
    use crate::base::{try_standard_binary_deserialization, try_standard_binary_serialization};
    use ark_serialize::{
        CanonicalDeserialize, CanonicalSerialize, Compress, Read, SerializationError, Valid,
        Validate, Write,
    };

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct CheckedSerdeByte(u8);

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct UncheckedSerdeByte(u8);

    macro_rules! impl_canonical_byte {
        ($t:ty) => {
            impl CanonicalSerialize for $t {
                fn serialize_with_mode<W: Write>(
                    &self,
                    writer: W,
                    compress: Compress,
                ) -> Result<(), SerializationError> {
                    self.0.serialize_with_mode(writer, compress)
                }

                fn serialized_size(&self, compress: Compress) -> usize {
                    self.0.serialized_size(compress)
                }
            }

            impl Valid for $t {
                fn check(&self) -> Result<(), SerializationError> {
                    if self.0 == u8::MAX {
                        Err(SerializationError::InvalidData)
                    } else {
                        Ok(())
                    }
                }
            }

            impl CanonicalDeserialize for $t {
                fn deserialize_with_mode<R: Read>(
                    reader: R,
                    compress: Compress,
                    validate: Validate,
                ) -> Result<Self, SerializationError> {
                    let value = Self(u8::deserialize_with_mode(reader, compress, Validate::No)?);
                    if validate == Validate::Yes {
                        value.check()?;
                    }
                    Ok(value)
                }
            }
        };
    }

    impl_canonical_byte!(CheckedSerdeByte);
    impl_canonical_byte!(UncheckedSerdeByte);
    impl_serde_for_ark_serde_checked!(CheckedSerdeByte);
    impl_serde_for_ark_serde_unchecked!(UncheckedSerdeByte);

    #[test]
    fn checked_ark_serde_helper_roundtrips_valid_values() {
        let value = CheckedSerdeByte(42);

        let serialized = try_standard_binary_serialization(value).unwrap();
        let (deserialized, consumed): (CheckedSerdeByte, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(deserialized, value);
        assert_eq!(consumed, serialized.len());
    }

    #[test]
    fn checked_ark_serde_helper_rejects_invalid_values() {
        let serialized = try_standard_binary_serialization(CheckedSerdeByte(u8::MAX)).unwrap();

        let result: Result<(CheckedSerdeByte, usize), _> =
            try_standard_binary_deserialization(&serialized);

        assert!(result.is_err());
    }

    #[test]
    fn unchecked_ark_serde_helper_accepts_invalid_values() {
        let value = UncheckedSerdeByte(u8::MAX);

        let serialized = try_standard_binary_serialization(value).unwrap();
        let (deserialized, consumed): (UncheckedSerdeByte, _) =
            try_standard_binary_deserialization(&serialized).unwrap();

        assert_eq!(deserialized, value);
        assert_eq!(consumed, serialized.len());
    }
}
