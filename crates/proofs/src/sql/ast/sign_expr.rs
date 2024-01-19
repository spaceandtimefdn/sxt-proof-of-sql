use super::{
    is_within_acceptable_range, verify_constant_abs_decomposition,
    verify_constant_sign_decomposition,
};
use crate::{
    base::{
        bit::{compute_varying_bit_matrix, BitDistribution},
        commitment::Commitment,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, ProofBuilder, SumcheckSubpolynomialTerm, SumcheckSubpolynomialType,
        VerificationBuilder,
    },
};
use bumpalo::Bump;
use num_traits::Zero;

/// Count the number of components needed to prove a sign decomposition
pub fn count_sign(builder: &mut CountBuilder) -> Result<(), ProofError> {
    let dist = builder.consume_bit_distribution()?;
    if !is_within_acceptable_range(&dist) {
        return Err(ProofError::VerificationError(
            "bit distribution outside of acceptable range",
        ));
    }
    if dist.num_varying_bits() == 0 {
        return Ok(());
    }
    builder.count_intermediate_mles(dist.num_varying_bits());
    builder.count_subpolynomials(dist.num_varying_bits());
    builder.count_degree(3);
    if dist.has_varying_sign_bit() && dist.num_varying_bits() > 1 {
        builder.count_anchored_mles(1);
        builder.count_subpolynomials(1);
    }
    Ok(())
}

/// Compute the sign bit for a column of scalars.
///
/// todo! make this more efficient and targeted at just the sign bit rather than all bits to create a proof
pub fn result_evaluate_sign<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, expr.len());
    // bit_distribution
    let dist = BitDistribution::new::<S, _>(expr);

    // handle the constant case
    if dist.num_varying_bits() == 0 {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    // prove that the bits are binary
    let bits = compute_varying_bit_matrix(alloc, expr, &dist);
    if !dist.has_varying_sign_bit() {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    let result = bits.last().unwrap();
    assert_eq!(table_length, result.len());
    result
}

/// Prove the sign decomposition for a column of scalars.
///
/// If x1, ..., xn denotes the data, prove the column of
/// booleans, i.e. sign bits, s1, ..., sn where si == 1 if xi > MID and
/// si == 1 if xi <= MID and MID is defined in base/bit/abs_bit_mask.rs
///
/// Note: We can only prove the sign bit for non-zero scalars, and we restict
/// the range of non-zero scalar so that there is a unique sign representation.
pub fn prover_evaluate_sign<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
) -> &'a [bool] {
    let table_length = expr.len();
    // bit_distribution
    let dist = BitDistribution::new::<S, _>(expr);
    builder.produce_bit_distribution(dist.clone());

    // handle the constant case
    if dist.num_varying_bits() == 0 {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    // prove that the bits are binary
    let bits = compute_varying_bit_matrix(alloc, expr, &dist);
    prove_bits_are_binary(builder, &bits);
    if !dist.has_varying_sign_bit() {
        return alloc.alloc_slice_fill_copy(table_length, dist.sign_bit());
    }

    if dist.num_varying_bits() > 1 {
        prove_bit_decomposition(builder, alloc, expr, &bits, &dist);
    }

    bits.last().unwrap()
}

