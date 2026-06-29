//! Tests for EVMProofPlanError.

#[cfg(test)]
mod evm_proof_plan_error_test {
    use crate::sql::evm_proof_plan::{EVMProofPlanError, EVMProofPlanResult};
    use crate::sql::AnalyzeError;
    use snafu::Snafu;

    #[test]
    fn test_error_not_supported() {
        let err = EVMProofPlanError::NotSupported;
        assert_eq!(err.to_string(), "plan not yet supported");
    }

    #[test]
    fn test_error_column_not_found() {
        let err = EVMProofPlanError::ColumnNotFound;
        assert_eq!(err.to_string(), "column not found");
    }

    #[test]
    fn test_error_table_not_found() {
        let err = EVMProofPlanError::TableNotFound;
        assert_eq!(err.to_string(), "table not found");
    }

    #[test]
    fn test_error_invalid_table_name() {
        let err = EVMProofPlanError::InvalidTableName;
        assert_eq!(err.to_string(), "table name can not be parsed into TableRef");
    }

    #[test]
    fn test_error_invalid_output_column_name() {
        let err = EVMProofPlanError::InvalidOutputColumnName;
        assert_eq!(err.to_string(), "invalid or missing output column name");
    }

    #[test]
    fn test_error_inconsistent_group_by_column_counts() {
        let err = EVMProofPlanError::InconsistentGroupByColumnCounts;
        assert_eq!(err.to_string(), "column counts in group by plans are inconsistent");
    }

    #[test]
    fn test_error_incorrect_scaling_factor() {
        let err = EVMProofPlanError::IncorrectScalingFactor;
        assert_eq!(err.to_string(), "incorrect scaling factor");
    }

    #[test]
    fn test_error_analyze_error() {
        #[derive(Snafu, Debug)]
        enum TestError {
            #[snafu(display("test error"))]
            Test,
        }
        let analyze_err = AnalyzeError::NotSupported {
            feature: "test".to_string(),
        };
        let err = EVMProofPlanError::AnalyzeError { source: analyze_err };
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_partial_eq() {
        let err1 = EVMProofPlanError::NotSupported;
        let err2 = EVMProofPlanError::NotSupported;
        let err3 = EVMProofPlanError::ColumnNotFound;
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_debug() {
        let err = EVMProofPlanError::NotSupported;
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_result_type() {
        let result: EVMProofPlanResult<i32> = Err(EVMProofPlanError::NotSupported);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EVMProofPlanError::NotSupported);
    }

    #[test]
    fn test_error_description() {
        let errors = vec![
            EVMProofPlanError::NotSupported,
            EVMProofPlanError::ColumnNotFound,
            EVMProofPlanError::TableNotFound,
            EVMProofPlanError::InvalidTableName,
            EVMProofPlanError::InvalidOutputColumnName,
            EVMProofPlanError::InconsistentGroupByColumnCounts,
            EVMProofPlanError::IncorrectScalingFactor,
        ];
        for err in errors {
            let s = err.to_string();
            assert!(!s.is_empty(), "Error should have a description");
        }
    }
}