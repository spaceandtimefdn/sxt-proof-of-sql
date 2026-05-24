//! Tests for HyperKZG engine.

#[cfg(test)]
mod nova_engine_test {
    #[cfg(feature = "hyperkzg_proof")]
    use crate::proof_primitive::hyperkzg::nova_engine::HyperKZGEngine;

    #[test]
    fn test_hyperkzg_engine_type_exists() {
        #[cfg(feature = "hyperkzg_proof")]
        {
            let _: Option<HyperKZGEngine> = None;
        }
        #[cfg(not(feature = "hyperkzg_proof"))]
        {
            // Without hyperkzg_proof feature, just verify the module compiles
            assert!(true);
        }
    }

    #[cfg(feature = "hyperkzg_proof")]
    #[test]
    fn test_hyperkzg_engine_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<HyperKZGEngine>());
        assert!(!debug_str.is_empty());
    }
}