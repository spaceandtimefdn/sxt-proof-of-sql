use crate::{
    base::{database::Column, scalar::Scalar},
    sql::proof::ProvableResultElement,
};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, length: u64) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], length: u64) -> usize;
}

impl<'a, T: ProvableResultElement<'a>> ProvableResultColumn for &[T] {
    fn num_bytes(&self, length: u64) -> usize {
        assert_eq!(self.len() as u64, length);
        self.iter().map(ProvableResultElement::required_bytes).sum()
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        let mut res = 0;
        for i in 0..length {
            let index: usize = usize::try_from(i).expect("Index out of bounds");
            res += self[index].encode(&mut out[res..]);
        }
        res
    }
}

impl<S: Scalar> ProvableResultColumn for Column<'_, S> {
    fn num_bytes(&self, length: u64) -> usize {
        match self {
            Column::Boolean(col) => col.num_bytes(length),
            Column::Uint8(col) => col.num_bytes(length),
            Column::TinyInt(col) => col.num_bytes(length),
            Column::SmallInt(col) => col.num_bytes(length),
            Column::Int(col) => col.num_bytes(length),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col.num_bytes(length),
            Column::Int128(col) => col.num_bytes(length),
            Column::Decimal75(_, _, col) | Column::Scalar(col) => col.num_bytes(length),
            Column::VarChar((col, _)) => col.num_bytes(length),
            Column::VarBinary((col, _)) => col.num_bytes(length),
        }
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        match self {
            Column::Boolean(col) => col.write(out, length),
            Column::Uint8(col) => col.write(out, length),
            Column::TinyInt(col) => col.write(out, length),
            Column::SmallInt(col) => col.write(out, length),
            Column::Int(col) => col.write(out, length),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col.write(out, length),
            Column::Int128(col) => col.write(out, length),
            Column::Decimal75(_, _, col) | Column::Scalar(col) => col.write(out, length),
            Column::VarChar((col, _)) => col.write(out, length),
            Column::VarBinary((col, _)) => col.write(out, length),
        }
    }
}

impl<'a, T: ProvableResultElement<'a>, const N: usize> ProvableResultColumn for [T; N] {
    fn num_bytes(&self, length: u64) -> usize {
        (&self[..]).num_bytes(length)
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        (&self[..]).write(out, length)
    }
}

#[cfg(test)]
mod tests {
    use super::ProvableResultColumn;
    use crate::base::{
        database::Column,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };
    use alloc::vec::Vec;

    fn encoded(column: &impl ProvableResultColumn, length: u64) -> Vec<u8> {
        let mut out = vec![0_u8; column.num_bytes(length)];
        let written = column.write(&mut out, length);
        assert_eq!(written, out.len());
        out
    }

    #[test]
    fn we_can_serialize_array_result_columns_like_slices() {
        let values = [3_i64, -5, 8];
        assert_eq!(encoded(&values, 3), encoded(&values.as_slice(), 3));
    }

    #[test]
    fn we_can_serialize_numeric_database_columns_like_their_slices() {
        let bool_values = [true, false, true];
        assert_eq!(
            encoded(&Column::<TestScalar>::Boolean(&bool_values), 3),
            encoded(&bool_values.as_slice(), 3)
        );

        let u8_values = [1_u8, 127, 255];
        assert_eq!(
            encoded(&Column::<TestScalar>::Uint8(&u8_values), 3),
            encoded(&u8_values.as_slice(), 3)
        );

        let i8_values = [-2_i8, 0, 5];
        assert_eq!(
            encoded(&Column::<TestScalar>::TinyInt(&i8_values), 3),
            encoded(&i8_values.as_slice(), 3)
        );

        let i16_values = [-300_i16, 0, 700];
        assert_eq!(
            encoded(&Column::<TestScalar>::SmallInt(&i16_values), 3),
            encoded(&i16_values.as_slice(), 3)
        );

        let i32_values = [-70_000_i32, 0, 90_000];
        assert_eq!(
            encoded(&Column::<TestScalar>::Int(&i32_values), 3),
            encoded(&i32_values.as_slice(), 3)
        );

        let i64_values = [-9_000_000_000_i64, 0, 9_000_000_000];
        assert_eq!(
            encoded(&Column::<TestScalar>::BigInt(&i64_values), 3),
            encoded(&i64_values.as_slice(), 3)
        );
        assert_eq!(
            encoded(
                &Column::<TestScalar>::TimestampTZ(
                    PoSQLTimeUnit::Second,
                    PoSQLTimeZone::utc(),
                    &i64_values,
                ),
                3,
            ),
            encoded(&i64_values.as_slice(), 3)
        );

        let i128_values = [-100_000_000_000_i128, 0, 100_000_000_000];
        assert_eq!(
            encoded(&Column::<TestScalar>::Int128(&i128_values), 3),
            encoded(&i128_values.as_slice(), 3)
        );
    }

    #[test]
    fn we_can_serialize_scalar_backed_database_columns_like_their_slices() {
        let scalars = [
            TestScalar::from(7),
            TestScalar::from(11),
            TestScalar::from(13),
        ];
        assert_eq!(
            encoded(&Column::Scalar(&scalars), 3),
            encoded(&scalars.as_slice(), 3)
        );
        assert_eq!(
            encoded(
                &Column::Decimal75(Precision::new(20).unwrap(), 4, &scalars),
                3,
            ),
            encoded(&scalars.as_slice(), 3)
        );
    }

    #[test]
    fn we_can_serialize_variable_width_database_columns_like_their_values() {
        let strings = ["alpha", "", "gamma"];
        let string_scalars = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        assert_eq!(
            encoded(&Column::VarChar((&strings, &string_scalars)), 3),
            encoded(&strings.as_slice(), 3)
        );

        let binary_values: [&[u8]; 3] = [b"abc".as_slice(), b"".as_slice(), b"\x00\xff".as_slice()];
        let binary_scalars = [
            TestScalar::from(4),
            TestScalar::from(5),
            TestScalar::from(6),
        ];
        assert_eq!(
            encoded(&Column::VarBinary((&binary_values, &binary_scalars)), 3),
            encoded(&binary_values.as_slice(), 3)
        );
    }
}
