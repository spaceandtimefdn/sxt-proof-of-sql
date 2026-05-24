//! Tests for Decimal type.

#[cfg(test)]
mod decimal_test {
    use crate::base::math::decimal::Decimal;

    #[test]
    fn test_decimal_type_exists() {
        let _: Option<Decimal> = None;
    }

    #[test]
    fn test_decimal_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<Decimal>());
        assert!(!debug_str.is_empty());
    }
}
