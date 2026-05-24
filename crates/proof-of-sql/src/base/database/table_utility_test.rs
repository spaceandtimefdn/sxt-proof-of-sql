//! Tests for table_utility module.

#[cfg(test)]
mod table_utility_tests {
    use crate::base::database::table_utility::*;
    use crate::base::scalar::test_scalar::TestScalar;
    use bumpalo::Bump;

    #[test]
    fn test_table_creation() {
        let alloc = Bump::new();
        let table = table::<TestScalar>([
            borrowed_bigint("a", [1, 2, 3], &alloc),
            borrowed_boolean("b", [true, false, true], &alloc),
        ]);
        assert_eq!(table.num_rows(), 3);
        assert_eq!(table.num_columns(), 2);
    }

    #[test]
    fn test_table_with_varchar() {
        let alloc = Bump::new();
        let table = table::<TestScalar>([
            borrowed_bigint("id", [1, 2, 3], &alloc),
            borrowed_varchar("name", ["alice", "bob", "charlie"], &alloc),
        ]);
        assert_eq!(table.num_rows(), 3);
    }

    #[test]
    fn test_borrowed_bigint_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_bigint("test", [10, 20, 30], &alloc);
        assert_eq!(name.to_string(), "test");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_varchar_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_varchar("names", ["a", "b", "c"], &alloc);
        assert_eq!(name.to_string(), "names");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_boolean_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_boolean("flag", [true, false, true], &alloc);
        assert_eq!(name.to_string(), "flag");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_scalar_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_scalar("s", [1, 2, 3], &alloc);
        assert_eq!(name.to_string(), "s");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_decimal75_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_decimal75("d", 12, 4, [100, 200, 300], &alloc);
        assert_eq!(name.to_string(), "d");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_smallint_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_smallint("s", [1, 2, 3], &alloc);
        assert_eq!(name.to_string(), "s");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_int_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_int("i", [1, 2, 3], &alloc);
        assert_eq!(name.to_string(), "i");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_borrowed_int128_macro() {
        let alloc = Bump::new();
        let (name, col) = borrowed_int128("big", [1, 2, 3], &alloc);
        assert_eq!(name.to_string(), "big");
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_table_with_options() {
        let alloc = Bump::new();
        let table = table_with_options::<TestScalar>(
            [borrowed_bigint("a", [1, 2, 3], &alloc)],
            TableOptions::new(Some(3)),
        );
        assert_eq!(table.num_rows(), 3);
    }

    #[test]
    fn test_borrowed_timestamp() {
        let alloc = Bump::new();
        let (name, col) = borrowed_timestamp("ts", [1000, 2000, 3000], &alloc);
        assert_eq!(name.to_string(), "ts");
        assert_eq!(col.len(), 3);
    }
}