//! Tests for base map module.

#[cfg(test)]
mod map_test {
    use crate::base::IndexMap;

    #[test]
    fn test_index_map_type_exists() {
        let _: Option<IndexMap<i32, i32>> = None;
    }

    #[test]
    fn test_index_map_creation() {
        use crate::base::indexmap;
        let map = indexmap! { 1 => 10, 2 => 20 };
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1), Some(&10));
    }
}
