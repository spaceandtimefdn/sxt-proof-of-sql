#[cfg(test)]
mod base_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::{
            arrow, bit, byte, commitment, database, encode, math, polynomial,
            posql_time, proof, scalar, map, slice_ops, rayon_cfg, ref_into, serialize,
        };
        let _ = (arrow, bit, byte, commitment, database, encode, math, polynomial,
                 posql_time, proof, scalar, map, slice_ops, rayon_cfg, ref_into, serialize);
        assert!(true);
    }
}
