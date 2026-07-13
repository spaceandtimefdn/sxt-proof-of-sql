//! Tests for ColumnCommitmentMetadata errors.

#[cfg(test)]
mod column_commitment_metadata_error_test {
    use crate::base::commitment::{
        ColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch, InvalidColumnCommitmentMetadata,
    };
    use crate::base::database::ColumnType;
    use crate::base::commitment::ColumnBounds;

    #[test]
    fn test_invalid_metadata_display() {
        let bounds = ColumnBounds::BigInt(crate::base::commitment::BoundsInner::sharp(
            crate::base::scalar::Scalar::from(1i64),
            crate::base::scalar::Scalar::from(100i64),
        ).unwrap());
        let err = InvalidColumnCommitmentMetadata::TypeBoundsMismatch {
            column_type: ColumnType::VarChar,
            column_bounds: bounds,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
        assert!(s.contains("VarChar") || s.contains("cannot have bounds"));
    }

    #[test]
    fn test_invalid_metadata_debug() {
        let bounds = ColumnBounds::NoOrder;
        let err = InvalidColumnCommitmentMetadata::TypeBoundsMismatch {
            column_type: ColumnType::Boolean,
            column_bounds: bounds,
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("InvalidColumnCommitmentMetadata"));
    }

    #[test]
    fn test_mismatch_display() {
        let err = ColumnCommitmentMetadataMismatch {
            datatype_a: ColumnType::BigInt,
            datatype_b: ColumnType::VarChar,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
        assert!(s.contains("BigInt") && s.contains("VarChar"));
    }

    #[test]
    fn test_mismatch_debug() {
        let err = ColumnCommitmentMetadataMismatch {
            datatype_a: ColumnType::Int,
            datatype_b: ColumnType::SmallInt,
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_metadata_try_new_valid() {
        let bounds = ColumnBounds::BigInt(crate::base::commitment::BoundsInner::sharp(
            crate::base::scalar::Scalar::from(0i64),
            crate::base::scalar::Scalar::from(100i64),
        ).unwrap());
        let result = ColumnCommitmentMetadata::try_new(ColumnType::BigInt, bounds);
        assert!(result.is_ok());
    }

    #[test]
    fn test_metadata_try_new_invalid() {
        let bounds = ColumnBounds::BigInt(crate::base::commitment::BoundsInner::sharp(
            crate::base::scalar::Scalar::from(0i64),
            crate::base::scalar::Scalar::from(100i64),
        ).unwrap());
        let result = ColumnCommitmentMetadata::try_new(ColumnType::VarChar, bounds);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_clone() {
        let bounds = ColumnBounds::NoOrder;
        let result = ColumnCommitmentMetadata::try_new(ColumnType::Boolean, bounds);
        if let Ok(metadata) = result {
            let cloned = *metadata;
            assert_eq!(metadata, &cloned);
        }
    }

    #[test]
    fn test_metadata_debug() {
        let bounds = ColumnBounds::NoOrder;
        let result = ColumnCommitmentMetadata::try_new(ColumnType::Scalar, bounds);
        if let Ok(metadata) = result {
            let debug_str = format!("{:?}", metadata);
            assert!(!debug_str.is_empty());
        }
    }
}