/// Verify the sign decomposition for a column of scalars.
///
/// See prover_evaluate_sign.
pub fn verifier_evaluate_sign<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    commit: &C,
    one_commit: &C,
) -> Result<C::Scalar, ProofError> {
    // bit_distribution
    let dist = builder.consume_bit_distribution();
    let num_varying_bits = dist.num_varying_bits();

    // extract evaluations and commitmens of the multilinear extensions for the varying
    // bits of the expression
    let mut bit_evals = Vec::with_capacity(num_varying_bits);
    let mut bit_commits = Vec::with_capacity(num_varying_bits);
    for _ in 0..num_varying_bits {
        let (eval, commit) = builder.consume_intermediate_mle_with_commit();
        bit_evals.push(eval);
        bit_commits.push(commit);
    }

    // establish that the bits are binary
    verify_bits_are_binary(builder, &bit_evals);

    // handle the special case of the sign bit being constant
    if !dist.has_varying_sign_bit() {
        return verifier_const_sign_evaluate(builder, &dist, commit, one_commit, &bit_commits);
    }

    // handle the special case of the absolute part being constant
    if dist.num_varying_bits() == 1 {
        verify_constant_abs_decomposition(&dist, commit, one_commit, &bit_commits[0])?;
    } else {
        verify_bit_decomposition(builder, commit, &bit_evals, &dist);
    }

    Ok(*bit_evals.last().unwrap())
}

fn verifier_const_sign_evaluate<C: Commitment>(
    builder: &VerificationBuilder<C>,
    dist: &BitDistribution,
    commit: &C,
    one_commit: &C,
    bit_commits: &[C],
) -> Result<C::Scalar, ProofError> {
    verify_constant_sign_decomposition(dist, commit, one_commit, bit_commits)?;
    if dist.sign_bit() {
        Ok(builder.mle_evaluations.one_evaluation)
    } else {
        Ok(C::Scalar::zero())
    }
}

fn prove_bits_are_binary<'a, S: Scalar>(builder: &mut ProofBuilder<'a, S>, bits: &[&'a [bool]]) {
    for &seq in bits.iter() {
        builder.produce_intermediate_mle(seq);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(seq)]),
                (-S::one(), vec![Box::new(seq), Box::new(seq)]),
            ],
        );
    }
}

fn verify_bits_are_binary<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    bit_evals: &[C::Scalar],
) {
    for bit_eval in bit_evals.iter() {
        let mut eval = *bit_eval - *bit_eval * *bit_eval;
        eval *= builder.mle_evaluations.random_evaluation;
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
}

fn prove_bit_decomposition<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    expr: &'a [S],
    bits: &[&'a [bool]],
    dist: &BitDistribution,
) {
    builder.produce_anchored_mle(expr);

    let sign_mle = bits.last().unwrap();
    let sign_mle: &[_] =
        alloc.alloc_slice_fill_with(sign_mle.len(), |i| 1 - 2 * (sign_mle[i] as i32));
    let mut terms: Vec<SumcheckSubpolynomialTerm<S>> = Vec::new();

    // expr
    terms.push((S::one(), vec![Box::new(expr)]));

    // expr bit decomposition
    let const_part = S::from(dist.constant_part());
    if !const_part.is_zero() {
        terms.push((-const_part, vec![Box::new(sign_mle)]));
    }
    let mut vary_index = 0;
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        terms.push((
            -S::from(mult),
            vec![Box::new(sign_mle), Box::new(bits[vary_index])],
        ));
        vary_index += 1;
    });
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomialType::Identity, terms);
}

fn verify_bit_decomposition<C: Commitment>(
    builder: &mut VerificationBuilder<'_, C>,
    expr_commit: &C,
    bit_evals: &[C::Scalar],
    dist: &BitDistribution,
) {
    let sign_eval = bit_evals.last().unwrap();
    let sign_eval = builder.mle_evaluations.one_evaluation - C::Scalar::TWO * *sign_eval;
    let mut vary_index = 0;
    let mut eval = builder.consume_anchored_mle(expr_commit);
    eval -= sign_eval * C::Scalar::from(dist.constant_part());
    dist.for_each_abs_varying_bit(|int_index: usize, bit_index: usize| {
        let mut mult = [0u64; 4];
        mult[int_index] = 1u64 << bit_index;
        let bit_eval = bit_evals[vary_index];
        eval -= C::Scalar::from(mult) * sign_eval * bit_eval;
        vary_index += 1;
    });
    eval *= builder.mle_evaluations.random_evaluation;
    builder.produce_sumcheck_subpolynomial_evaluation(&eval);
}
