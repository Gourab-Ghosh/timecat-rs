use super::*;

#[derive(Default, Debug, Clone)]
pub struct RepetitionTable {
    count_map: HashMap<u64, usize>,
}

impl RepetitionTable {
    pub fn new() -> Self {
        Self {
            count_map: HashMap::default(),
        }
    }

    #[inline(always)]
    pub fn get_repetition(&self, key: u64) -> u8 {
        *self.count_map.get(&key).unwrap_or(&0) as u8
    }

    pub fn insert_and_get_repetition(&mut self, key: u64) -> u8 {
        let count_entry = self.count_map.entry(key).or_insert(0);
        *count_entry += 1;
        *count_entry as u8
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

    #[inline(always)]
    pub fn clear(&mut self) {
        self.count_map.clear();
    }
}
