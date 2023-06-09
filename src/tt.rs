use super::*;
use EntryFlag::*;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct CacheTableEntry<T: Copy + Clone + PartialEq + PartialOrd> {
    hash: u64,
    entry: T,
}

impl<T: Copy + Clone + PartialEq + PartialOrd> CacheTableEntry<T> {
    #[inline(always)]
    pub fn new(hash: u64, entry: T) -> CacheTableEntry<T> {
        CacheTableEntry { hash, entry }
    }

    #[inline(always)]
    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    pub fn get_entry(&self) -> T {
        self.entry
    }
}

pub struct CacheTable<T: Copy + Clone + PartialEq + PartialOrd> {
    table: Box<[CacheTableEntry<T>]>,
    mask: usize,
    num_collisions: usize,
}

impl<T: Copy + Clone + PartialEq + PartialOrd> CacheTable<T> {
    #[inline(always)]
    pub fn new(size: usize, default: T) -> CacheTable<T> {
        if size.count_ones() != 1 {
            panic!("You cannot create a CacheTable with a non-binary number.");
        }
        let values = vec![
            CacheTableEntry {
                hash: 0,
                entry: default
            };
            size
        ];
        CacheTable {
            table: values.into_boxed_slice(),
            mask: size - 1,
            num_collisions: 0,
        }
    }

    #[inline(always)]
    fn get_index(&self, hash: u64) -> usize {
        // (hash ^ hash.rotate_left(32)) as usize & self.mask
        hash as usize & self.mask
    }

    #[inline(always)]
    pub fn get(&self, hash: u64) -> Option<T> {
        let entry = unsafe { *self.table.get_unchecked(self.get_index(hash)) };
        if entry.hash == hash {
            Some(entry.entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn add(&mut self, hash: u64, entry: T) {
        let e = unsafe { self.table.get_unchecked_mut(self.get_index(hash)) };
        if e.hash != 0 && e.hash != hash {
            self.num_collisions += 1;
        }
        *e = CacheTableEntry { hash, entry };
    }

    #[inline(always)]
    pub fn replace_if<F: Fn(T) -> bool>(&mut self, hash: u64, entry: T, replace: F) {
        let e = unsafe { self.table.get_unchecked_mut(self.get_index(hash)) };
        if replace(e.entry) {
            if e.hash != 0 && e.hash != hash {
                self.num_collisions += 1;
            }
            *e = CacheTableEntry { hash, entry };
        }
    }

    pub fn clear(&mut self) {
        for e in self.table.iter_mut() {
            e.hash = 0;
        }
        self.num_collisions = 0;
    }

    #[inline(always)]
    pub fn get_num_collisions(&self) -> usize {
        self.num_collisions
    }

    #[inline(always)]
    pub fn reset_num_collisions(&mut self) {
        self.num_collisions = 0;
    }

    pub fn reset_variables(&mut self) {
        self.reset_num_collisions();
        self.clear()
    }

    #[inline(always)]
    pub fn get_hash_full(&self) -> f64 {
        (self.table.iter().filter(|&&e| e.hash != 0).count() as f64 / self.table.len() as f64)
            * 100.0
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Default)]
pub enum EntryFlag {
    #[default]
    HashExact,
    HashAlpha,
    HashBeta,
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
struct TranspositionTableData {
    depth: Depth,
    score: Score,
    flag: EntryFlag,
}

impl TranspositionTableData {
    pub fn depth(&self) -> Depth {
        self.depth
    }

    pub fn score(&self) -> Score {
        self.score
    }

    pub fn flag(&self) -> EntryFlag {
        self.flag
    }
}

impl Default for TranspositionTableData {
    fn default() -> Self {
        Self {
            depth: -1,
            score: Default::default(),
            flag: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Default)]
pub struct TranspositionTableEntry {
    optional_data: Option<TranspositionTableData>,
    best_move: Option<Move>,
}

pub struct TranspositionTable {
    table: CacheTable<TranspositionTableEntry>,
}

impl TranspositionTable {
    fn generate_new_table(cache_table_size: CacheTableSize) -> CacheTable<TranspositionTableEntry> {
        println_info(
            "Transposition Table Cache size",
            format!(
                "{} MB",
                cache_table_size.to_cache_table_memory_size::<TranspositionTableEntry>()
            ),
        );
        println_info(
            "Transposition Table Cells Count",
            format!(
                "{} MB",
                cache_table_size.to_cache_table_size::<TranspositionTableEntry>()
            ),
        );
        CacheTable::new(
            cache_table_size.to_cache_table_size::<TranspositionTableEntry>(),
            TranspositionTableEntry::default(),
        )
    }

    pub fn new() -> Self {
        Self {
            table: Self::generate_new_table(get_t_table_size()),
        }
    }

    pub fn read(
        &self,
        key: u64,
        depth: Depth,
        alpha: Score,
        beta: Score,
    ) -> (Option<Score>, Option<Move>) {
        let tt_entry = match self.table.get(key) {
            Some(entry) => entry,
            None => return (None, None),
        };
        let best_move = tt_entry.best_move;
        if DISABLE_T_TABLE || tt_entry.optional_data.is_none() {
            return (None, best_move);
        }
        let data = tt_entry.optional_data.unwrap();
        if data.depth < depth {
            return (None, best_move);
        }
        let score = data.score;
        match data.flag {
            HashExact => (Some(score), best_move),
            HashAlpha => {
                if score <= alpha {
                    (Some(score), best_move)
                } else {
                    (None, best_move)
                }
            }
            HashBeta => {
                if score >= beta {
                    (Some(score), best_move)
                } else {
                    (None, best_move)
                }
            }
        }
    }

    pub fn read_best_move(&self, key: u64) -> Option<Move> {
        self.table.get(key)?.best_move
    }

    pub fn write(
        &mut self,
        key: u64,
        depth: Depth,
        _ply: Ply,
        score: Score,
        flag: EntryFlag,
        best_move: impl Into<Option<Move>>,
    ) {
        // if is_checkmate(score) {
        //     let mate_ply = CHECKMATE_SCORE.abs_diff(score.abs()).abs_diff(ply.try_into().unwrap()) as Score;
        //     let mate_score = CHECKMATE_SCORE - mate_ply;
        //     score = if score.is_positive() { mate_score } else { -mate_score };
        // }
        // let save_score = !DISABLE_T_TABLE;
        let save_score = !DISABLE_T_TABLE || !is_checkmate(score);
        let optional_data = if save_score {
            let old_optional_data = self.table.get(key).and_then(|entry| entry.optional_data);
            if old_optional_data.map(|data| data.depth).unwrap_or(-1) < depth {
                Some(TranspositionTableData { depth, score, flag })
            } else {
                old_optional_data
            }
        } else {
            None
        };
        self.table.add(
            key,
            TranspositionTableEntry {
                optional_data,
                best_move: best_move.into(),
            },
        );
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn clear_best_moves(&mut self) {
        for e in self.table.table.iter_mut() {
            e.entry.best_move = None;
        }
    }

    pub fn get_num_collisions(&self) -> usize {
        self.table.get_num_collisions()
    }

    pub fn get_hash_full(&self) -> f64 {
        self.table.get_hash_full()
    }

    pub fn reset_variables(&mut self) {
        self.table.reset_variables();
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}
