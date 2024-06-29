use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash)]
struct IdentityHasher(u64);

impl std::hash::Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _: &[u8]) {}

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Debug, Clone)]
pub struct RepetitionTable {
    count_map: std::collections::HashMap<u64, u8, std::hash::BuildHasherDefault<IdentityHasher>>,
}

impl RepetitionTable {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get_repetition(&self, key: u64) -> u8 {
        self.count_map.get(&key).copied().unwrap_or_default()
    }

    #[inline]
    pub fn insert(&mut self, key: u64) {
        *self.count_map.entry(key).or_insert(0) += 1;
    }

    pub fn insert_and_get_repetition(&mut self, key: u64) -> u8 {
        let count_entry = self.count_map.entry(key).or_insert(0);
        *count_entry += 1;
        *count_entry
    }

    pub fn remove(&mut self, key: u64) {
        let count_entry = self.count_map.get_mut(&key).unwrap_or_else(|| {
            panic!(
                "Tried to remove the key {} that doesn't exist!",
                key.stringify()
            )
        });
        if *count_entry == 1 {
            self.count_map.remove(&key);
            return;
        }
        *count_entry -= 1;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.count_map.clear();
    }
}

// const REPETITION_TABLE_SIZE: usize = 1;

// #[cfg(feature = "speed")]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[derive(Debug, Clone)]
// pub struct RepetitionTable {
//     count_map: Box<[u8]>,
//     mask: usize,
// }

// #[cfg(feature = "speed")]
// impl Default for RepetitionTable {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// #[cfg(feature = "speed")]
// impl RepetitionTable {
//     pub fn new() -> Self {
//         let size = REPETITION_TABLE_SIZE << 20;
//         #[cfg(test)]
//         debug_assert!(size.is_power_of_two());
//         Self {
//             count_map: vec![0; size].into_boxed_slice(),
//             mask: size - 1,
//         }
//     }

//     #[inline]
//     const fn get_index(&self, key: u64) -> usize {
//         (key as usize) & self.mask
//     }

//     #[inline]
//     pub fn get_repetition(&self, key: u64) -> u8 {
//         *get_item_unchecked!(self.count_map, self.get_index(key))
//     }

//     #[inline]
//     pub fn insert(&mut self, key: u64) {
//         *get_item_unchecked_mut!(self.count_map, self.get_index(key)) += 1;
//     }

//     pub fn insert_and_get_repetition(&mut self, key: u64) -> u8 {
//         let count_entry = get_item_unchecked_mut!(self.count_map, self.get_index(key));
//         *count_entry += 1;
//         *count_entry
//     }

//     pub fn remove(&mut self, key: u64) {
//         let index = self.get_index(key);
//         let entry = get_item_unchecked_mut!(self.count_map, index);
//         if *entry == 0 {
//             panic!(
//                 "Tried to remove the key {} that doesn't exist!",
//                 key.stringify()
//             )
//         }
//         *entry -= 1;
//     }

//     #[inline]
//     pub fn clear(&mut self) {
//         self.count_map.iter_mut().for_each(|entry| *entry = 0);
//     }
// }
