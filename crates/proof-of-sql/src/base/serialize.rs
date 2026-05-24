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
    use ark_bn254::Fr;
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

    #[derive(
        Clone,
        Debug,
        PartialEq,
        Eq,
        ark_serialize::CanonicalSerialize,
        ark_serialize::CanonicalDeserialize,
    )]
    struct CheckedFr(Fr);

    #[derive(
        Clone,
        Debug,
        PartialEq,
        Eq,
        ark_serialize::CanonicalSerialize,
        ark_serialize::CanonicalDeserialize,
    )]
    struct UncheckedFr(Fr);

    impl_serde_for_ark_serde_checked!(CheckedFr);
    impl_serde_for_ark_serde_unchecked!(UncheckedFr);

    #[test]
    fn checked_macro_round_trip_succeeds() {
        let value = CheckedFr(Fr::from(42u64));
        let encoded = serde_json::to_vec(&value).expect("serialize should succeed");
        let decoded: CheckedFr =
            serde_json::from_slice(&encoded).expect("deserialize should succeed");
        assert_eq!(decoded, value);
    }

    #[test]
    fn checked_macro_rejects_invalid_bytes() {
        let encoded =
            serde_json::to_vec(&vec![1u8, 2, 3, 4, 5]).expect("serialize bytes should succeed");
        let result: Result<CheckedFr, _> = serde_json::from_slice(&encoded);
        assert!(
            result.is_err(),
            "checked deserialization should reject malformed bytes"
        );
    }

    #[test]
    fn unchecked_macro_round_trip_succeeds() {
        let value = UncheckedFr(Fr::from(7u64));
        let encoded = serde_json::to_vec(&value).expect("serialize should succeed");
        let decoded: UncheckedFr =
            serde_json::from_slice(&encoded).expect("deserialize should succeed");
        assert_eq!(decoded, value);
    }

    #[test]
    fn unchecked_macro_rejects_truncated_bytes() {
        let encoded = serde_json::to_vec(&vec![1u8, 2, 3]).expect("serialize bytes should succeed");
        let result: Result<UncheckedFr, _> = serde_json::from_slice(&encoded);
        assert!(
            result.is_err(),
            "unchecked deserialization still requires sufficient bytes"
        );
    }
}
