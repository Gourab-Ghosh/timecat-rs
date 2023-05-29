use std::collections::hash_map::Entry;
use EntryFlag::*;

use super::*;

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
    option_data: Option<TranspositionTableData>,
    best_move: Option<Move>,
}

// pub struct TranspositionTable {
//     table: Arc<Mutex<HashMap<u64, TranspositionTableEntry>>>,
// }

// impl TranspositionTable {
//     pub fn new() -> Self {
//         Self {
//             table: Arc::new(Mutex::new(HashMap::default())),
//         }
//     }

//     pub fn read(
//         &self,
//         key: u64,
//         depth: Depth,
//         alpha: Score,
//         beta: Score,
//     ) -> (Option<Score>, Option<Move>) {
//         let tt_entry = match self.table.lock().unwrap().get(&key) {
//             Some(entry) => *entry,
//             None => return (None, None),
//         };
//         let best_move = tt_entry.best_move;
//         if DISABLE_T_TABLE || tt_entry.option_data.is_none() {
//             return (None, best_move);
//         }
//         let data = tt_entry.option_data.unwrap();
//         if data.depth < depth {
//             return (None, best_move);
//         }
//         let score = data.score;
//         match data.flag {
//             HashExact => (Some(score), best_move),
//             HashAlpha => {
//                 if score <= alpha {
//                     (Some(score), best_move)
//                 } else {
//                     (None, best_move)
//                 }
//             }
//             HashBeta => {
//                 if score >= beta {
//                     (Some(score), best_move)
//                 } else {
//                     (None, best_move)
//                 }
//             }
//         }
//     }

//     pub fn read_best_move(&self, key: u64) -> Option<Move> {
//         self.table.lock().unwrap().get(&key)?.best_move
//     }

//     fn update_tt_entry(
//         tt_entry: &mut TranspositionTableEntry,
//         option_data: Option<TranspositionTableData>,
//         best_move: Option<Move>,
//     ) {
//         tt_entry.best_move = best_move;

//         if let Some(data) = tt_entry.option_data {
//             if let Some(curr_data) = option_data {
//                 if data.depth < curr_data.depth {
//                     tt_entry.option_data = option_data;
//                 }
//             }
//         } else {
//             tt_entry.option_data = option_data;
//         }

//         // tt_entry.option_data = option_data;
//     }

//     pub fn write(
//         &self,
//         key: u64,
//         depth: Depth,
//         ply: Ply,
//         mut score: Score,
//         flag: EntryFlag,
//         best_move: impl Into<Option<Move>>,
//     ) {
//         let best_move = best_move.into();
//         if is_checkmate(score) {
//             score += score.signum() * ply as Score;
//         }
//         let save_score = !DISABLE_T_TABLE;
//         let option_data = if save_score {
//             Some(TranspositionTableData { depth, score, flag })
//         } else {
//             None
//         };
//         let mut table_entry = self.table.lock().unwrap();
//         match table_entry.entry(key) {
//             Entry::Occupied(tt_entry) => {
//                 let tt_entry = tt_entry.into_mut();
//                 Self::update_tt_entry(tt_entry, option_data, best_move);
//             }
//             Entry::Vacant(tt_entry) => {
//                 tt_entry.insert(TranspositionTableEntry {
//                     option_data,
//                     best_move,
//                 });
//             }
//         }
//     }

//     pub fn clear(&mut self) {
//         self.table.lock().unwrap().clear();
//         // self.table = Arc::new(Mutex::new(HashMap::default()));
//     }
// }

pub struct TranspositionTable {
    table: CacheTable<TranspositionTableEntry>,
}

impl TranspositionTable {
    fn generate_new_table(memory_size: CacheTableSize) -> CacheTable<TranspositionTableEntry> {
        let (size, entry_size) =
            memory_size.to_cache_table_and_entry_size::<TranspositionTableEntry>();
        println!(
            "Transposition Table Cache size: {} MB",
            size * entry_size / 2_usize.pow(20)
        );
        CacheTable::new(size, TranspositionTableEntry::default())
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
        if DISABLE_T_TABLE || tt_entry.option_data.is_none() {
            return (None, best_move);
        }
        let data = tt_entry.option_data.unwrap();
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
        //     score += score.signum() * ply as Score;
        // }
        // let save_score = !DISABLE_T_TABLE;
        let save_score = !DISABLE_T_TABLE && score.abs() < CHECKMATE_THRESHOLD;
        let option_data = if save_score {
            let old_option_data = self.table.get(key).and_then(|entry| entry.option_data);
            if old_option_data.map(|data| data.depth).unwrap_or(-1) < depth {
                Some(TranspositionTableData { depth, score, flag })
            } else {
                old_option_data
            }
        } else {
            None
        };
        self.table.add(
            key,
            TranspositionTableEntry {
                option_data,
                best_move: best_move.into(),
            },
        );
    }

    pub fn clear(&mut self) {
        self.table = Self::generate_new_table(get_t_table_size());
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}
