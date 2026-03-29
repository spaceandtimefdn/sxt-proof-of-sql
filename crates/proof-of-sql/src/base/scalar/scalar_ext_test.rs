use super::ScalarExt;
use crate::base::scalar::test_scalar::TestScalar;
use std::str::FromStr;

#[test]
fn test_from_str_zero() {
    let s = TestScalar::from_str("0").expect("parse zero");
    assert_eq!(s, TestScalar::ZERO);
}

#[test]
fn test_from_str_one() {
    let s = TestScalar::from_str("1").expect("parse one");
    assert_eq!(s, TestScalar::ONE);
}

#[test]
fn test_from_str_positive_integer() {
    let s = TestScalar::from_str("42").expect("parse 42");
    assert_eq!(s, TestScalar::from(42u64));
}

#[test]
fn test_from_str_large_integer() {
    // Should parse without overflow for valid scalars
    let s = TestScalar::from_str("1000000").expect("parse large int");
    assert_eq!(s, TestScalar::from(1_000_000u64));
}

#[test]
fn test_from_str_invalid_input_returns_error() {
    assert!(TestScalar::from_str("not_a_number").is_err());
}

#[test]
fn test_from_str_empty_string_returns_error() {
    assert!(TestScalar::from_str("").is_err());
}

#[test]
fn test_from_str_negative_integer() {
    // Negative values may be supported depending on implementation
    // At minimum this should not panic
    let _result = TestScalar::from_str("-1");
}

#[test]
fn test_from_str_decimal_string() {
    // Decimal strings with scale information
    let result = TestScalar::from_str("3.14");
    // We don't mandate success, but it must not panic
    let _ = result;
}
