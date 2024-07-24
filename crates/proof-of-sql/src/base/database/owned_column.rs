/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
use super::ColumnType;
use crate::base::{
    math::{
        decimal::Precision,
        permutation::{Permutation, PermutationError},
    },
    scalar::Scalar,
};
use core::cmp::Ordering;
use proof_of_sql_parser::{
    intermediate_ast::OrderByDirection,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
};

#[derive(Debug, PartialEq, Clone, Eq)]
#[non_exhaustive]
/// Supported types for OwnedColumn
pub enum OwnedColumn<S: Scalar> {
    /// Boolean columns
    Boolean(Vec<bool>),
    /// i16 columns
    SmallInt(Vec<i16>),
    /// i32 columns
    Int(Vec<i32>),
    /// i64 columns
    BigInt(Vec<i64>),
    /// String columns
    VarChar(Vec<String>),
    /// i128 columns
    Int128(Vec<i128>),
    /// Decimal columns
    Decimal75(Precision, i8, Vec<S>),
    /// Scalar columns
    Scalar(Vec<S>),
    /// Timestamp columns
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, Vec<i64>),
}

impl<S: Scalar> OwnedColumn<S> {
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            OwnedColumn::Boolean(col) => col.len(),
            OwnedColumn::SmallInt(col) => col.len(),
            OwnedColumn::Int(col) => col.len(),
            OwnedColumn::BigInt(col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
            OwnedColumn::Decimal75(_, _, col) => col.len(),
            OwnedColumn::Scalar(col) => col.len(),
            OwnedColumn::TimestampTZ(_, _, col) => col.len(),
        }
    }

    /// Returns the column with its entries permutated
    pub fn try_permute(&self, permutation: &Permutation) -> Result<Self, PermutationError> {
        Ok(match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(permutation.try_apply(col)?),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(permutation.try_apply(col)?),
            OwnedColumn::Int(col) => OwnedColumn::Int(permutation.try_apply(col)?),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(permutation.try_apply(col)?),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(permutation.try_apply(col)?),
            OwnedColumn::Int128(col) => OwnedColumn::Int128(permutation.try_apply(col)?),
            OwnedColumn::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, permutation.try_apply(col)?)
            }
            OwnedColumn::Scalar(col) => OwnedColumn::Scalar(permutation.try_apply(col)?),
            OwnedColumn::TimestampTZ(tu, tz, col) => {
                OwnedColumn::TimestampTZ(*tu, *tz, permutation.try_apply(col)?)
            }
        })
    }

    /// Returns the sliced column.
    pub fn slice(&self, start: usize, end: usize) -> Self {
        match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(col[start..end].to_vec()),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(col[start..end].to_vec()),
            OwnedColumn::Int(col) => OwnedColumn::Int(col[start..end].to_vec()),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(col[start..end].to_vec()),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(col[start..end].to_vec()),
            OwnedColumn::Int128(col) => OwnedColumn::Int128(col[start..end].to_vec()),
            OwnedColumn::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, col[start..end].to_vec())
            }
            OwnedColumn::Scalar(col) => OwnedColumn::Scalar(col[start..end].to_vec()),
            OwnedColumn::TimestampTZ(tu, tz, col) => {
                OwnedColumn::TimestampTZ(*tu, *tz, col[start..end].to_vec())
            }
        }
    }

    /// Returns true if the column is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedColumn::Boolean(col) => col.is_empty(),
            OwnedColumn::SmallInt(col) => col.is_empty(),
            OwnedColumn::Int(col) => col.is_empty(),
            OwnedColumn::BigInt(col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::Int128(col) => col.is_empty(),
            OwnedColumn::Scalar(col) => col.is_empty(),
            OwnedColumn::Decimal75(_, _, col) => col.is_empty(),
            OwnedColumn::TimestampTZ(_, _, col) => col.is_empty(),
        }
    }
    /// Returns the type of the column.
    pub fn column_type(&self) -> ColumnType {
        match self {
            OwnedColumn::Boolean(_) => ColumnType::Boolean,
            OwnedColumn::SmallInt(_) => ColumnType::SmallInt,
            OwnedColumn::Int(_) => ColumnType::Int,
            OwnedColumn::BigInt(_) => ColumnType::BigInt,
            OwnedColumn::VarChar(_) => ColumnType::VarChar,
            OwnedColumn::Int128(_) => ColumnType::Int128,
            OwnedColumn::Scalar(_) => ColumnType::Scalar,
            OwnedColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
            OwnedColumn::TimestampTZ(tu, tz, _) => ColumnType::TimestampTZ(*tu, *tz),
        }
    }

    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i16], panicking if it is not.
    pub fn i16_iter(&self) -> impl Iterator<Item = &i16> {
        match self {
            OwnedColumn::SmallInt(col) => col.iter(),
            _ => panic!("Expected SmallInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i32], panicking if it is not.
    pub fn i32_iter(&self) -> impl Iterator<Item = &i32> {
        match self {
            OwnedColumn::Int(col) => col.iter(),
            _ => panic!("Expected Int column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i64], panicking if it is not.
    pub fn i64_iter(&self) -> impl Iterator<Item = &i64> {
        match self {
            OwnedColumn::BigInt(col) => col.iter(),
            OwnedColumn::TimestampTZ(_, _, col) => col.iter(),
            _ => panic!("Expected TimestampTZ or BigInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i128], panicking if it is not.
    pub fn i128_iter(&self) -> impl Iterator<Item = &i128> {
        match self {
            OwnedColumn::Int128(col) => col.iter(),
            _ => panic!("Expected Int128 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [bool], panicking if it is not.
    pub fn bool_iter(&self) -> impl Iterator<Item = &bool> {
        match self {
            OwnedColumn::Boolean(col) => col.iter(),
            _ => panic!("Expected Boolean column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is a [Scalar], panicking if it is not.
    pub fn scalar_iter(&self) -> impl Iterator<Item = &S> {
        match self {
            OwnedColumn::Scalar(col) => col.iter(),
            OwnedColumn::Decimal75(_, _, col) => col.iter(),
            _ => panic!("Expected Scalar or Decimal75 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [String], panicking if it is not.
    pub fn string_iter(&self) -> impl Iterator<Item = &String> {
        match self {
            OwnedColumn::VarChar(col) => col.iter(),
            _ => panic!("Expected VarChar column"),
        }
    }
}

/// Compares the tuples (order_by_pairs[0][i], order_by_pairs[1][i], ...) and
/// (order_by_pairs[0][j], order_by_pairs[1][j], ...) in lexicographic order.
/// Note that direction flips the ordering.
pub(crate) fn compare_indexes_by_owned_columns_with_direction<S: Scalar>(
    order_by_pairs: &[(OwnedColumn<S>, OrderByDirection)],
    i: usize,
    j: usize,
) -> Ordering {
    order_by_pairs
        .iter()
        .map(|(col, direction)| {
            let ordering = match col {
                OwnedColumn::Boolean(col) => col[i].cmp(&col[j]),
                OwnedColumn::SmallInt(col) => col[i].cmp(&col[j]),
                OwnedColumn::Int(col) => col[i].cmp(&col[j]),
                OwnedColumn::BigInt(col) => col[i].cmp(&col[j]),
                OwnedColumn::Int128(col) => col[i].cmp(&col[j]),
                OwnedColumn::Decimal75(_, _, col) => col[i].cmp(&col[j]),
                OwnedColumn::Scalar(col) => col[i].cmp(&col[j]),
                OwnedColumn::VarChar(col) => col[i].cmp(&col[j]),
                OwnedColumn::TimestampTZ(_, _, col) => col[i].cmp(&col[j]),
            };
            match direction {
                OrderByDirection::Asc => ordering,
                OrderByDirection::Desc => ordering.reverse(),
            }
        })
        .find(|&ord| ord != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::Curve25519Scalar};
    use proof_of_sql_parser::intermediate_ast::OrderByDirection;

    #[test]
    fn we_can_slice_a_column() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        assert_eq!(col.slice(1, 4), OwnedColumn::Int128(vec![2, 3, 4]));
    }

    #[test]
    fn we_can_permute_a_column() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        let permutation = Permutation::try_new(vec![1, 3, 4, 0, 2]).unwrap();
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Int128(vec![2, 4, 5, 1, 3])
        );
    }

    #[test]
    fn we_can_compare_columns() {
        let col1: OwnedColumn<Curve25519Scalar> = OwnedColumn::SmallInt(vec![1, 1, 2, 1, 1]);
        let col2: OwnedColumn<Curve25519Scalar> = OwnedColumn::VarChar(
            ["b", "b", "a", "b", "a"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        let col3: OwnedColumn<Curve25519Scalar> = OwnedColumn::Decimal75(
            Precision::new(70).unwrap(),
            20,
            [1, 2, 2, 1, 2]
                .iter()
                .map(|&i| Curve25519Scalar::from(i))
                .collect(),
        );
        let order_by_pairs = vec![
            (col1, OrderByDirection::Asc),
            (col2, OrderByDirection::Desc),
            (col3, OrderByDirection::Asc),
        ];
        // Equal on col1 and col2, less on col3
        assert_eq!(
            compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 1),
            Ordering::Less
        );
        // Less on col1
        assert_eq!(
            compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 2),
            Ordering::Less
        );
        // Equal on all 3 columns
        assert_eq!(
            compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 0, 3),
            Ordering::Equal
        );
        // Equal on col1, greater on col2 reversed
        assert_eq!(
            compare_indexes_by_owned_columns_with_direction(&order_by_pairs, 1, 4),
            Ordering::Less
        )
    }
}
