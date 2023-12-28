use super::*;

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

macro_rules! update_overwrites_and_collisions {
    ($self: ident, $e_hash: ident, $e_entry: ident, $hash: ident, $entry: ident) => {
        if $e_hash != 0 {
            if $e_hash == $hash {
                if $e_entry != $entry {
                    $self.num_overwrites.fetch_add(1, MEMORY_ORDERING);
                }
            } else {
                $self.num_collisions.fetch_add(1, MEMORY_ORDERING);
            }
        }
    };
}

pub struct CacheTable<T: Copy + Clone + PartialEq + PartialOrd> {
    table: Mutex<Box<[CacheTableEntry<T>]>>,
    size: Mutex<CacheTableSize>,
    default: T,
    mask: AtomicUsize,
    is_safe_to_do_bitwise_and: AtomicBool,
    num_overwrites: AtomicUsize,
    num_collisions: AtomicUsize,
}

impl<T: Copy + Clone + PartialEq + PartialOrd> CacheTable<T> {
    #[inline(always)]
    fn generate_table(size: CacheTableSize, default: T) -> Box<[CacheTableEntry<T>]> {
        vec![
            CacheTableEntry {
                hash: 0,
                entry: default
            };
            size.to_cache_table_size::<T>()
        ]
        .into_boxed_slice()
    }

    fn is_safe_to_do_bitwise_and(size: usize) -> bool {
        size.count_ones() == 1 && size > 1
    }

    fn get_mask(table: &[CacheTableEntry<T>]) -> usize {
        if Self::is_safe_to_do_bitwise_and(table.len()) {
            table.len() - 1
        } else {
            table.len()
        }
    }

    fn reset_mask(&self, table: &[CacheTableEntry<T>]) {
        self.mask.store(Self::get_mask(table), MEMORY_ORDERING);
        self.is_safe_to_do_bitwise_and.store(
            Self::is_safe_to_do_bitwise_and(table.len()),
            MEMORY_ORDERING,
        );
    }

    #[inline(always)]
    pub fn new(size: CacheTableSize, default: T) -> CacheTable<T> {
        let cache_table = CacheTable {
            table: Mutex::new(Self::generate_table(size, default)),
            size: Mutex::new(size),
            default,
            mask: Default::default(),
            is_safe_to_do_bitwise_and: Default::default(),
            num_overwrites: AtomicUsize::new(0),
            num_collisions: AtomicUsize::new(0),
        };
        cache_table.reset_mask(&cache_table.table.lock().unwrap());
        cache_table
    }

    #[inline(always)]
    fn get_index(&self, hash: u64) -> usize {
        if self.is_safe_to_do_bitwise_and.load(MEMORY_ORDERING) {
            hash as usize & self.mask.load(MEMORY_ORDERING)
        } else {
            hash as usize % self.mask.load(MEMORY_ORDERING)
        }
    }

