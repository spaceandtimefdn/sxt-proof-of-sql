#[cfg(test)]
mod encode_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::encode::{U256, ZigZag, VarInt};
        assert!(U256::ZERO.bytes.len() > 0);
        assert!(ZigZag::encode(0i64) >= 0);
        assert!(VarInt::MAX_VARINT_LEN >= 1);
    }
}
