#[cfg(test)]
mod proof_primitive_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::proof_primitive::{dory, hyperkzg, inner_product, sumcheck};
        let _ = (dory, hyperkzg, inner_product, sumcheck);
        assert!(true);
    }
}
