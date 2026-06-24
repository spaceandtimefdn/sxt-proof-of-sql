use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) fn serialize_limbs<S: Serializer>(
    limbs: &[u64; 4],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    [limbs[3], limbs[2], limbs[1], limbs[0]].serialize(serializer)
}

pub(crate) fn deserialize_to_limbs<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<[u64; 4], D::Error> {
    let limbs = <[u64; 4]>::deserialize(deserializer)?;
    Ok([limbs[3], limbs[2], limbs[1], limbs[0]])
}

#[cfg(test)]
mod tests {
    use super::{deserialize_to_limbs, serialize_limbs};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct LimbsWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn serialize_reverses_limb_order() {
        let w = LimbsWrapper { limbs: [1, 2, 3, 4] };
        let json = serde_json::to_string(&w).unwrap();
        assert!(json.contains("[4,3,2,1]"), "expected reversed order, got: {json}");
    }

    #[test]
    fn deserialize_reverses_limb_order_back() {
        let json = r#"{"limbs":[4,3,2,1]}"#;
        let result: LimbsWrapper = serde_json::from_str(json).unwrap();
        assert_eq!(result.limbs, [1, 2, 3, 4]);
    }

    #[test]
    fn roundtrip_preserves_original_limbs() {
        let original = LimbsWrapper { limbs: [10, 20, 30, 40] };
        let json = serde_json::to_string(&original).unwrap();
        let recovered: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.limbs, original.limbs);
    }

    #[test]
    fn zero_limbs_roundtrip() {
        let w = LimbsWrapper { limbs: [0, 0, 0, 0] };
        let json = serde_json::to_string(&w).unwrap();
        let recovered: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.limbs, [0, 0, 0, 0]);
    }

    #[test]
    fn all_distinct_limbs_roundtrip() {
        let w = LimbsWrapper { limbs: [u64::MAX, 0, u64::MAX / 2, 12345] };
        let json = serde_json::to_string(&w).unwrap();
        let recovered: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.limbs, w.limbs);
    }

    #[test]
    fn serialize_positions_are_reversed() {
        let w = LimbsWrapper { limbs: [0, 0, 0, 99] };
        let json = serde_json::to_string(&w).unwrap();
        let inner: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(inner["limbs"][0], serde_json::json!(99));
        assert_eq!(inner["limbs"][3], serde_json::json!(0));
    }
}
