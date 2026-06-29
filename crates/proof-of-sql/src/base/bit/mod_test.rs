#[cfg(test)]
mod bit_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::bit::{bit_distribution, bit_matrix};
        let _ = (bit_distribution, bit_matrix);
        assert!(true);
    }
}
