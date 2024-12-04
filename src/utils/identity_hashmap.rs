use super::*;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash)]
pub struct IdentityHasher(u64);

impl std::hash::Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {}

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

pub type IdentityHashMap<K, V> = HashMap<K, V, BuildHasherDefault<IdentityHasher>>;
