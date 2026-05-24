//! Tests for owned_table_utility module.

#[cfg(test)]
mod owned_table_utility_tests {
    use crate::base::database::owned_table_utility::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_owned_table_creation() {
        let table = owned_table::<TestScalar>([
            bigint("a", [1, 2, 3]),
            boolean("b", [true, false, true]),
        ]);
        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.num_columns(), 2);
    }

    #[test]
    fn test_owned_table_with_varchar() {
        let table = owned_table::<TestScalar>([
            bigint("id", [1, 2, 3]),
            varchar("name", ["alice", "bob", "charlie"]),
        ]);
        assert_eq!(table.num_rows(), 3);
    }

    #[test]
    fn test_owned_table_with_decimal75() {
        let table = owned_table::<TestScalar>([
            bigint("price", [100, 200, 300]),
            decimal75("amount", 12, 2, [1000, 2000, 3000]),
        ]);
        assert_eq!(table.num_rows(), 3);
    }

    #[test]
    fn test_owned_table_with_scalar() {
        let table = owned_table::<TestScalar>([
            scalar("s", [1, 2, 3]),
        ]);
        assert_eq!(table.num_rows(), 3);
    }

    #[test]
    fn test_owned_table_with_int128() {
        let table = owned_table::<TestScalar>([
            int128("big", [1, 2, 3]),
        ]);
        assert_eq!(table.num_rows(), 3);
    }

    #[test]
    fn test_bigint_macro() {
        let (name, col) = bigint("test", [10, 20, 30]);
        assert_eq!(name.to_string(), "test");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_varchar_macro() {
        let (name, col) = varchar("names", ["a", "b", "c"]);
        assert_eq!(name.to_string(), "names");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_boolean_macro() {
        let (name, col) = boolean("flag", [true, false, true]);
        assert_eq!(name.to_string(), "flag");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_scalar_macro() {
        let (name, col) = scalar("s", [1, 2, 3]);
        assert_eq!(name.to_string(), "s");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_decimal75_macro() {
        let (name, col) = decimal75("d", 12, 4, [100, 200, 300]);
        assert_eq!(name.to_string(), "d");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_smallint_macro() {
        let (name, col) = smallint("s", [1, 2, 3]);
        assert_eq!(name.to_string(), "s");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_int_macro() {
        let (name, col) = int("i", [1, 2, 3]);
        assert_eq!(name.to_string(), "i");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_tinyint_macro() {
        let (name, col) = tinyint("t", [1, 2, 3]);
        assert_eq!(name.to_string(), "t");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_uint8_macro() {
        let (name, col) = uint8("u", [1, 2, 3]);
        assert_eq!(name.to_string(), "u");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_timestamp_macro() {
        let (name, col) = timestamp("ts", [1000, 2000, 3000]);
        assert_eq!(name.to_string(), "ts");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_varbinary_macro() {
        let (name, col) = varbinary("vb", [[1, 2, 3], [4, 5, 6]]);
        assert_eq!(name.to_string(), "vb");
        assert_eq!(col.len(), 2);
    }
}