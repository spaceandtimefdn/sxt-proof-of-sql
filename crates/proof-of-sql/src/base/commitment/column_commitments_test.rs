//! Tests for ColumnCommitments errors.

#[cfg(test)]
mod column_commitments_error_test {
    use crate::base::commitment::{AppendColumnCommitmentsError, ColumnCommitmentsMismatch, DuplicateIdents};

    #[test]
    fn test_duplicate_idents_display() {
        let err = DuplicateIdents { id: "test_col".to_string() };
        assert_eq!(format!("{}", err), "cannot create commitments with duplicate ident: test_col");
    }

    #[test]
    fn test_duplicate_idents_debug() {
        let err = DuplicateIdents { id: "test_col".to_string() };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("DuplicateIdents"));
    }

    #[test]
    fn test_duplicate_idents_partial_eq() {
        let err1 = DuplicateIdents { id: "col1".to_string() };
        let err2 = DuplicateIdents { id: "col1".to_string() };
        let err3 = DuplicateIdents { id: "col2".to_string() };
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_mismatch_error_source() {
        let err = AppendColumnCommitmentsError::Mismatch {
            source: ColumnCommitmentsMismatch {
                datatype_a: crate::base::database::ColumnType::BigInt,
                datatype_b: crate::base::database::ColumnType::VarChar,
            },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_mismatch_error_source2() {
        let err = AppendColumnCommitmentsError::DuplicateIdents {
            source: DuplicateIdents { id: "dup".to_string() },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_append_error_debug() {
        let err = AppendColumnCommitmentsError::DuplicateIdents {
            source: DuplicateIdents { id: "test".to_string() },
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_column_commitments_mismatch_display() {
        let err = ColumnCommitmentsMismatch {
            datatype_a: crate::base::database::ColumnType::BigInt,
            datatype_b: crate::base::database::ColumnType::Int,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
        assert!(s.contains("BigInt") || s.contains("Int"));
    }

    #[test]
    fn test_column_commitments_mismatch_debug() {
        let err = ColumnCommitmentsMismatch {
            datatype_a: crate::base::database::ColumnType::SmallInt,
            datatype_b: crate::base::database::ColumnType::TinyInt,
        };
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }
}