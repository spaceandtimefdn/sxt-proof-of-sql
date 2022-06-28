use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use pedersen::commitments::compute_commitments;
use pedersen::commitments::get_generators;
use std::cmp;
use std::slice;

use crate::base::math::{is_pow2, log2_up};
use crate::base::polynomial::CompositePolynomialInfo;
use crate::base::proof::{Commitment, PIPProof, ProofError, Transcript};
use crate::base::scalar::inner_product;
use crate::pip::hadamard::{compute_evaluation_vector, make_sumcheck_polynomial};
use crate::proof_primitive::inner_product::InnerProductProof;
use crate::proof_primitive::sumcheck::SumcheckProof;

#[derive(Clone, Debug)]
pub struct HadamardProof {
    pub commit_ab: Commitment,
    pub sumcheck_proof: SumcheckProof,
    pub f_a: Scalar,
    pub f_a_proof: InnerProductProof,
    pub f_b: Scalar,
    pub f_b_proof: InnerProductProof,
    pub f_ab_proof: InnerProductProof,
}

impl PIPProof for HadamardProof {
    /// Create a hadamard proof.
    ///
    /// See protocols/multiplication.pdf
    fn create(
        transcript: &mut Transcript,
        input_columns: &[&[Scalar]],
        output_columns: &[&[Scalar]],
        input_commitments: &[Commitment],
    ) -> Self {
        assert_eq!(input_columns.len(), 2);
        assert_eq!(input_commitments.len(), 2);
        assert_eq!(output_columns.len(), 1);
        let a_vec = input_columns[0];
        let b_vec = input_columns[1];
        let ab_vec = output_columns[0];

        let n = a_vec.len();
        assert!(n > 0);
        assert_eq!(a_vec.len(), n);
        assert_eq!(b_vec.len(), n);
        assert_eq!(ab_vec.len(), n);

        let num_vars = compute_num_variables(n);
        if is_pow2(n) && n > 1 {
            return create_proof_impl(transcript, a_vec, b_vec, ab_vec, num_vars, n);
        }
        let n_p = 1 << num_vars;
        let a_vec = extend_scalar_vector(a_vec, n_p);
        let b_vec = extend_scalar_vector(b_vec, n_p);
        let ab_vec = extend_scalar_vector(ab_vec, n_p);
        create_proof_impl(transcript, &a_vec, &b_vec, &ab_vec, num_vars, n)
    }

    /// Verifies that a hadamard proof is correct given the associated commitments.
    fn verify(&self, transcript: &mut Transcript, inputs: &[Commitment]) -> Result<(), ProofError> {
        assert_eq!(inputs.len(), 2);
        let commit_a = inputs[0].commitment;
        let commit_b = inputs[1].commitment;
        assert_eq!(inputs[0].length, inputs[1].length);
        let n = inputs[0].length;

        let num_vars = compute_num_variables(n);

        let n = 1 << num_vars;
        transcript.hadamard_domain_sep(num_vars as u64);
        transcript.append_point(b"c_ab", &self.commit_ab.commitment);
        let mut r_vec = vec![Scalar::from(0u64); n];
        transcript.challenge_scalars(&mut r_vec, b"r_vec");

        let polynomial_info = CompositePolynomialInfo {
            max_multiplicands: 3,
            num_variables: num_vars,
        };
        let subclaim = self.sumcheck_proof.verify_without_evaluation(
            transcript,
            polynomial_info,
            &Scalar::zero(),
        )?;
        let evaluation_vec = compute_evaluation_vector(&subclaim.evaluation_point);
        let f_r = inner_product(&r_vec, &evaluation_vec);

        // subclam.expected_evaluation == f_r * (f_a * f_b - f_ab) or
        // f_ab == f_a * f_b - subclam.expected_evaluation / f_r
        if f_r == Scalar::zero() {
            // Note: This happens with probability nearly zero
            return Ok(());
        }
        let f_ab = self.f_a * self.f_b - subclaim.expected_evaluation * f_r.invert();

        let mut generators = vec![CompressedRistretto::identity(); n + 1];
        get_generators(&mut generators, 0);
        let generators: Vec<RistrettoPoint> = generators
            .iter()
            .map(|&x| x.decompress().unwrap())
            .collect();
        let product_g = generators[n];

        // verify f_a
        let f_commit = commit_a.decompress().unwrap() + self.f_a * product_g;
        self.f_a_proof.verify(
            transcript,
            &f_commit,
            &product_g,
            &generators[0..n],
            &evaluation_vec,
        )?;

        // verify f_b
        let f_commit = commit_b.decompress().unwrap() + self.f_b * product_g;
        self.f_b_proof.verify(
            transcript,
            &f_commit,
            &product_g,
            &generators[0..n],
            &evaluation_vec,
        )?;

        // verify f_ab
        let f_commit = self.commit_ab.commitment.decompress().unwrap() + f_ab * product_g;
        self.f_ab_proof.verify(
            transcript,
            &f_commit,
            &product_g,
            &generators[0..n],
            &evaluation_vec,
        )?;

        Ok(())
    }

