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
