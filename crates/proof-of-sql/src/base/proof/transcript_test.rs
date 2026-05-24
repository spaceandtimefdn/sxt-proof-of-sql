//! Tests for Transcript.

#[cfg(test)]
mod transcript_test {
    use crate::base::proof::transcript::Transcript;

    #[test]
    fn test_transcript_new() {
        let t = Transcript::new();
        let debug_str = format!("{:?}", t);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_transcript_type_exists() {
        let _: Option<Transcript> = None;
    }
}