    fn get_output_commitments(&self) -> &[Commitment] {
        return slice::from_ref(&self.commit_ab);
    }
}

fn compute_num_variables(n: usize) -> usize {
    // Note: This isn't a space efficient way of handling
    // the case n == 1, but keeping it simple for the first iteration
    cmp::max(log2_up(n), 1)
}

fn extend_scalar_vector(a_vec: &[Scalar], n: usize) -> Vec<Scalar> {
    let mut vec = Vec::with_capacity(n);
    for i in 0..a_vec.len() {
        vec.push(a_vec[i]);
    }
    for _ in a_vec.len()..n {
        vec.push(Scalar::from(0u64));
    }
    vec
}

fn create_proof_impl(
    transcript: &mut Transcript,
    a_vec: &[Scalar],
    b_vec: &[Scalar],
    ab_vec: &[Scalar],
    num_vars: usize,
    length: usize,
) -> HadamardProof {
    transcript.hadamard_domain_sep(num_vars as u64);
    let n = a_vec.len();

    let mut c_ab = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut c_ab), &[ab_vec]);
    transcript.append_point(b"c_ab", &c_ab);

    let mut r_vec = vec![Scalar::zero(); n];
    transcript.challenge_scalars(&mut r_vec, b"r_vec");

    let poly = make_sumcheck_polynomial(num_vars, a_vec, b_vec, ab_vec, &r_vec);
    let mut evaluation_point = vec![Scalar::zero(); poly.num_variables];
    let sumcheck_proof = SumcheckProof::create(transcript, &mut evaluation_point, &poly);

    let evaluation_vec = compute_evaluation_vector(&evaluation_point);
    let mut generators = vec![CompressedRistretto::identity(); n + 1];
    get_generators(&mut generators, 0);
    let generators: Vec<RistrettoPoint> = generators
        .iter()
        .map(|&x| x.decompress().unwrap())
        .collect();
    let product_g = generators[n];

    let f_a = inner_product(&evaluation_vec, a_vec);
    let f_a_proof = InnerProductProof::create(
        transcript,
        &product_g,
        &generators[0..n],
        a_vec,
        &evaluation_vec,
    );

    let f_b = inner_product(&evaluation_vec, b_vec);
    let f_b_proof = InnerProductProof::create(
        transcript,
        &product_g,
        &generators[0..n],
        b_vec,
        &evaluation_vec,
    );

    let f_ab_proof = InnerProductProof::create(
        transcript,
        &product_g,
        &generators[0..n],
        ab_vec,
        &evaluation_vec,
    );

    HadamardProof {
        commit_ab: Commitment {
            commitment: c_ab,
            length: length,
        },
        sumcheck_proof: sumcheck_proof,
        f_a: f_a,
        f_a_proof: f_a_proof,
        f_b: f_b,
        f_b_proof: f_b_proof,
        f_ab_proof: f_ab_proof,
    }
}
