use crate::base::scalar::MontScalar;
use ark_ff::PrimeField;

/// A wrapper type around the field element `ark_curve25519::Fr` and should be used in place of `ark_curve25519::Fr`.
///
/// Using the `Scalar` trait rather than this type is encouraged to allow for easier switching of the underlying field.
pub type Curve25519Scalar = MontScalar<ark_curve25519::FrConfig>;

impl From<Curve25519Scalar> for curve25519_dalek::scalar::Scalar {
    fn from(value: Curve25519Scalar) -> Self {
        (&value).into()
    }
}

impl From<&Curve25519Scalar> for curve25519_dalek::scalar::Scalar {
    ///
    /// # Panics
    ///
    /// This method will panic if the byte array is not of the expected length (32 bytes) or if it cannot be converted to a valid canonical scalar. However, under normal conditions, valid `Curve25519Scalar` values should always satisfy these requirements.
    fn from(value: &Curve25519Scalar) -> Self {
        let bytes = ark_ff::BigInteger::to_bytes_le(&value.0.into_bigint());
        curve25519_dalek::scalar::Scalar::from_canonical_bytes(bytes.try_into().unwrap()).unwrap()
    }
}

impl core::ops::Mul<curve25519_dalek::ristretto::RistrettoPoint> for Curve25519Scalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}

impl core::ops::Mul<&curve25519_dalek::ristretto::RistrettoPoint> for Curve25519Scalar {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: &curve25519_dalek::ristretto::RistrettoPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}

impl core::ops::Mul<Curve25519Scalar> for curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}

impl core::ops::Mul<Curve25519Scalar> for &curve25519_dalek::ristretto::RistrettoPoint {
    type Output = curve25519_dalek::ristretto::RistrettoPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::Curve25519Scalar;

    #[test]
    fn zero_converts_to_dalek_zero() {
        let c = Curve25519Scalar::from(0i32);
        let d = curve25519_dalek::scalar::Scalar::from(c);
        assert_eq!(d, curve25519_dalek::scalar::Scalar::ZERO);
    }

    #[test]
    fn nonzero_converts_to_nonzero_dalek() {
        let c = Curve25519Scalar::from(1i32);
        let d = curve25519_dalek::scalar::Scalar::from(c);
        assert_ne!(d, curve25519_dalek::scalar::Scalar::ZERO);
    }

    #[test]
    fn reference_and_value_conversions_agree() {
        let c = Curve25519Scalar::from(42i32);
        let by_ref = curve25519_dalek::scalar::Scalar::from(&c);
        let by_val = curve25519_dalek::scalar::Scalar::from(c);
        assert_eq!(by_ref, by_val);
    }

    #[test]
    fn distinct_scalars_convert_to_distinct_dalek() {
        let a = Curve25519Scalar::from(1i32);
        let b = Curve25519Scalar::from(2i32);
        let da = curve25519_dalek::scalar::Scalar::from(a);
        let db = curve25519_dalek::scalar::Scalar::from(b);
        assert_ne!(da, db);
    }

    #[test]
    fn scalar_mul_ristretto_basepoint_does_not_panic() {
        use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
        let c = Curve25519Scalar::from(2i32);
        let _result = c * RISTRETTO_BASEPOINT_POINT;
    }

    #[test]
    fn ristretto_mul_scalar_does_not_panic() {
        use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
        let c = Curve25519Scalar::from(3i32);
        let _result = RISTRETTO_BASEPOINT_POINT * c;
    }
}
