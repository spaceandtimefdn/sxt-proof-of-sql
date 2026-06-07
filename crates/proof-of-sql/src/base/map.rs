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
    use super::{indexmap, indexset, IndexMap, IndexSet};
    use alloc::{vec, vec::Vec};

    #[test]
    fn indexmap_macro_preserves_insertion_order_and_overwrites_duplicates() {
        let map = indexmap! {
            "first" => 1,
            "second" => 2,
            "first" => 3,
        };

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("first"), Some(&3));
        assert_eq!(
            map.keys().copied().collect::<Vec<_>>(),
            vec!["first", "second"]
        );
    }

    #[test]
    fn indexset_macro_preserves_insertion_order_and_deduplicates_values() {
        let set = indexset!["alpha", "beta", "alpha"];

        assert_eq!(set.len(), 2);
        assert_eq!(
            set.iter().copied().collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
    }

    #[test]
    fn map_aliases_use_the_same_default_hasher_as_the_macros() {
        let map: IndexMap<&str, u8> = indexmap! {"one" => 1};
        let set: IndexSet<&str> = indexset!["one"];

        assert_eq!(map.get("one"), Some(&1));
        assert!(set.contains("one"));
    }
}
