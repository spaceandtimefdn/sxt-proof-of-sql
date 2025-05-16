use super::ByteDistribution;
use crate::base::{bit::bit_mask_utils::make_bit_mask, scalar::Scalar};
use alloc::vec::Vec;
use bnum::types::U256;
use bumpalo::Bump;
use core::ops::Shr;

/// Calculates the byte matrix for a column of data
/// The `Vec<&'a [u8]>` values are the bytes of the columns that vary.
/// The `ByteDistribution` indicate which columns are constant and what those constant values are.
#[expect(clippy::missing_panics_doc)]
#[cfg_attr(not(test), expect(dead_code))]
pub fn compute_varying_byte_matrix<'a, S: Scalar>(
    column_data: &[S],
    alloc: &'a Bump,
) -> (Vec<&'a [u8]>, ByteDistribution) {
    let dist = ByteDistribution::new::<S, S>(column_data);
    let byte_matrix = dist
        .varying_byte_indices()
        .map(|start_index| {
            alloc.alloc_slice_fill_iter(column_data.iter().map(|row| {
                let bit_mask = make_bit_mask(*row);
                // This will not panic because & 255 guarantees the value is at most 255
                (bit_mask.shr(start_index) & U256::from(255u8))
                    .try_into()
                    .unwrap()
            })) as &[_]
        })
        .collect();
    (byte_matrix, dist)
}

#[cfg(test)]
mod tests {
    use crate::base::{
        byte::{byte_matrix_utils::compute_varying_byte_matrix, ByteDistribution},
        scalar::{test_scalar::TestScalar, Scalar},
    };
    use bumpalo::Bump;

    #[test]
    fn we_can_compute_varying_byte_matrix_for_small_scalars() {
        let alloc = Bump::new();
        let scalars: Vec<TestScalar> = [1, 2, 3, 255, 256, 257]
            .iter()
            .map(TestScalar::from)
            .collect();
        let expected_byte_distribution = ByteDistribution::new::<TestScalar, TestScalar>(&scalars);
        let (varying_columns, byte_distribution) =
            compute_varying_byte_matrix::<TestScalar>(&scalars, &alloc);
        assert_eq!(byte_distribution, expected_byte_distribution);
        let expected_word_columns = vec![vec![1, 2, 3, 255, 0, 1], vec![0, 0, 0, 0, 1, 1]];
        assert_eq!(varying_columns, expected_word_columns);
    }

    #[test]
    fn we_can_compute_varying_byte_matrix_for_large_scalars() {
        let alloc = Bump::new();
        let scalars = vec![
            TestScalar::MAX_SIGNED,
            TestScalar::from(u64::MAX),
            -TestScalar::ONE,
        ];
        let expected_byte_distribution = ByteDistribution::new::<TestScalar, TestScalar>(&scalars);
        let (varying_columns, byte_distribution) =
            compute_varying_byte_matrix::<TestScalar>(&scalars, &alloc);
        assert_eq!(byte_distribution, expected_byte_distribution);

        let expected_word_columns = vec![
            [246, 255, 255],
            [233, 255, 255],
            [122, 255, 255],
            [46, 255, 255],
            [141, 255, 255],
            [49, 255, 255],
            [9, 255, 255],
            [44, 255, 255],
            [107, 0, 255],
            [206, 0, 255],
            [123, 0, 255],
            [81, 0, 255],
            [239, 0, 255],
            [124, 0, 255],
            [111, 0, 255],
            [10, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [0, 0, 255],
            [136, 128, 127],
        ];

        assert_eq!(varying_columns, expected_word_columns);
    }
}
