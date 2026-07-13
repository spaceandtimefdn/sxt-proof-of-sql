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
    use crate::base::scalar::Scalar;
    use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

    #[test]
    fn we_can_multiply_curve25519_scalar_with_ristretto_point() {
        let scalar = Curve25519Scalar::from(3u64);
        let point = RISTRETTO_BASEPOINT_POINT;

        // scalar * owned_point
        let _ = scalar * point;
        // scalar * &point
        let _ = scalar * &point;
        // owned_point * scalar
        let _ = point * scalar;
        // &point * scalar
        let _ = &point * scalar;
    }

    #[test]
    fn we_can_convert_curve25519_scalar_to_dalek_scalar_by_value() {
        let scalar = Curve25519Scalar::from(7u64);
        let _dalek: curve25519_dalek::scalar::Scalar = scalar.into();
    }
}
