use curve25519_dalek::scalar::Scalar;
use byte_slice_cast::AsMutByteSlice;
use ark_ff::fields::{Fp256, MontBackend, MontConfig, MontFp};
use ark_ff::BigInt;

#[derive(MontConfig)]
#[modulus = "7237005577332262213973186563042994240857116359379907606001950938285454250989"]
#[generator = "2"]
pub struct ArkScalarConfig;
pub type ArkScalar = Fp256<MontBackend<ArkScalarConfig, 4>>;

#[allow(unused_variables)]
pub fn to_ark_scalar(x: Scalar) -> ArkScalar {
    let mut values: [u64; 4] = [0; 4];
    values.as_mut_byte_slice().clone_from_slice(x.as_bytes());
    ArkScalar::new(BigInt::new(values))
}
