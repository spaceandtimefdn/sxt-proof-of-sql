//! Tests for table_evaluation.rs
use crate::base::{database::TableEvaluation, scalar::test_scalar::TestScalar};

#[test]
fn table_evaluation_new_and_accessors() {
    let evals = vec![TestScalar::from(1), TestScalar::from(2), TestScalar::from(3)];
    let chi = (TestScalar::from(42), 10);
    let te = TableEvaluation::new(evals.clone(), chi);

    assert_eq!(te.column_evals(), evals.as_slice());
    assert_eq!(te.chi_eval(), TestScalar::from(42));
    assert_eq!(te.chi(), (TestScalar::from(42), 10));
}

#[test]
fn table_evaluation_empty_column_evals() {
    let te = TableEvaluation::<TestScalar>::new(vec![], (TestScalar::from(0), 0));
    assert!(te.column_evals().is_empty());
    assert_eq!(te.chi(), (TestScalar::from(0), 0));
}

#[test]
fn table_evaluation_equality() {
    let te1 = TableEvaluation::new(vec![TestScalar::from(1)], (TestScalar::from(2), 3));
    let te2 = TableEvaluation::new(vec![TestScalar::from(1)], (TestScalar::from(2), 3));
    assert_eq!(te1, te2);
}

#[test]
fn table_evaluation_inequality() {
    let te1 = TableEvaluation::new(vec![TestScalar::from(1)], (TestScalar::from(2), 3));
    let te2 = TableEvaluation::new(vec![TestScalar::from(1)], (TestScalar::from(2), 4));
    assert_ne!(te1, te2);
}

#[test]
fn table_evaluation_clone() {
    let te = TableEvaluation::new(vec![TestScalar::from(7)], (TestScalar::from(8), 9));
    let cloned = te.clone();
    assert_eq!(te, cloned);
}
