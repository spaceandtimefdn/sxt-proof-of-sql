//! This module gives some utility functions for determining the position of data within the dynamic dory matrix
//!
//! In general, the data is filled in such a way that the new data is always in the last row, and the row size
//! (and consequently, the matrix size) is strictly increasing.
//! Aside from the first 3 rows, the pattern is to have 3\*2^n rows of length 4\*2^n.
//! In particular this means that a 2^n by 2^n matrix contains exactly 2^(2\*n-1) data points when n>0.
//!
//! This structure allows for a multilinear point evaluation of the associated MLE to be represented as a
//! relatively simple matrix product.
//!
//! Concretely, if the data being committed to is 0, 1, 2, 3, etc., the matrix is filled as shown below.
//!
//! ```text
//!   0
//!   _,   1
//!   2,   3
//!   4,   5,   6,   7
//!   8,   9,  10,  11
//!  12,  13,  14,  15
//!  16,  17,  18,  19,  20,  21,  22,  23
//!  24,  25,  26,  27,  28,  29,  30,  31
//!  32,  33,  34,  35,  36,  37,  38,  39
//!  40,  41,  42,  43,  44,  45,  46,  47
//!  48,  49,  50,  51,  52,  53,  54,  55
//!  56,  57,  58,  59,  60,  61,  62,  63
//!  64,  65,  66,  67,  68,  69,  70,  71,  72,  73,  74,  75,  76,  77,  78,  79
//!  80,  81,  82,  83,  84,  85,  86,  87,  88,  89,  90,  91,  92,  93,  94,  95
//!  96,  97,  98,  99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111
//! 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127
//! ...
//! ```

/// Sets the data to indicate if no index exists.
const fn no_index() -> usize {
    usize::MAX
}

/// Returns if the index exists.
pub(crate) const fn index_exists(index: usize) -> bool {
    index != no_index()
}

/// Returns the full width of a row in the matrix.
pub(crate) const fn full_width_of_row(row: usize) -> usize {
    ((2 * row + 4) / 3).next_power_of_two()
}

/// Returns the index that belongs in the first column in a particular row.
///
/// Note: when row = 1, this correctly returns 0, even though no data belongs there.
#[cfg(test)]
pub(crate) const fn row_start_index(row: usize) -> usize {
    let width_of_row = full_width_of_row(row);
    width_of_row * (row - width_of_row / 2)
}

/// Returns the (row, column) in the matrix where the data with the given index belongs.
pub(crate) const fn row_and_column_from_index(index: usize) -> (usize, usize) {
    let width_of_row = 1 << (((2 * index + 1).ilog2() + 1) / 2);
    let row = index / width_of_row + width_of_row / 2;
    let column = index % width_of_row;
    (row, column)
}

/// Returns the index of data where the (row, column) belongs.
pub(crate) const fn index_from_row_and_column(row: usize, column: usize) -> usize {
    let width_of_row = full_width_of_row(row);

    // Return no_index if the (row, column) is not in dynamic Dory structure.
    if column >= width_of_row || (row == 1 && column == 0) {
        return no_index();
    }

    width_of_row * (row - width_of_row / 2) + column
}

