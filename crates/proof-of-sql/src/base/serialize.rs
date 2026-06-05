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
    use super::{impl_serde_for_ark_serde_checked, impl_serde_for_ark_serde_unchecked};
    use ark_serialize::{
        CanonicalDeserialize, CanonicalSerialize, Compress, Read, SerializationError, Valid,
        Validate, Write,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct CheckedByte(u8);

    impl Valid for CheckedByte {
        fn check(&self) -> Result<(), SerializationError> {
            if self.0 <= 7 {
                Ok(())
            } else {
                Err(SerializationError::InvalidData)
            }
        }
    }

    impl CanonicalSerialize for CheckedByte {
        fn serialize_with_mode<W: Write>(
            &self,
            mut writer: W,
            compress: Compress,
        ) -> Result<(), SerializationError> {
            assert!(matches!(compress, Compress::Yes));
            writer.write_all(&[self.0])?;
            Ok(())
        }

        fn serialized_size(&self, compress: Compress) -> usize {
            assert!(matches!(compress, Compress::Yes));
            1
        }
    }

    impl CanonicalDeserialize for CheckedByte {
        fn deserialize_with_mode<R: Read>(
            mut reader: R,
            compress: Compress,
            validate: Validate,
        ) -> Result<Self, SerializationError> {
            assert!(matches!(compress, Compress::Yes));
            let mut bytes = [0_u8; 1];
            reader.read_exact(&mut bytes)?;
            let value = Self(bytes[0]);
            if validate == Validate::Yes {
                value.check()?;
            }
            Ok(value)
        }
    }

    impl_serde_for_ark_serde_checked!(CheckedByte);

    #[derive(Debug, PartialEq, Eq)]
    struct UncheckedByte(u8);

    impl Valid for UncheckedByte {
        fn check(&self) -> Result<(), SerializationError> {
            if self.0 <= 7 {
                Ok(())
            } else {
                Err(SerializationError::InvalidData)
            }
        }
    }

    impl CanonicalSerialize for UncheckedByte {
        fn serialize_with_mode<W: Write>(
            &self,
            mut writer: W,
            compress: Compress,
        ) -> Result<(), SerializationError> {
            assert!(matches!(compress, Compress::Yes));
            writer.write_all(&[self.0])?;
            Ok(())
        }

        fn serialized_size(&self, compress: Compress) -> usize {
            assert!(matches!(compress, Compress::Yes));
            1
        }
    }

    impl CanonicalDeserialize for UncheckedByte {
        fn deserialize_with_mode<R: Read>(
            mut reader: R,
            compress: Compress,
            validate: Validate,
        ) -> Result<Self, SerializationError> {
            assert!(matches!(compress, Compress::Yes));
            let mut bytes = [0_u8; 1];
            reader.read_exact(&mut bytes)?;
            let value = Self(bytes[0]);
            if validate == Validate::Yes {
                value.check()?;
            }
            Ok(value)
        }
    }

    impl_serde_for_ark_serde_unchecked!(UncheckedByte);

    #[test]
    fn checked_serde_round_trips_valid_canonical_bytes() {
        let encoded = serde_json::to_vec(&CheckedByte(7)).unwrap();
        assert_eq!(encoded, b"[7]");

        let decoded: CheckedByte = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, CheckedByte(7));
    }

    #[test]
    fn checked_deserialization_rejects_invalid_canonical_data() {
        let encoded_invalid_bytes = serde_json::to_vec(&[8_u8]).unwrap();

        assert!(serde_json::from_slice::<CheckedByte>(&encoded_invalid_bytes).is_err());
    }

    #[test]
    fn unchecked_deserialization_skips_canonical_validation() {
        let encoded_invalid_bytes = serde_json::to_vec(&[8_u8]).unwrap();

        let decoded: UncheckedByte = serde_json::from_slice(&encoded_invalid_bytes).unwrap();
        assert_eq!(decoded, UncheckedByte(8));
    }
}
