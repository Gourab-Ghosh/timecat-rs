// use super::*;

// #[derive(Default, Debug, Clone)]
// pub struct RepetitionTable {
//     count_map: HashMap<u64, u8>,
// }

// impl RepetitionTable {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     #[inline(always)]
//     pub fn get_repetition(&self, key: u64) -> u8 {
//         self.count_map.get(&key).copied().unwrap_or_default()
//     }

//     pub fn insert_and_get_repetition(&mut self, key: u64) -> u8 {
//         let count_entry = self.count_map.entry(key).or_insert(0);
//         *count_entry += 1;
//         *count_entry
//     }

//     pub fn remove(&mut self, key: u64) {
//         let count_entry = self.count_map.get_mut(&key).unwrap_or_else(|| {
//             panic!(
//                 "Tried to remove the key {} that doesn't exist!",
//                 key.stringify()
//             )
//         });
//         if *count_entry == 1 {
//             self.count_map.remove(&key);
//             return;
//         }
//         *count_entry -= 1;
//     }

//     #[inline(always)]
//     pub fn clear(&mut self) {
//         self.count_map.clear();
//     }
// }









use super::*;

#[derive(Debug, Clone)]
pub struct RepetitionTable {
    count_map: Box<[u8]>,
    mask: usize,
}

impl Default for RepetitionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl RepetitionTable {
    pub fn new() -> Self {
        let size = REPETITION_TABLE_SIZE << 20;
        Self {
            count_map: vec![0; size].into_boxed_slice(),
            mask: size - 1,
        }
    }

    #[inline(always)]
    const fn get_index(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }

    #[inline(always)]
    pub fn get_repetition(&self, key: u64) -> u8 {
        *get_item_unchecked!(self.count_map, self.get_index(key))
    }

    pub fn insert(&mut self, key: u64) {
        let index = self.get_index(key);
        *get_item_unchecked_mut!(self.count_map, index) += 1;
    }

    pub fn remove(&mut self, key: u64) {
        let index = self.get_index(key);
        let entry = get_item_unchecked_mut!(self.count_map, index);
        if *entry == 0 {
            panic!(
                "Tried to remove the key {} that doesn't exist!",
                key.stringify()
            )
        }
        *entry -= 1;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.count_map.iter_mut().for_each(|entry| *entry = 0);
    }
}
