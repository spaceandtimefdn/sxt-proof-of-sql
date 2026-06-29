//! Tests for ColumnCommitmentsMismatch error.

#[cfg(test)]
mod column_commitments_mismatch_test {
    use crate::base::commitment::ColumnCommitmentsMismatch;

    #[test]
    fn test_mismatch_num_columns_display() {
        let err = ColumnCommitmentsMismatch::NumColumns;
        assert_eq!(
            format!("{}", err),
            "commitments with different column counts cannot operate with each other"
        );
    }

    #[test]
    fn test_mismatch_num_columns_debug() {
        let err = ColumnCommitmentsMismatch::NumColumns;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NumColumns"));
    }

    #[test]
    fn test_mismatch_ident_display() {
        let err = ColumnCommitmentsMismatch::Ident {
            id_a: "col_a".to_string(),
            id_b: "col_b".to_string(),
        };
        let s = format!("{}", err);
        assert!(s.contains("col_a"));
        assert!(s.contains("col_b"));
    }

    #[test]
    fn test_mismatch_ident_debug() {
        let err = ColumnCommitmentsMismatch::Ident {
            id_a: "column1".to_string(),
            id_b: "column2".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Ident"));
        assert!(debug_str.contains("column1"));
        assert!(debug_str.contains("column2"));
    }

    #[test]
    fn test_mismatch_metadata_source() {
        let err = ColumnCommitmentsMismatch::ColumnCommitmentMetadata {
            source: crate::base::commitment::ColumnCommitmentMetadataMismatch {
                datatype_a: crate::base::database::ColumnType::BigInt,
                datatype_b: crate::base::database::ColumnType::VarChar,
            },
        };
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_mismatch_partial_eq() {
        let err1 = ColumnCommitmentsMismatch::NumColumns;
        let err2 = ColumnCommitmentsMismatch::NumColumns;
        let err3 = ColumnCommitmentsMismatch::Ident {
            id_a: "a".to_string(),
            id_b: "b".to_string(),
        };
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_ident_partial_eq() {
        let err1 = ColumnCommitmentsMismatch::Ident {
            id_a: "col1".to_string(),
            id_b: "col2".to_string(),
        };
        let err2 = ColumnCommitmentsMismatch::Ident {
            id_a: "col1".to_string(),
            id_b: "col2".to_string(),
        };
        let err3 = ColumnCommitmentsMismatch::Ident {
            id_a: "col1".to_string(),
            id_b: "col3".to_string(),
        };
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_all_variants_debug() {
        let errs = vec![
            ColumnCommitmentsMismatch::NumColumns,
            ColumnCommitmentsMismatch::Ident {
                id_a: "a".to_string(),
                id_b: "b".to_string(),
            },
        ];
        for err in errs {
            let debug_str = format!("{:?}", err);
            assert!(!debug_str.is_empty());
        }
    }
}