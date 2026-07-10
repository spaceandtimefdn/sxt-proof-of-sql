use std::hash::{Hash, Hasher};

pub trait Scalar: Clone + Copy + Eq + Hash + Send + Sync {
    fn from_limbs(val: [u64; 4]) -> Self;
    fn to_limbs(&self) -> [u64; 4];

    fn from_str_via_hash(val: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        val.hash(&mut hasher);
        let hash = hasher.finish();
        let limbs = [(hash >> 64) as u64, (hash >> 32) as u64, hash as u64, 0];
        Self::from_limbs(limbs)
    }
}

impl Scalar for u64 {
    fn from_limbs(val: [u64; 4]) -> Self {
        val[0]
    }

    fn to_limbs(&self) -> [u64; 4] {
        [*self, 0, 0, 0]
    }
}

impl Scalar for [u64; 4] {
    fn from_limbs(val: [u64; 4]) -> Self {
        val
    }

    fn to_limbs(&self) -> [u64; 4] {
        *self
    }
}
