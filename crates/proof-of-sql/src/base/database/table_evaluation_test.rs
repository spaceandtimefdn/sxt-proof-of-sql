use super::TableEvaluation;
use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn we_can_access_table_evaluation_components() {
    let column_evals = vec![TestScalar::from(3u64), TestScalar::from(5u64)];
    let chi = (TestScalar::from(7u64), 11);
    let table_evaluation = TableEvaluation::new(column_evals.clone(), chi);

    assert_eq!(table_evaluation.column_evals(), column_evals.as_slice());
    assert_eq!(table_evaluation.chi_eval(), chi.0);
    assert_eq!(table_evaluation.chi(), chi);
}
