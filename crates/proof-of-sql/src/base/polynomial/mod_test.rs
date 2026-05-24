#[cfg(test)]
mod polynomial_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::polynomial::{CompositePolynomial, MultilinearExtension};
        let _ = CompositePolynomial::default();
        let _ = MultilinearExtension::default();
    }
}
