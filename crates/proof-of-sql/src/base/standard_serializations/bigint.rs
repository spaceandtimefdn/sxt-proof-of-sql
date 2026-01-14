use serde::{Serialize, Serializer};

pub(crate) fn serialize_bigint_as_string_for_json<S>(
    arr: &[i64],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        arr.iter()
            .map(|val| format!("{}n", val))
            .collect::<Vec<_>>()
            .serialize(serializer)
    } else {
        arr.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::serialize_bigint_as_string_for_json;
    use crate::base::try_standard_binary_serialization;
    use serde::{Deserialize, Serialize};
    use serde_json;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        #[serde(serialize_with = "serialize_bigint_as_string_for_json")]
        values: Vec<i64>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStructWithoutCustomSerialization {
        values: Vec<i64>,
    }

    #[test]
    fn test_serialize_bigint_as_string_for_json() {
        let test_struct = TestStruct {
            values: vec![1234567890123456789, -987654321098765432],
        };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        let expected = "{\"values\":[\"1234567890123456789n\",\"-987654321098765432n\"]}";
        assert_eq!(serialized, expected);
    }

    #[test]
    fn serializer_method_does_nothing_with_binary_serialization() {
        let test_struct = TestStruct {
            values: vec![1234567890123456789, -987654321098765432],
        };
        let serialized = try_standard_binary_serialization(&test_struct).unwrap();
        let test_struct_without_custom = TestStructWithoutCustomSerialization {
            values: vec![1234567890123456789, -987654321098765432],
        };
        let serialized_expected =
            try_standard_binary_serialization(test_struct_without_custom).unwrap();
        assert_eq!(serialized, serialized_expected);
    }
}
