/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::interpolate_uni_poly;
use crate::base::proof::ProofError;

pub struct Subclaim {
    pub evaluation_point: Vec<Scalar>,
    pub expected_evaluation: Scalar,
}

impl Subclaim {
    #[allow(unused_variables)]
    pub fn create(
        evaluation_point: Vec<Scalar>,
        evaluations: &Vec<Vec<Scalar>>,
        max_multiplicands: usize,
        claimed_sum: &Scalar,
    ) -> Result<Subclaim, ProofError> {
        let num_vars = evaluation_point.len();
        assert!(max_multiplicands > 0);
        assert_eq!(num_vars, evaluations.len());
        let mut expected_sum = claimed_sum.clone();
        for round_index in 0..num_vars {
            let round_evaluation = &evaluations[round_index];
            if round_evaluation.len() != max_multiplicands + 1 {
                return Err(ProofError::VerificationError);
            }
            if expected_sum != round_evaluation[0] + round_evaluation[1] {
                return Err(ProofError::VerificationError);
            }
            expected_sum = interpolate_uni_poly(round_evaluation, evaluation_point[round_index]);
        }
        Ok(Subclaim {
            evaluation_point: evaluation_point,
            expected_evaluation: expected_sum,
        })
    }
}
