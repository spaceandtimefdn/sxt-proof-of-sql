use crate::{
    base::map::IndexSet,
    sql::evm_proof_plan::{EVMProofPlanError, EVMProofPlanResult},
};
use core::iter;

pub(crate) fn try_unwrap_output_column_names(
    output_column_names: Option<&IndexSet<String>>,
    length: usize,
) -> EVMProofPlanResult<IndexSet<String>> {
    let output_column_names = match output_column_names {
        Some(output_column_names) => {
            if length > output_column_names.len() {
                return Err(EVMProofPlanError::InvalidOutputColumnName);
            }
            output_column_names.clone()
        }
        None => (0..length).map(|i| i.to_string()).collect::<IndexSet<_>>(),
    };
    Ok(output_column_names)
}

pub(crate) fn try_unwrap_output_column_names_with_count_alias(
    output_column_names: Option<&IndexSet<String>>,
    length: usize,
    count_alias: &String,
) -> EVMProofPlanResult<IndexSet<String>> {
    let output_column_names = match output_column_names {
        Some(output_column_names) => {
            if length > output_column_names.len() {
                return Err(EVMProofPlanError::InvalidOutputColumnName);
            }
            output_column_names.clone()
        }
        None => (0..length)
            .map(|i| i.to_string())
            .filter(|name| name != count_alias)
            .take(length - 1)
            .chain(iter::once(count_alias.clone()))
            .collect::<IndexSet<_>>(),
    };
    Ok(output_column_names)
}

#[cfg(test)]
mod tests {
    use crate::sql::evm_proof_plan::{
        plans::try_unwrap_output_column_names_with_count_alias, EVMProofPlanError,
    };
    use indexmap::IndexSet;

    #[test]
    fn we_can_unwrap_correct_output_column_names_when_none() {
        let output_column_names =
            try_unwrap_output_column_names_with_count_alias(None, 2, &"0".to_string()).unwrap();
        let expected_output_column_names: IndexSet<
            String,
            core::hash::BuildHasherDefault<ahash::AHasher>,
        > = vec!["1".to_string(), "0".to_string()].into_iter().collect();
        assert_eq!(output_column_names, expected_output_column_names);
    }

    #[test]
    fn we_can_unwrap_correct_output_column_names_when_some() {
        let expected_output_column_names: IndexSet<String, _> =
            vec!["a".to_string(), "b".to_string()].into_iter().collect();
        let output_column_names = try_unwrap_output_column_names_with_count_alias(
            Some(&expected_output_column_names),
            2,
            &"b".to_string(),
        )
        .unwrap();

        assert_eq!(output_column_names, expected_output_column_names);
    }

    #[test]
    fn we_can_unwrap_err_when_mismatching_count_alias() {
        let expected_output_column_names: IndexSet<String, _> =
            vec!["a".to_string(), "b".to_string()].into_iter().collect();
        let err = try_unwrap_output_column_names_with_count_alias(
            Some(&expected_output_column_names),
            3,
            &"b".to_string(),
        )
        .unwrap_err();

        assert!(matches!(err, EVMProofPlanError::InvalidOutputColumnName));
    }
}
