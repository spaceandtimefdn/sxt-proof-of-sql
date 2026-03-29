#[cfg(test)]
mod tests {
    use crate::base::database::ColumnType;

    // -----------------------------------------------------------------------
    // Numeric type compatibility for arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn test_bigint_is_numeric() {
        assert!(ColumnType::BigInt.is_numeric());
    }

    #[test]
    fn test_int128_is_numeric() {
        assert!(ColumnType::Int128.is_numeric());
    }

    #[test]
    fn test_boolean_is_not_numeric() {
        assert!(!ColumnType::Boolean.is_numeric());
    }

    #[test]
    fn test_varchar_is_not_numeric() {
        assert!(!ColumnType::VarChar.is_numeric());
    }

    // -----------------------------------------------------------------------
    // Bit-width / byte-width ordering
    // -----------------------------------------------------------------------

    #[test]
    fn test_bigint_byte_size() {
        assert_eq!(ColumnType::BigInt.byte_size(), 8);
    }

    #[test]
    fn test_int128_byte_size() {
        assert_eq!(ColumnType::Int128.byte_size(), 16);
    }

    #[test]
    fn test_smallint_byte_size() {
        assert_eq!(ColumnType::SmallInt.byte_size(), 2);
    }

    #[test]
    fn test_int_byte_size() {
        assert_eq!(ColumnType::Int.byte_size(), 4);
    }

    #[test]
    fn test_boolean_byte_size() {
        assert_eq!(ColumnType::Boolean.byte_size(), 1);
    }

    // -----------------------------------------------------------------------
    // Type upcast / widening
    // -----------------------------------------------------------------------

    #[test]
    fn test_upcast_bigint_bigint() {
        let result = ColumnType::BigInt.upcast_to(ColumnType::BigInt);
        assert_eq!(result, Some(ColumnType::BigInt));
    }

    #[test]
    fn test_upcast_smallint_to_bigint() {
        // SmallInt can be widened to BigInt.
        let result = ColumnType::SmallInt.upcast_to(ColumnType::BigInt);
        assert_eq!(result, Some(ColumnType::BigInt));
    }

    #[test]
    fn test_upcast_bigint_to_int128() {
        let result = ColumnType::BigInt.upcast_to(ColumnType::Int128);
        assert_eq!(result, Some(ColumnType::Int128));
    }

    #[test]
    fn test_upcast_incompatible_types_returns_none() {
        // A numeric type cannot be widened to VarChar.
        let result = ColumnType::BigInt.upcast_to(ColumnType::VarChar);
        assert_eq!(result, None);
    }
}
