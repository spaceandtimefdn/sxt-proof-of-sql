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
    use ark_bn254::Fr;
    use ark_ff::PrimeField;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

    #[derive(CanonicalSerialize, CanonicalDeserialize, Debug, PartialEq, Eq)]
    struct CheckedArkSerdeTest(Fr);

    #[derive(CanonicalSerialize, CanonicalDeserialize, Debug, PartialEq, Eq)]
    struct UncheckedArkSerdeTest(Fr);

    impl_serde_for_ark_serde_checked!(CheckedArkSerdeTest);
    impl_serde_for_ark_serde_unchecked!(UncheckedArkSerdeTest);

    #[test]
    fn we_can_roundtrip_checked_ark_serde_type() {
        let value = CheckedArkSerdeTest(Fr::from(123u64));
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: CheckedArkSerdeTest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, value);
    }

    #[test]
    fn we_can_roundtrip_unchecked_ark_serde_type() {
        let value = UncheckedArkSerdeTest(Fr::from(456u64));
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: UncheckedArkSerdeTest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, value);
    }

    #[test]
    fn we_reject_invalid_checked_ark_serde_bytes() {
        let invalid_field_element_bytes = vec![u8::MAX; Fr::MODULUS_BIT_SIZE.div_ceil(8) as usize];
        let serialized = serde_json::to_string(&invalid_field_element_bytes).unwrap();

        assert!(serde_json::from_str::<CheckedArkSerdeTest>(&serialized).is_err());
    }
}
