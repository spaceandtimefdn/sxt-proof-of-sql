/// Tests for OwnedColumn covering uncovered branches and error paths
#[cfg(test)]
mod tests {
    use crate::base::database::{ColumnType, OwnedColumn};
    use crate::base::scalar::Curve25519Scalar;

    #[test]
    fn test_owned_column_boolean_len() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::Boolean(vec![true, false, true]);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    #[test]
    fn test_owned_column_empty_is_empty() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::BigInt(vec![]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn test_owned_column_tinyint() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::TinyInt(vec![1i8, -1, 0, 127, -128]);
        assert_eq!(col.len(), 5);
        assert_eq!(col.column_type(), ColumnType::TinyInt);
    }

    #[test]
    fn test_owned_column_smallint() {
        let col: OwnedColumn<Curve25519Scalar> =
            OwnedColumn::SmallInt(vec![100i16, -100, 0]);
        assert_eq!(col.len(), 3);
        assert_eq!(col.column_type(), ColumnType::SmallInt);
    }

    #[test]
    fn test_owned_column_int() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::Int(vec![1i32, 2, 3]);
        assert_eq!(col.len(), 3);
        assert_eq!(col.column_type(), ColumnType::Int);
    }

    #[test]
    fn test_owned_column_bigint() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::BigInt(vec![1i64, 2, 3]);
        assert_eq!(col.len(), 3);
        assert_eq!(col.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_owned_column_int128() {
        let col: OwnedColumn<Curve25519Scalar> =
            OwnedColumn::Int128(vec![1i128, -1, 0]);
        assert_eq!(col.len(), 3);
        assert_eq!(col.column_type(), ColumnType::Int128);
    }

    #[test]
    fn test_owned_column_varchar() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::VarChar(vec![
            "hello".to_string(),
            "world".to_string(),
        ]);
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn test_owned_column_scalar() {
        let col: OwnedColumn<Curve25519Scalar> = OwnedColumn::Scalar(vec![
            Curve25519Scalar::from(1u64),
            Curve25519Scalar::from(2u64),
        ]);
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn test_owned_column_boolean_column_type() {
        let col: OwnedColumn<Curve25519Scalar> =
            OwnedColumn::Boolean(vec![true, false]);
        assert_eq!(col.column_type(), ColumnType::Boolean);
    }
}
