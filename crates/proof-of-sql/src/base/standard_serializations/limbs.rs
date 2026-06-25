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

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct LimbsWrapper {
        #[serde(
            serialize_with = "serialize_limbs",
            deserialize_with = "deserialize_to_limbs"
        )]
        limbs: [u64; 4],
    }

    #[test]
    fn roundtrip_preserves_all_limb_values() {
        let w = LimbsWrapper { limbs: [1, 2, 3, 4] };
        let json = serde_json::to_string(&w).unwrap();
        let back: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(back, w);
    }

    #[test]
    fn serialize_reverses_limb_order() {
        let w = LimbsWrapper { limbs: [1, 2, 3, 4] };
        let json = serde_json::to_string(&w).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = v["limbs"].as_array().unwrap();
        assert_eq!(arr[0].as_u64().unwrap(), 4);
        assert_eq!(arr[1].as_u64().unwrap(), 3);
        assert_eq!(arr[2].as_u64().unwrap(), 2);
        assert_eq!(arr[3].as_u64().unwrap(), 1);
    }

    #[test]
    fn roundtrip_with_all_zeros() {
        let w = LimbsWrapper { limbs: [0, 0, 0, 0] };
        let json = serde_json::to_string(&w).unwrap();
        let back: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(back, w);
    }

    #[test]
    fn roundtrip_with_max_values() {
        let w = LimbsWrapper { limbs: [u64::MAX, u64::MAX, u64::MAX, u64::MAX] };
        let json = serde_json::to_string(&w).unwrap();
        let back: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(back, w);
    }

    #[test]
    fn deserialize_reverses_encoded_order() {
        // Wire format: [4,3,2,1] → decoded limbs: [1,2,3,4]
        let json = r#"{"limbs":[4,3,2,1]}"#;
        let back: LimbsWrapper = serde_json::from_str(json).unwrap();
        assert_eq!(back.limbs, [1, 2, 3, 4]);
    }

    #[test]
    fn roundtrip_with_distinct_values_per_limb() {
        let w = LimbsWrapper { limbs: [0xAABB_CCDD_0011_2233, 0x4455_6677_8899_AABB, 0xCCDD_EEFF_0102_0304, 0x0506_0708_090A_0B0C] };
        let json = serde_json::to_string(&w).unwrap();
        let back: LimbsWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(back, w);
    }
}
