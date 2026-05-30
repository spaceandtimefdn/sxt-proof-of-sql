use super::TableEvaluation;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn table_evaluation_preserves_column_evaluations_and_chi() {
    let column_evals = vec![TestScalar::from(3), TestScalar::from(5)];
    let chi = (TestScalar::from(8), 13);
    let evaluation = TableEvaluation::new(column_evals.clone(), chi);

    assert_eq!(evaluation.column_evals(), column_evals);
    assert_eq!(evaluation.chi_eval(), chi.0);
    assert_eq!(evaluation.chi(), chi);
}

#[test]
fn cloned_table_evaluation_preserves_values() {
    let evaluation = TableEvaluation::new(vec![TestScalar::from(2)], (TestScalar::from(7), 11));
    let cloned = evaluation.clone();

    assert_eq!(cloned, evaluation);
}
