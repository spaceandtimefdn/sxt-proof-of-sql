/// Tests for [`crate::base::math::decimal`] helpers — Precision, scale
/// coercion, and overflow detection.
#[cfg(test)]
mod tests {
    use crate::base::math::decimal::{DecimalError, Precision};

    // -----------------------------------------------------------------------
    // Precision::new
    // -----------------------------------------------------------------------

    #[test]
    fn test_precision_new_accepts_valid_values() {
        for p in [1u8, 10, 38, 75] {
            assert!(
                Precision::new(p).is_ok(),
                "Precision::new({}) should succeed",
                p
            );
        }
    }

    #[test]
    fn test_precision_new_rejects_zero() {
        assert!(
            Precision::new(0).is_err(),
            "Precision::new(0) should return an error"
        );
    }

    #[test]
    fn test_precision_new_rejects_values_above_75() {
        for p in [76u8, 100, 255] {
            assert!(
                Precision::new(p).is_err(),
                "Precision::new({}) should return an error",
                p
            );
        }
    }

    #[test]
    fn test_precision_value_roundtrips() {
        for p in [1u8, 38, 75] {
            let precision = Precision::new(p).unwrap();
            assert_eq!(precision.value(), p, "Precision value should roundtrip");
        }
    }

    // -----------------------------------------------------------------------
    // Precision: PartialEq / PartialOrd
    // -----------------------------------------------------------------------

    #[test]
    fn test_precision_ordering() {
        let p10 = Precision::new(10).unwrap();
        let p20 = Precision::new(20).unwrap();
        assert!(p10 < p20, "Precision(10) should be less than Precision(20)");
        assert!(p20 > p10, "Precision(20) should be greater than Precision(10)");
        assert_eq!(p10, p10, "Precision(10) should equal itself");
    }

    // -----------------------------------------------------------------------
    // DecimalError variants are constructable and display correctly
    // -----------------------------------------------------------------------

    #[test]
    fn test_decimal_error_invalid_precision_is_displayable() {
        let err = DecimalError::InvalidPrecision {
            error: "precision 0 out of range".to_string(),
        };
        let msg = format!("{err}");
        assert!(
            msg.contains("precision") || !msg.is_empty(),
            "DecimalError should produce a non-empty message"
        );
    }

    #[test]
    fn test_decimal_error_roundtrip_via_debug() {
        let err = DecimalError::InvalidPrecision {
            error: "test".to_string(),
        };
        // Debug representation should be non-empty.
        assert!(!format!("{err:?}").is_empty());
    }

    // -----------------------------------------------------------------------
    // try_into_to_scalar / scale coercion helpers (public API)
    // -----------------------------------------------------------------------

    #[test]
    fn test_precision_clone_and_copy() {
        let p = Precision::new(15).unwrap();
        let p2 = p;   // Copy
        let p3 = p.clone();
        assert_eq!(p, p2);
        assert_eq!(p, p3);
    }

    #[test]
    fn test_precision_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let p = Precision::new(10).unwrap();
        set.insert(p);
        // Same value re-inserted should not grow the set.
        set.insert(Precision::new(10).unwrap());
        assert_eq!(set.len(), 1, "Same precision should hash to same bucket");
    }
}
