//! Tests for nullable column support

#[cfg(test)]
mod tests {
    use super::super::{Column, ColumnType, LiteralValue, OwnedColumn};
    use crate::base::scalar::test_scalar::TestScalar;
    use bumpalo::Bump;

    #[test]
    fn test_nullable_column_type_creation() {
        // Test creating nullable column types
        let nullable_bigint = ColumnType::Nullable(Box::new(ColumnType::BigInt));
        let nullable_varchar = ColumnType::Nullable(Box::new(ColumnType::VarChar));
        
        // Test that they have the correct type
        assert!(matches!(nullable_bigint, ColumnType::Nullable(_)));
        assert!(matches!(nullable_varchar, ColumnType::Nullable(_)));
    }

    #[test]
    fn test_nullable_column_type_properties() {
        let nullable_bigint = ColumnType::Nullable(Box::new(ColumnType::BigInt));
        let nullable_varchar = ColumnType::Nullable(Box::new(ColumnType::VarChar));
        
        // Test that nullable types inherit properties from inner types
        assert!(nullable_bigint.is_numeric());
        assert!(nullable_bigint.is_integer());
        assert!(!nullable_varchar.is_numeric());
        assert!(!nullable_varchar.is_integer());
        
        // Test precision and scale
        assert_eq!(nullable_bigint.precision_value(), Some(19));
        assert_eq!(nullable_bigint.scale(), Some(0));
        assert_eq!(nullable_varchar.precision_value(), None);
        assert_eq!(nullable_varchar.scale(), None);
    }

    #[test]
    fn test_null_literal() {
        let null_literal = LiteralValue::Null;
        
        // Test null literal properties
        assert!(null_literal.is_null());
        assert!(matches!(null_literal.column_type(), ColumnType::Nullable(_)));
        
        // Test scalar conversion
        let scalar = null_literal.to_scalar::<TestScalar>();
        assert_eq!(scalar, TestScalar::from(0));
    }

    #[test]
    fn test_nullable_owned_column_creation() {
        // Create nullable integer column with some nulls
        let data = vec![1, 2, 0, 4]; // 0 is dummy value for null
        let null_bitmap = vec![true, true, false, true]; // false means null
        
        let inner_col = OwnedColumn::BigInt(data);
        let nullable_col = OwnedColumn::Nullable(Box::new(inner_col), null_bitmap);
        
        // Test basic properties
        assert_eq!(nullable_col.len(), 4);
        assert!(!nullable_col.is_empty());
        assert!(matches!(nullable_col.column_type(), ColumnType::Nullable(_)));
    }

    #[test]
    fn test_nullable_owned_column_operations() {
        // Create test data
        let data = vec![10, 20, 0, 40, 50];
        let null_bitmap = vec![true, true, false, true, true];
        
        let inner_col = OwnedColumn::BigInt(data);
        let nullable_col = OwnedColumn::Nullable(Box::new(inner_col), null_bitmap);
        
        // Test slicing
        let sliced = nullable_col.slice(1, 4);
        assert_eq!(sliced.len(), 3);
        
        // Test that slicing preserves nullable structure
        assert!(matches!(sliced, OwnedColumn::Nullable(_, _)));
        if let OwnedColumn::Nullable(inner, bitmap) = sliced {
            if let OwnedColumn::BigInt(values) = *inner {
                assert_eq!(values, vec![20, 0, 40]);
            }
            assert_eq!(bitmap, vec![true, false, true]);
        }
    }

    #[test]
    fn test_nullable_column_from_literal() {
        let alloc = Bump::new();
        
        // Test creating column from null literal
        let null_literal = LiteralValue::Null;
        let col = Column::<TestScalar>::from_literal_with_length(&null_literal, 3, &alloc);
        
        assert_eq!(col.len(), 3);
        assert!(matches!(col, Column::Nullable(_, _)));
        
        if let Column::Nullable(_inner, null_bitmap) = col {
            // All values should be null
            assert_eq!(null_bitmap, &[false, false, false]);
        }
    }

    #[test]
    fn test_nullable_column_type_system_integration() {
        let nullable_int = ColumnType::Nullable(Box::new(ColumnType::Int));
        let nullable_decimal = ColumnType::Nullable(Box::new(ColumnType::Decimal75(
            crate::base::math::decimal::Precision::new(10).unwrap(),
            2,
        )));
        
        // Test byte size includes null bitmap overhead
        assert_eq!(nullable_int.byte_size(), 4 + 1); // int size + bool size
        
        // Test signed property delegation
        assert!(nullable_int.is_signed());
        
        // Test precision/scale delegation
        assert_eq!(nullable_decimal.precision_value(), Some(10));
        assert_eq!(nullable_decimal.scale(), Some(2));
    }

    #[test]
    fn test_nullable_column_inner_product() {
        use crate::base::slice_ops::inner_product_ref_cast;
        
        // Create test data with nulls
        let data = vec![1, 2, 0, 4]; // 0 is dummy for null at index 2
        let null_bitmap = vec![true, true, false, true];
        
        let inner_col = OwnedColumn::BigInt(data.clone());
        let nullable_col = OwnedColumn::Nullable(Box::new(inner_col), null_bitmap);
        
        let multipliers = vec![
            TestScalar::from(1),
            TestScalar::from(2), 
            TestScalar::from(3),
            TestScalar::from(4)
        ];
        
        // Inner product should work on underlying data for now
        // TODO: In full implementation, this should handle nulls properly
        let result = nullable_col.inner_product(&multipliers);
        let expected = inner_product_ref_cast(&data, &multipliers);
        assert_eq!(result, expected);
    }
}
