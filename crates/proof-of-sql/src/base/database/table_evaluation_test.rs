use crate::base::{
    database::TableEvaluation,
    scalar::{test_scalar::TestScalar, Scalar},
};

#[test]
fn we_can_create_a_table_evaluation() {
    let column_evals = vec![TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)];
    let chi = (TestScalar::from(5), 10);
    let eval = TableEvaluation::new(column_evals.clone(), chi);
    assert_eq!(eval.column_evals(), &column_evals);
    assert_eq!(eval.chi_eval(), TestScalar::from(5));
    assert_eq!(eval.chi(), (TestScalar::from(5), 10));
}

#[test]
fn we_can_create_an_empty_table_evaluation() {
    let column_evals: Vec<TestScalar> = vec![];
    let chi = (TestScalar::ZERO, 0);
    let eval = TableEvaluation::new(column_evals, chi);
    assert!(eval.column_evals().is_empty());
    assert_eq!(eval.chi_eval(), TestScalar::ZERO);
    assert_eq!(eval.chi(), (TestScalar::ZERO, 0));
}

#[test]
fn table_evaluation_clone_produces_equal_instance() {
    let column_evals = vec![TestScalar::from(42)];
    let chi = (TestScalar::ONE, 1);
    let eval = TableEvaluation::new(column_evals, chi);
    let cloned = eval.clone();
    assert_eq!(eval, cloned);
}

#[test]
fn table_evaluation_debug_does_not_panic() {
    let eval = TableEvaluation::new(vec![TestScalar::ONE], (TestScalar::ONE, 1));
    let debug_str = format!("{eval:?}");
    assert!(!debug_str.is_empty());
}

#[test]
fn table_evaluations_with_different_column_evals_are_not_equal() {
    let eval1 = TableEvaluation::new(
        vec![TestScalar::from(1)],
        (TestScalar::ONE, 1),
    );
    let eval2 = TableEvaluation::new(
        vec![TestScalar::from(2)],
        (TestScalar::ONE, 1),
    );
    assert_ne!(eval1, eval2);
}

#[test]
fn table_evaluations_with_different_chi_are_not_equal() {
    let eval1 = TableEvaluation::new(
        vec![TestScalar::ONE],
        (TestScalar::ONE, 1),
    );
    let eval2 = TableEvaluation::new(
        vec![TestScalar::ONE],
        (TestScalar::TWO, 1),
    );
    assert_ne!(eval1, eval2);
}

#[test]
fn table_evaluations_with_different_chi_length_are_not_equal() {
    let eval1 = TableEvaluation::new(
        vec![TestScalar::ONE],
        (TestScalar::ONE, 1),
    );
    let eval2 = TableEvaluation::new(
        vec![TestScalar::ONE],
        (TestScalar::ONE, 2),
    );
    assert_ne!(eval1, eval2);
}
