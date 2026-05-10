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
    use crate::base::{database::Column, scalar::test_scalar::TestScalar};
    use alloc::vec;

    #[test]
    fn arrays_delegate_result_column_serialization_to_slices() {
        let values = [7_i16, -11_i16];
        let slice = &values[..];
        assert_eq!(values.num_bytes(2), slice.num_bytes(2));

        let mut array_output = vec![0; values.num_bytes(2)];
        let mut slice_output = vec![0; slice.num_bytes(2)];
        assert_eq!(values.write(&mut array_output, 2), array_output.len());
        assert_eq!(slice.write(&mut slice_output, 2), slice_output.len());
        assert_eq!(array_output, slice_output);
    }

    #[test]
    fn columns_delegate_result_column_serialization_to_backing_values() {
        let values = [7_i64, -11_i64];
        let column = Column::<TestScalar>::BigInt(&values);
        let slice = &values[..];
        assert_eq!(column.num_bytes(2), slice.num_bytes(2));

        let mut column_output = vec![0; column.num_bytes(2)];
        let mut slice_output = vec![0; slice.num_bytes(2)];
        assert_eq!(column.write(&mut column_output, 2), column_output.len());
        assert_eq!(slice.write(&mut slice_output, 2), slice_output.len());
        assert_eq!(column_output, slice_output);
    }
}
