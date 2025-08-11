// This code is adapted from the blitzar-rs project:
// https://github.com/spaceandtimefdn/blitzar-rs/blob/main/src/compute/arkworks_halo2_interop.rs

use ark_bn254::{Fq as Bn254Fq, G1Affine as Bn254G1Affine};
use ark_ff::BigInteger256;
use core::mem;
use halo2curves::{
    bn256::{Fq as Halo2Bn256Fq, G1Affine as Halo2Bn256G1Affine},
    serde::SerdeObject,
};

fn convert_halo2_to_limbs(point: &Halo2Bn256Fq) -> [u64; 4] {
    let limbs: [u64; 4] = unsafe { mem::transmute(*point) };
    limbs
}

/// Converts a Halo2 BN256 G1 Affine point to an Arkworks BN254 G1 Affine point.
pub fn convert_to_ark_bn254_g1_affine(point: &Halo2Bn256G1Affine) -> Bn254G1Affine {
    let x_limbs: [u64; 4] = convert_halo2_to_limbs(&point.x);
    let y_limbs: [u64; 4] = convert_halo2_to_limbs(&point.y);

    Bn254G1Affine {
        x: Bn254Fq::new_unchecked(BigInteger256::new(x_limbs)),
        y: Bn254Fq::new_unchecked(BigInteger256::new(y_limbs)),
        infinity: *point == Halo2Bn256G1Affine::default(),
    }
}

/// Converts an Arkworks BN254 G1 Affine point to a Halo2 BN256 G1 Affine point.
pub fn convert_to_halo2_bn256_g1_affine(point: &Bn254G1Affine) -> Halo2Bn256G1Affine {
    if point.infinity {
        return Halo2Bn256G1Affine::default();
    }

    let x_bytes = bytemuck::cast::<[u64; 4], [u8; 32]>(point.x.0 .0);
    let y_bytes = bytemuck::cast::<[u64; 4], [u8; 32]>(point.y.0 .0);

    Halo2Bn256G1Affine {
        x: Halo2Bn256Fq::from_raw_bytes_unchecked(&x_bytes),
        y: Halo2Bn256Fq::from_raw_bytes_unchecked(&y_bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2curves::bn256::Fq as Halo2Bn256Fq;

    #[test]
    fn test_convert_halo2_modulus_to_limbs() {
        let expected: [u64; 4] = [
            4_332_616_871_279_656_263,
            10_917_124_144_477_883_021,
            13_281_191_951_274_694_749,
            3_486_998_266_802_970_665,
        ];
        let modulus = Halo2Bn256Fq::from_raw(expected);
        let point = convert_halo2_to_limbs(&modulus);
        assert_eq!(point, [0, 0, 0, 0]);
    }

    #[test]
    fn test_convert_halo2_one_to_one_in_montgomery_form_in_limbs() {
        let one: [u64; 4] = [1, 0, 0, 0];
        let one_in_mont = Halo2Bn256Fq::from_raw(one);
        let point = convert_halo2_to_limbs(&one_in_mont);

        let expected: [u64; 4] = [
            15_230_403_791_020_821_917,
            754_611_498_739_239_741,
            7_381_016_538_464_732_716,
            1_011_752_739_694_698_287,
        ];

        assert_eq!(point, expected);
    }
}
