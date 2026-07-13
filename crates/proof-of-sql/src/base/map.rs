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
    use super::IndexSet;

    #[test]
    fn indexset_macro_preserves_insertion_order() {
        let values = super::indexset![3_u8, 1_u8, 2_u8];
        let collected: Vec<_> = values.into_iter().collect();
        assert_eq!(collected, vec![3, 1, 2]);
    }

    #[test]
    fn indexset_macro_deduplicates_values() {
        let values = super::indexset!["a", "b", "a"];
        assert_eq!(values.len(), 2);
        assert!(values.contains("a"));
        assert!(values.contains("b"));
    }

    #[test]
    fn indexset_alias_uses_default_hasher_type() {
        let values: IndexSet<u32> = super::indexset![10_u32, 20_u32];
        assert_eq!(values.get_index(0), Some(&10));
        assert_eq!(values.get_index(1), Some(&20));
    }
}
