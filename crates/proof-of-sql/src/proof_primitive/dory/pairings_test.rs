//! Tests for Dory pairings.

#[cfg(test)]
mod pairings_test {
    use crate::proof_primitive::dory::pairings;

    #[test]
    fn test_pairings_module_exists() {
        // Test that pairing function can be referenced
        let _ = pairings::pairing::<ark_bn254::Pairing>;
    }

    #[test]
    fn test_multi_pairing_empty() {
        // Test with empty iterators
        let empty_g1: Vec<ark_bn254::G1Prepared> = vec![];
        let empty_g2: Vec<ark_bn254::G2Prepared> = vec![];
        let result = pairings::multi_pairing::<ark_bn254::Pairing>(empty_g1, empty_g2);
        let _ = result;
    }

    #[test]
    fn test_multi_pairing_2_empty() {
        let empty: Vec<ark_bn254::G1Prepared> = vec![];
        let result = pairings::multi_pairing_2::<ark_bn254::Pairing>(
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
        );
        let _ = result;
    }

    #[test]
    fn test_multi_pairing_4_empty() {
        let empty: Vec<ark_bn254::G1Prepared> = vec![];
        let result = pairings::multi_pairing_4::<ark_bn254::Pairing>(
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
        );
        let _ = result;
    }

    #[test]
    fn test_multi_pairing_8_empty() {
        let empty: Vec<ark_bn254::G1Prepared> = vec![];
        let result = pairings::multi_pairing_8::<ark_bn254::Pairing>(
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
            (empty.clone(), empty.clone()),
        );
        let _ = result;
    }
}