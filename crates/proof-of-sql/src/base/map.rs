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

    #[test]
    fn indexmap_macro_preserves_insert_order() {
        let map: IndexMap<&str, i32> = indexmap! {
            "first" => 1,
            "second" => 2,
        };

        assert_eq!(map.keys().copied().collect::<Vec<_>>(), ["first", "second"]);
        assert_eq!(map.get("first"), Some(&1));
        assert_eq!(map.get("second"), Some(&2));
    }

    #[test]
    fn indexset_macro_preserves_insert_order() {
        let set: IndexSet<&str> = indexset!["alpha", "beta", "gamma"];

        assert_eq!(
            set.iter().copied().collect::<Vec<_>>(),
            ["alpha", "beta", "gamma"]
        );
        assert!(set.contains("beta"));
    }
}
