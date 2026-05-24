#[cfg(test)]
mod column_type_test {
    use crate::base::database::ColumnType;
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use crate::base::math::decimal::Precision;

    #[test]
    fn test_column_type_variants() {
        // Test all ColumnType variants can be constructed
        let _ = ColumnType::Boolean;
        let _ = ColumnType::Uint8;
        let _ = ColumnType::TinyInt;
        let _ = ColumnType::SmallInt;
        let _ = ColumnType::Int;
        let _ = ColumnType::BigInt;
        let _ = ColumnType::Int128;
        let _ = ColumnType::VarChar;
        let _ = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let _ = ColumnType::TimestampTZ(PoSQLTimeUnit::default(), PoSQLTimeZone::UTC);
        let _ = ColumnType::Scalar;
        let _ = ColumnType::VarBinary;
    }

    #[test]
    fn test_column_type_is_numeric() {
        assert!(ColumnType::Uint8.is_numeric());
        assert!(ColumnType::TinyInt.is_numeric());
        assert!(!ColumnType::Boolean.is_numeric());
        assert!(!ColumnType::VarChar.is_numeric());
    }

    #[test]
    fn test_column_type_is_integer() {
        assert!(ColumnType::TinyInt.is_integer());
        assert!(ColumnType::Int.is_integer());
        assert!(ColumnType::BigInt.is_integer());
        assert!(!ColumnType::VarChar.is_integer());
    }
}
