use super::*;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash)]
pub struct IdentityHasher(u64);

impl std::hash::Hasher for IdentityHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // TODO: Maybe make the logic better?
        for &byte in bytes {
            self.0 = self.0 << 3 ^ byte as u64;
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 ^= i;
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_i128(&mut self, i: i128) {
        self.0 ^= i as u64;
    }

    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.0 ^= i as u64;
    }
}

pub type IdentityHashMap<K, V> = HashMap<K, V, BuildHasherDefault<IdentityHasher>>;
