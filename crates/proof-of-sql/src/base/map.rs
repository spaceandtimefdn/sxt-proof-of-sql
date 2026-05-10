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
    use alloc::vec;

    #[test]
    fn indexmap_macro_preserves_insertion_order() {
        let map = indexmap! {
            "alpha" => 1,
            "beta" => 2,
            "gamma" => 3,
        };

        assert_eq!(
            map.iter()
                .map(|(key, value)| (*key, *value))
                .collect::<alloc::vec::Vec<_>>(),
            vec![("alpha", 1), ("beta", 2), ("gamma", 3)]
        );
    }

    #[test]
    fn indexset_macro_preserves_insertion_order_and_removes_duplicates() {
        let set = indexset!["alpha", "beta", "alpha", "gamma"];

        assert_eq!(
            set.iter().copied().collect::<alloc::vec::Vec<_>>(),
            vec!["alpha", "beta", "gamma"]
        );
    }
}
