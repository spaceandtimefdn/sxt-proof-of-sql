//! Tests for LiteralValue.

#[cfg(test)]
mod literal_value_test {
    use crate::base::database::LiteralValue;

    #[test]
    fn test_literal_value_type_exists() {
        let _: Option<LiteralValue> = None;
    }

    #[test]
    fn test_literal_value_debug() {
        let val = LiteralValue::Boolean(true);
        let debug_str = format!("{:?}", val);
        assert!(!debug_str.is_empty());
    }
}
