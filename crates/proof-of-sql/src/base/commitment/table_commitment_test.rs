//! Tests for TableCommitment errors.

#[cfg(test)]
mod table_commitment_error_test {
    use crate::base::commitment::{
        AppendColumnCommitmentsError, AppendTableCommitmentError, ColumnCommitmentsMismatch,
        ColumnCommitments, DuplicateIdents, MixedLengthColumns, NegativeRange,
        TableCommitmentArithmeticError, TableCommitmentFromColumnsError,
    };

    #[test]
    fn test_negative_range_display() {
        let err = NegativeRange;
        assert_eq!(format!("{}", err), "cannot create a TableCommitment with a negative range");
    }

    #[test]
    fn test_negative_range_debug() {
        let err = NegativeRange;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NegativeRange"));
    }

    #[test]
    fn test_mixed_length_columns_display() {
        let err = MixedLengthColumns;
        assert_eq!(format!("{}", err), "cannot create a TableCommitment from columns of mixed length");
    }

    #[test]
    fn test_mixed_length_columns_debug() {
        let err = MixedLengthColumns;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("MixedLengthColumns"));
    }

    #[test]
    fn test_from_columns_error_mixed_length() {
        let err = TableCommitmentFromColumnsError::MixedLengthColumns {
            source: MixedLengthColumns,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_from_columns_error_duplicate_idents() {
        let err = TableCommitmentFromColumnsError::DuplicateIdents {
            source: DuplicateIdents { id: "dup_col".to_string() },
        };
        let s = format!("{}", err);
        assert!(s.contains("dup_col"));
    }

    #[test]
    fn test_append_error_mixed_length() {
        let err = AppendTableCommitmentError::MixedLengthColumns {
            source: MixedLengthColumns,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_append_error_column_commitments() {
        let err = AppendTableCommitmentError::AppendColumnCommitments {
            source: AppendColumnCommitmentsError::DuplicateIdents {
                source: DuplicateIdents { id: "test".to_string() },
            },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_arithmetic_error_column_mismatch() {
        let err = TableCommitmentArithmeticError::ColumnMismatch {
            source: ColumnCommitmentsMismatch {
                datatype_a: crate::base::database::ColumnType::BigInt,
                datatype_b: crate::base::database::ColumnType::VarChar,
            },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_arithmetic_error_negative_range() {
        let err = TableCommitmentArithmeticError::NegativeRange {
            source: NegativeRange,
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_arithmetic_error_noncontiguous() {
        let err = TableCommitmentArithmeticError::NonContiguous;
        let s = format!("{}", err);
        assert!(s.contains("noncontiguous") || s.contains("NonContiguous"));
    }

    #[test]
    fn test_unit_errors_equal() {
        let nr1 = NegativeRange;
        let nr2 = NegativeRange;
        assert_eq!(nr1, nr2);

        let ml1 = MixedLengthColumns;
        let ml2 = MixedLengthColumns;
        assert_eq!(ml1, ml2);
    }
}