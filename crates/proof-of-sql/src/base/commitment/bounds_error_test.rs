use crate::base::commitment::NegativeBounds;

#[test]
fn negative_bounds_error_display() {
    let err = NegativeBounds;
    assert_eq!(format!("{}"), "cannot construct bounds where min is greater than max");
}

#[test]
fn negative_bounds_debug_formatting() {
    let err = NegativeBounds;
    assert!(format!("{:?}").contains("NegativeBounds"));
}

#[test]
fn negative_bounds_unit_struct() {
    // Unit struct errors are equal if they're the same type
    let err1 = NegativeBounds;
    let err2 = NegativeBounds;
    assert_eq!(err1, err2);
}
