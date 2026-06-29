#[cfg(test)]
mod standard_serializations_mod_test {
    // Module re-exports: binary, limbs
    #[test]
    fn test_module_re_exports() {
        use crate::base::standard_serializations::binary;
        use crate::base::standard_serializations::limbs;
        let _ = (binary::SERIALIZER_VERSION);
        assert!(true);
    }
}
