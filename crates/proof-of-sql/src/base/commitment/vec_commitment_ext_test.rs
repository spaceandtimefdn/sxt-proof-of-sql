//! Tests for NumColumnsMismatch error.

#[cfg(test)]
mod num_columns_mismatch_test {
    use crate::base::commitment::NumColumnsMismatch;

    #[test]
    fn num_columns_mismatch_display() {
        let err = NumColumnsMismatch;
        assert_eq!(
            format!("{}", err),
            "cannot update commitment collections with different column counts"
        );
    }

    #[test]
    fn num_columns_mismatch_debug() {
        let err = NumColumnsMismatch;
        let debug_str = format!("{:?}", err);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn num_columns_mismatch_unit_struct() {
        let err1 = NumColumnsMismatch;
        let err2 = NumColumnsMismatch;
        assert_eq!(err1, err2);
    }
}