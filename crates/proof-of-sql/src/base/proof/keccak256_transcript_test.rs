//! Tests for Keccak256Transcript.

#[cfg(test)]
mod keccak256_transcript_test {
    use crate::base::proof::Keccak256Transcript;

    #[test]
    fn test_keccak256_transcript_new() {
        let t = Keccak256Transcript::new();
        let debug_str = format!("{:?}", t);
        assert!(!debug_str.is_empty());
    }
}
