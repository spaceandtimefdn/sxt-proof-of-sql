use super::TableEvaluation;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn table_evaluation_exposes_its_column_and_chi_evaluations() {
    let column_evals = vec![
        TestScalar::from(2_u64),
        TestScalar::from(3_u64),
        TestScalar::from(5_u64),
    ];
    let chi = (TestScalar::from(7_u64), 11);

    let evaluation = TableEvaluation::new(column_evals.clone(), chi);

    assert_eq!(evaluation.column_evals(), column_evals);
    assert_eq!(evaluation.chi_eval(), chi.0);
    assert_eq!(evaluation.chi(), chi);
}

#[test]
fn table_evaluation_supports_tables_without_columns() {
    let chi = (TestScalar::from(1_u64), 0);

    let evaluation = TableEvaluation::new(Vec::new(), chi);

    assert!(evaluation.column_evals().is_empty());
    assert_eq!(evaluation.chi(), chi);
}