/// Returns a matrix size that can hold the given number of data points being committed with respect to an offset.
pub(crate) const fn matrix_size(data_len: usize, offset: usize) -> (usize, usize) {
    let (last_row, _) = row_and_column_from_index(offset + data_len - 1);
    let width_of_last_row = full_width_of_row(last_row);
    (width_of_last_row, last_row + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Creates a 2^nu by 2^nu matrix that is partially filled with the indexes that belong in each position.
    fn create_position_matrix(nu: usize) -> Vec<Vec<Option<usize>>> {
        let max_index = 1 << (2 * nu - 1);
        let mut matrix = vec![];
        for i in 0..max_index {
            let (row, column) = row_and_column_from_index(i);
            if matrix.len() <= row {
                matrix.resize_with(row + 1, Vec::new);
            }
            if matrix[row].len() <= column {
                matrix[row].resize(column + 1, None);
            }
            matrix[row][column] = Some(i);
        }
        matrix
    }
    #[test]
    fn we_can_compute_positions_from_indexes() {
        assert_eq!(
            create_position_matrix(2),
            vec![
                vec![Some(0)],                            // length 1
                vec![None, Some(1)],                      // length "1"
                vec![Some(2), Some(3)],                   // length 2
                vec![Some(4), Some(5), Some(6), Some(7)], // length 4
            ],
        );
        assert_eq!(
            create_position_matrix(4),
            vec![
                vec![Some(0)],                                // length 1
                vec![None, Some(1)],                          // length "1"
                vec![Some(2), Some(3)],                       // length 2
                vec![Some(4), Some(5), Some(6), Some(7)],     // length 4
                vec![Some(8), Some(9), Some(10), Some(11)],   // length 4
                vec![Some(12), Some(13), Some(14), Some(15)], // length 4
                (16..=23).map(Some).collect(),                // length 8
                (24..=31).map(Some).collect(),                // length 8
                (32..=39).map(Some).collect(),                // length 8
                (40..=47).map(Some).collect(),                // length 8
                (48..=55).map(Some).collect(),                // length 8
                (56..=63).map(Some).collect(),                // length 8
                (64..=79).map(Some).collect(),                // length 16
                (80..=95).map(Some).collect(),                // length 16
                (96..=111).map(Some).collect(),               // length 16
                (112..=127).map(Some).collect()               // length 16
            ],
        );
    }
    #[test]
    fn we_can_fill_matrix_with_no_collisions_and_correct_size_and_row_starts() {
        for nu in 1..10 {
            let num_vars = nu * 2 - 1;
            let matrix = create_position_matrix(nu);
            // Every position should be unique.
            assert_eq!(
                matrix.iter().flatten().filter(|&x| x.is_some()).count(),
                1 << num_vars
            );
            // The matrix should have 2^nu rows.
            assert_eq!(matrix.len(), 1 << nu);
            // The matrix should have 2^nu columns.
            assert_eq!(matrix.iter().map(Vec::len).max().unwrap(), 1 << nu);
            for (index, row) in matrix.iter().enumerate() {
                // The start of each row should match with `row_start_index`.
                assert_eq!(
                    row_start_index(index),
                    match index {
                        1 => 0, // Row 1 starts at 0, even though nothing is put in the 0th position. This is because the 0th index is at position (0, 0)
                        _ => row[0]
                            .expect("Every row except 1 should have a value in the 0th position."),
                    }
                )
            }
        }
    }
    #[test]
    fn we_can_correctly_identify_index_existence() {
        assert_eq!(index_exists(0), true);
        assert_eq!(index_exists(no_index()), false);
        assert_eq!(index_exists(no_index() - 1), true);
    }
    #[test]
    fn we_can_find_the_full_width_of_row() {
        let width_of_row = full_width_of_row(0);
        assert_eq!(width_of_row, 1);

        let width_of_row = full_width_of_row(1);
        assert_eq!(width_of_row, 2);

        let width_of_row = full_width_of_row(4);
        assert_eq!(width_of_row, 4);

        let width_of_row = full_width_of_row(7);
        assert_eq!(width_of_row, 8);

        let width_of_row = full_width_of_row(11);
        assert_eq!(width_of_row, 8);

        let width_of_row = full_width_of_row(12);
        assert_eq!(width_of_row, 16);

        let width_of_row = full_width_of_row(13);
        assert_eq!(width_of_row, 16);
    }
    #[test]
    fn we_can_produce_the_correct_matrix_size() {
        let (width, height) = matrix_size(2, 0);
        assert_eq!(width, 2);
        assert_eq!(height, 2);

        let (width, height) = matrix_size(3, 0);
        assert_eq!(width, 2);
        assert_eq!(height, 3);

        let (width, height) = matrix_size(4, 0);
        assert_eq!(width, 2);
        assert_eq!(height, 3);

        let (width, height) = matrix_size(16, 0);
        assert_eq!(width, 4);
        assert_eq!(height, 6);

        let (width, height) = matrix_size(17, 0);
        assert_eq!(width, 8);
        assert_eq!(height, 7);

        let (width, height) = matrix_size(64, 0);
        assert_eq!(width, 8);
        assert_eq!(height, 12);

        let (width, height) = matrix_size(71, 0);
        assert_eq!(width, 16);
        assert_eq!(height, 13);
    }
    #[test]
    fn we_can_produce_the_correct_matrix_size_with_offset() {
        let (width, height) = matrix_size(2, 1);
        assert_eq!(width, 2);
        assert_eq!(height, 3);

        let (width, height) = matrix_size(16, 1);
        assert_eq!(width, 8);
        assert_eq!(height, 7);

        let (width, height) = matrix_size(64, 1);
        assert_eq!(width, 16);
        assert_eq!(height, 13);

        let (width, height) = matrix_size(4, 16);
        assert_eq!(width, 8);
        assert_eq!(height, 7);
    }
    #[test]
    fn we_can_find_index_from_row_and_column() {
        for i in 0..1 << 10 {
            let (row, column) = row_and_column_from_index(i);
            let index = index_from_row_and_column(row, column);
            assert_eq!(index, i);
        }
    }
    #[test]
    fn we_can_find_index_from_row_and_column_not_in_dynamic_dory_structure() {
        let index = index_from_row_and_column(0, 1);
        assert_eq!(index, no_index());

        let index = index_from_row_and_column(1, 0);
        assert_eq!(index, no_index());

        let index = index_from_row_and_column(1, 2);
        assert_eq!(index, no_index());

        let index = index_from_row_and_column(3, 4);
        assert_eq!(index, no_index());

        let index = index_from_row_and_column(6, 8);
        assert_eq!(index, no_index());

        let index = index_from_row_and_column(12, 16);
        assert_eq!(index, no_index());
    }
}
