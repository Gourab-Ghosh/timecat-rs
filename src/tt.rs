use super::*;

trait ToUnsigned {
    type Unsigned;
    fn to_unsigned(self) -> Self::Unsigned;
}

macro_rules! to_unsigned {
    ($from:ty, $to:ty) => {
        impl ToUnsigned for $from {
            type Unsigned = $to;

            fn to_unsigned(self) -> Self::Unsigned {
                self as Self::Unsigned
            }
        }
    };
}

to_unsigned!(i8, u8);
to_unsigned!(i16, u16);
to_unsigned!(i32, u32);
to_unsigned!(i64, u64);
to_unsigned!(i128, u128);
to_unsigned!(isize, usize);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Default)]
pub enum EntryFlag {
    #[default]
    HashExact,
    HashAlpha,
    HashBeta,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
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

    pub fn new(cache_table_size: CacheTableSize) -> Self {
        Self {
            table: Self::generate_new_table(cache_table_size),
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
        if tt_entry.optional_data.is_none() {
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
        // TODO: Logic Wrong Here
        let save_score = !is_checkmate(score);
        if save_score && is_checkmate(score) {
            let mate_distance = CHECKMATE_SCORE
                .abs_diff(score.abs())
                .abs_diff(ply as <Score as ToUnsigned>::Unsigned)
                as Score;
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
        for e in self.table.get_table().write().unwrap().iter_mut() {
            e.get_entry_mut().set_best_move(None);
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
        self.set_size(GLOBAL_UCI_STATE.get_t_table_size());
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(GLOBAL_UCI_STATE.get_t_table_size())
    }
}