    #[inline(always)]
    pub fn get(&self, hash: u64) -> Option<T> {
        let entry = unsafe {
            *self
                .table
                .lock()
                .unwrap()
                .get_unchecked(self.get_index(hash))
        };
        if entry.hash == hash {
            Some(entry.entry)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn add(&self, hash: u64, entry: T) {
        let mut table = self.table.lock().unwrap();
        let e = unsafe { table.get_unchecked_mut(self.get_index(hash)) };
        let e_hash = e.get_hash();
        let e_entry = e.get_entry();
        *e = CacheTableEntry { hash, entry };
        drop(table);
        update_overwrites_and_collisions!(self, e_hash, e_entry, hash, entry);
    }

    #[inline(always)]
    pub fn replace_if<F: Fn(T) -> bool>(&self, hash: u64, entry: T, replace: F) {
        let mut table = self.table.lock().unwrap();
        let e = unsafe { table.get_unchecked_mut(self.get_index(hash)) };
        if replace(e.entry) {
            let e_hash = e.get_hash();
            let e_entry = e.get_entry();
            *e = CacheTableEntry { hash, entry };
            drop(table);
            update_overwrites_and_collisions!(self, e_hash, e_entry, hash, entry);
        }
    }

    pub fn clear(&self) {
        self.table
            .lock()
            .unwrap()
            .iter_mut()
            .for_each(|e| e.hash = 0);
    }

    #[inline(always)]
    pub fn get_num_overwrites(&self) -> usize {
        self.num_overwrites.load(MEMORY_ORDERING)
    }

    #[inline(always)]
    pub fn get_num_collisions(&self) -> usize {
        self.num_collisions.load(MEMORY_ORDERING)
    }

    #[inline(always)]
    pub fn reset_num_overwrites(&self) {
        self.num_overwrites.store(0, MEMORY_ORDERING);
    }

    #[inline(always)]
    pub fn reset_num_collisions(&self) {
        self.num_collisions.store(0, MEMORY_ORDERING);
    }

    pub fn reset_variables(&self) {
        self.reset_num_overwrites();
        self.reset_num_collisions();
    }

    #[inline(always)]
    pub fn get_hash_full(&self) -> f64 {
        let inner_table = self.table.lock().unwrap();
        (inner_table.iter().filter(|&&e| e.hash != 0).count() as f64 / inner_table.len() as f64)
            * 100.0
    }

    pub fn len(&self) -> usize {
        self.table.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.lock().unwrap().iter().all(|&e| e.hash == 0)
    }

    pub fn get_size(&self) -> CacheTableSize {
        *self.size.lock().unwrap()
    }

    pub fn set_size(&self, size: CacheTableSize) {
        *self.size.lock().unwrap() = size;
        let mut table = self.table.lock().unwrap();
        let current_table_copy = table.clone();
        *table = Self::generate_table(size, self.default);
        self.reset_mask(&table);
        drop(table);
        self.reset_variables();
        for &CacheTableEntry { hash, entry } in current_table_copy.iter() {
            if hash != 0 {
                self.add(hash, entry);
            }
        }
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

impl TranspositionTableEntry {
    fn new(optional_data: Option<TranspositionTableData>, best_move: Option<Move>) -> Self {
        Self {
            optional_data,
            best_move,
        }
    }

    fn get_best_move(&self) -> Option<Move> {
        self.best_move
    }

    fn set_best_move(&mut self, move_: Option<Move>) {
        self.best_move = move_;
    }
}

pub struct TranspositionTable {
    table: CacheTable<TranspositionTableEntry>,
}

impl TranspositionTable {
    pub fn print_info(&self) {
        print_cache_table_info("Hash Table", self.table.len(), self.table.get_size());
    }

    fn generate_new_table(cache_table_size: CacheTableSize) -> CacheTable<TranspositionTableEntry> {
        CacheTable::new(cache_table_size, TranspositionTableEntry::default())
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
        ply: Ply,
    ) -> (Option<(Score, EntryFlag)>, Option<Move>) {
        let tt_entry = match self.table.get(key) {
            Some(entry) => entry,
            None => return (None, None),
        };
        let best_move = tt_entry.get_best_move();
        if DISABLE_T_TABLE || tt_entry.optional_data.is_none() {
            return (None, best_move);
        }
        let data = tt_entry.optional_data.unwrap();
        if data.depth < depth {
            return (None, best_move);
        }
        let mut score = data.score;
        if is_checkmate(score) {
            score -= if score.is_positive() {
                ply as Score
            } else {
                -(ply as Score)
            };
        }
        (Some((score, data.flag)), best_move)
    }

    pub fn read_best_move(&self, key: u64) -> Option<Move> {
        self.table.get(key)?.get_best_move()
    }

    pub fn write(
        &self,
        key: u64,
        depth: Depth,
        ply: Ply,
        mut score: Score,
        flag: EntryFlag,
        best_move: impl Into<Option<Move>>,
    ) {
        let save_score = !DISABLE_T_TABLE && !is_checkmate(score);
        if save_score && is_checkmate(score) {
            let mate_distance = CHECKMATE_SCORE
                .abs_diff(score.abs())
                .abs_diff(ply.try_into().unwrap()) as Score;
            let mate_score = CHECKMATE_SCORE - mate_distance;
            score = if score.is_positive() {
                mate_score
            } else {
                -mate_score
            };
        }
        let old_optional_entry = self.table.get(key);
        let optional_data = if save_score {
            let old_optional_data = old_optional_entry.and_then(|entry| entry.optional_data);
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
            TranspositionTableEntry::new(
                optional_data,
                best_move
                    .into()
                    .or(old_optional_entry.and_then(|entry| entry.get_best_move())),
            ),
        );
    }

    pub fn clear(&self) {
        self.table.clear();
    }

    pub fn clear_best_moves(&self) {
        for e in self.table.table.lock().unwrap().iter_mut() {
            e.entry.set_best_move(None);
        }
    }

    pub fn get_num_overwrites(&self) -> usize {
        self.table.get_num_overwrites()
    }

    pub fn get_num_collisions(&self) -> usize {
        self.table.get_num_collisions()
    }

    pub fn get_hash_full(&self) -> f64 {
        self.table.get_hash_full()
    }

    pub fn reset_variables(&self) {
        self.table.reset_variables();
    }

    pub fn set_size(&self, size: CacheTableSize) {
        self.table.set_size(size);
    }

    pub fn reset_size(&self) {
        self.set_size(get_t_table_size());
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}
