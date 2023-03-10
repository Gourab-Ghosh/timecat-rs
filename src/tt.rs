use std::collections::hash_map::Entry;
use EntryFlag::*;

use super::*;

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum EntryFlag {
    HashExact,
    HashAlpha,
    HashBeta,
}

impl Default for EntryFlag {
    fn default() -> Self {
        Self::HashExact
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Default)]
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

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Default)]
struct TranspositionTableEntry {
    option_data: Option<TranspositionTableData>,
    best_move: Option<Move>,
}

// pub struct TranspositionTable {
//     table: CacheTable<TranspositionTableEntry>,
// }

// impl TranspositionTable {
//     fn generate_cache_key(key: u64) -> u64 {
//         let mut cache_key = key;
//         cache_key ^= cache_key >> 32;
//         cache_key ^= cache_key >> 16;
//         cache_key ^= cache_key >> 8;
//         cache_key ^= cache_key >> 4;
//         cache_key ^= cache_key >> 2;
//         cache_key ^= cache_key >> 1;
//         cache_key
//     }

//     pub fn new() -> Self {
//         let size = mem::size_of::<TranspositionTableEntry>();
//         let dynamic_size = 2 << (18 + (T_TABLE_SIZE as f64 / size as f64).log(2.0).round() as i32);
//         println!("size: {}, dynamic_size: {}", size, dynamic_size);
//         Self {
//             table: CacheTable::new(dynamic_size, Default::default()),
//         }
//     }

//     // pub fn read(
//     //     &self,
//     //     key: u64,
//     //     depth: Depth,
//     //     alpha: Score,
//     //     beta: Score,
//     // ) -> Option<(Option<Score>, Option<Move>)> {
//     //     if DISABLE_T_TABLE {
//     //         return None;
//     //     }
//     //     match self.table.get(key) {
//     //         Some(tt_entry) => {
//     //             let best_move = tt_entry.best_move;
//     //             if let Some(data) = tt_entry.option_data {
//     //                 if data.depth >= depth {
//     //                     match data.flag {
//     //                         HashExact => Some((Some(data.score), best_move)),
//     //                         HashAlpha => {
//     //                             if data.score <= alpha {
//     //                                 Some((Some(data.score), best_move))
//     //                             } else {
//     //                                 Some((None, best_move))
//     //                             }
//     //                         }
//     //                         HashBeta => {
//     //                             if data.score >= beta {
//     //                                 Some((Some(data.score), best_move))
//     //                             } else {
//     //                                 Some((None, best_move))
//     //                             }
//     //                         }
//     //                     }
//     //                 } else {
//     //                     Some((None, best_move))
//     //                 }
//     //             } else {
//     //                 Some((None, best_move))
//     //             }
//     //         }
//     //         None => None,
//     //     }
//     // }

//     pub fn read(
//         &self,
//         key: u64,
//         depth: Depth,
//         alpha: Score,
//         beta: Score,
//     ) -> Option<(Option<Score>, Option<Move>)> {
//         if DISABLE_T_TABLE {
//             return None;
//         }
//         let hash = Self::generate_cache_key(key);
//         match self.table.get(hash) {
//             Some(tt_entry) => {
//                 let best_move = tt_entry.best_move;
//                 if let Some(data) = tt_entry.option_data {
//                     if data.depth >= depth {
//                         return match data.flag {
//                             HashExact => Some((Some(data.score), best_move)),
//                             HashAlpha => {
//                                 if data.score <= alpha {
//                                     Some((Some(data.score), best_move))
//                                 } else {
//                                     Some((None, best_move))
//                                 }
//                             }
//                             HashBeta => {
//                                 if data.score >= beta {
//                                     Some((Some(data.score), best_move))
//                                 } else {
//                                     Some((None, best_move))
//                                 }
//                             }
//                         };
//                     }
//                 }
//                 Some((None, best_move))
//             }
//             None => None,
//         }
//     }

//     #[inline(always)]
//     pub fn read_best_move(&self, key: u64) -> Option<Move> {
//         self.table.get(key).map(|d| d.best_move).flatten()
//     }

//     pub fn write(
//         &mut self,
//         key: u64,
//         depth: Depth,
//         score: Score,
//         flag: EntryFlag,
//         best_move: Option<Move>,
//         write_tt: bool,
//     ) {
//         let hash = Self::generate_cache_key(key);
//         let save_score = !DISABLE_T_TABLE && write_tt && self.table.get(hash).map(|e| e.option_data.map(|d| d.depth)).flatten().unwrap_or(0) <= depth;
//         let option_data = if save_score {
//             Some(TranspositionTableData { depth, score, flag })
//         } else {
//             None
//         };
//         self.table.add(hash, TranspositionTableEntry { option_data, best_move });
//     }
// }

pub struct TranspositionTable {
    table: Arc<Mutex<HashMap<u64, TranspositionTableEntry>>>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(HashMap::default())),
        }
    }

    pub fn read(
        &self,
        key: u64,
        depth: Depth,
        alpha: Score,
        beta: Score,
    ) -> Option<(Option<Score>, Option<Move>)> {
        if DISABLE_T_TABLE {
            return None;
        }
        match self.table.lock().unwrap().get(&key) {
            Some(tt_entry) => {
                let best_move = tt_entry.best_move;
                if let Some(data) = tt_entry.option_data {
                    if data.depth >= depth {
                        return match data.flag {
                            HashExact => Some((Some(data.score), best_move)),
                            HashAlpha => {
                                if data.score <= alpha {
                                    Some((Some(data.score), best_move))
                                } else {
                                    Some((None, best_move))
                                }
                            }
                            HashBeta => {
                                if data.score >= beta {
                                    Some((Some(data.score), best_move))
                                } else {
                                    Some((None, best_move))
                                }
                            }
                        };
                    }
                }
                Some((None, best_move))
            }
            None => None,
        }
    }

    pub fn read_best_move(&self, key: u64) -> Option<Move> {
        match self.table.lock().unwrap().get(&key) {
            Some(tt_entry) => tt_entry.best_move,
            None => None,
        }
    }

    fn update_tt_entry(
        tt_entry: &mut TranspositionTableEntry,
        option_data: Option<TranspositionTableData>,
        best_move: Option<Move>,
    ) {
        tt_entry.best_move = best_move;

        if let Some(data) = tt_entry.option_data {
            if let Some(curr_data) = option_data {
                if data.depth < curr_data.depth {
                    tt_entry.option_data = option_data;
                }
            }
        } else {
            tt_entry.option_data = option_data;
        }

        // tt_entry.option_data = option_data;
    }

    pub fn write(
        &self,
        key: u64,
        depth: Depth,
        score: Score,
        flag: EntryFlag,
        best_move: Option<Move>,
        write_tt: bool,
    ) {
        let save_score = write_tt && !DISABLE_T_TABLE;
        // let save_score = !is_checkmate(score) && !DISABLE_T_TABLE;
        let option_data = if save_score {
            Some(TranspositionTableData { depth, score, flag })
        } else {
            None
        };
        let mut table_entry = self.table.lock().unwrap();
        match table_entry.entry(key) {
            Entry::Occupied(tt_entry) => {
                let tt_entry = tt_entry.into_mut();
                Self::update_tt_entry(tt_entry, option_data, best_move);
            }
            Entry::Vacant(tt_entry) => {
                tt_entry.insert(TranspositionTableEntry {
                    option_data,
                    best_move,
                });
            }
        }
    }

    pub fn clear(&mut self) {
        self.table.lock().unwrap().clear();
        // self.table = Arc::new(Mutex::new(HashMap::default()));
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}