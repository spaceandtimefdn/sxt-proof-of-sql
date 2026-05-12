/// Alias for the `IndexMap` with a hash builder of type `BuildHasherDefault<AHasher>`
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, core::hash::BuildHasherDefault<ahash::AHasher>>;
pub(crate) type IndexSet<T> = indexmap::IndexSet<T, core::hash::BuildHasherDefault<ahash::AHasher>>;

/// Create an [`IndexMap`][self::IndexMap] from a list of key-value pairs
#[cfg(test)]
macro_rules! indexmap {
    ($($key:expr => $value:expr,)+) => { indexmap::indexmap_with_default!{ahash::AHasher; $($key => $value),+} };
    ($($key:expr => $value:expr),*) => { indexmap::indexmap_with_default!{ahash::AHasher; $($key => $value),*} };
}

/// Create an [`IndexSet`][self::IndexSet] from a list of values
macro_rules! indexset {
    ($($value:expr,)+) => { indexmap::indexset_with_default!{ahash::AHasher; $($value),+} };
    ($($value:expr),*) => { indexmap::indexset_with_default!{ahash::AHasher; $($value),*} };
}

#[cfg(test)]
pub(crate) use indexmap;
pub(crate) use indexset;

#[cfg(test)]
mod tests {
    use super::{IndexMap, IndexSet};
    use alloc::{string::String, vec};

    #[test]
    fn indexmap_macro_preserves_insertion_order() {
        let map = indexmap! {
            "first" => 1,
            "second" => 2,
            "third" => 3,
        };

        assert_eq!(
            map.keys().copied().collect::<Vec<_>>(),
            vec!["first", "second", "third"]
        );
        assert_eq!(map.values().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn indexmap_alias_uses_expected_hasher_and_keeps_latest_duplicate_value() {
        let mut map: IndexMap<String, usize> = IndexMap::default();

        map.insert("alpha".into(), 1);
        map.insert("beta".into(), 2);
        map.insert("alpha".into(), 3);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("alpha"), Some(&3));
        assert_eq!(
            map.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
    }

    #[test]
    fn indexset_macro_preserves_insertion_order_and_removes_duplicates() {
        let set = indexset!["red", "green", "red", "blue"];

        assert_eq!(set.len(), 3);
        assert_eq!(
            set.iter().copied().collect::<Vec<_>>(),
            vec!["red", "green", "blue"]
        );
    }

    #[test]
    fn indexset_alias_uses_expected_hasher() {
        let mut set: IndexSet<String> = IndexSet::default();

        assert!(set.insert("north".into()));
        assert!(set.insert("south".into()));
        assert!(!set.insert("north".into()));

        assert_eq!(
            set.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["north", "south"]
        );
    }
}
