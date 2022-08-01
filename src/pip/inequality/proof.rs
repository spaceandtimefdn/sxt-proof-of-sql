use crate::base::{
    proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};
use crate::pip::hadamard::HadamardProof;
use curve25519_dalek::scalar::Scalar;
use std::iter;

#[derive(Debug)]
pub struct InequalityProof {
    pub c_c: Commitment,
    pub c_d: Commitment,
    pub proof_ez0: HadamardProof,
    pub proof_czd: HadamardProof,
}

impl<T> PipProve<(Column<T>, Column<T>), Column<bool>> for InequalityProof
where
    T: Clone + IntoScalar,
{
    fn prove(
        transcript: &mut Transcript,
        input: (Column<T>, Column<T>),
        output: Column<bool>,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        let a = input.0;
        let b = input.1;
        let d = output;
        let c_a = input_commitments.0;
        let c_b = input_commitments.1;
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), d.len());
        assert_eq!(a.len(), c_a.length);
        assert_eq!(a.len(), c_b.length);
        create_inequality_proof(transcript, a, b, d, c_a, c_b)
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for InequalityProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        let c_a = input_commitments.0;
        let c_b = input_commitments.1;
        verify_proof(transcript, self, c_a, c_b)
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_d
    }
}

fn create_inequality_proof<T>(
    transcript: &mut Transcript,
    a: Column<T>,
    b: Column<T>,
    d: Column<bool>,
    c_a: Commitment,
    c_b: Commitment,
) -> InequalityProof
where
    T: IntoScalar + Clone,
{
    transcript.equality_domain_sep(c_a.length as u64);
    let length = a.len();
    let (z_vec, c_vec): (Vec<Scalar>, Vec<Scalar>) = a
        .data
        .into_iter()
        .zip(b.data)
        .map(|(ai, bi)| {
            let zi = ai.into_scalar() - bi.into_scalar();
            let ci = if zi == Scalar::zero() {
                Scalar::zero()
            } else {
                zi.invert()
            };
            (zi, ci)
        })
        .unzip();

    let d_scalar: Vec<Scalar> = d.iter().map(|di| di.into_scalar()).collect();
    let e_vec: Vec<Scalar> = d_scalar.iter().map(|di| Scalar::one() - di).collect();
    let zero_vec: Vec<Scalar> = iter::repeat(Scalar::zero()).take(length).collect();
    let one_vec: Vec<Scalar> = iter::repeat(Scalar::one()).take(length).collect();
    let c_1 = Commitment::from(&one_vec[..]);

    let c_c = Commitment::from(&c_vec[..]);
    let c_d = Commitment::from(d_scalar.as_slice());
    let c_e = c_1 - c_d;
    let c_z = c_a - c_b;
    transcript.append_point(b"c_c", &c_c.commitment);
    transcript.append_point(b"c_d", &c_d.commitment);
    let proof_ez0 = HadamardProof::prove(
        transcript,
        (e_vec.into(), z_vec.clone().into()),
        zero_vec.into(),
        (c_e, c_z),
    );
    let proof_czd = HadamardProof::prove(
        transcript,
        (c_vec.into(), z_vec.into()),
        d_scalar.into(),
        (c_c, c_z),
    );

    InequalityProof {
        c_c,
        c_d,
        proof_ez0,
        proof_czd,
    }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &InequalityProof,
    c_a: Commitment,
    c_b: Commitment,
) -> Result<(), ProofError> {
    transcript.equality_domain_sep(c_a.length as u64);
    // Computing c_0 and c_1 here is terrible. It should be cached.
    let (zero, one): (Vec<Scalar>, Vec<Scalar>) = iter::repeat((Scalar::zero(), Scalar::one()))
        .take(c_a.length)
        .unzip();
    let c_0 = Commitment::from(&zero[..]);
    let c_1 = Commitment::from(&one[..]);
    let c_e = c_1 - proof.c_d;
    let c_z = c_a - c_b;
    transcript.append_point(b"c_c", &proof.c_c.commitment);
    transcript.append_point(b"c_d", &proof.c_d.commitment);
    proof.proof_ez0.verify(transcript, (c_e, c_z))?;
    proof.proof_czd.verify(transcript, (proof.c_c, c_z))?;
    if proof.proof_ez0.commit_ab != c_0 || proof.proof_czd.commit_ab != proof.c_d {
        Err(ProofError::VerificationError)
    } else {
        Ok(())
    }
}
