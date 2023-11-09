#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

mod board;
mod constants;
mod engine;
mod evaluate;
mod nnue;
mod nnue_rs;
mod parse;
mod polyglot;
mod search;
mod selfplay;
mod sort;
mod syzygy;
mod tests;
mod timer;
mod tt;
mod uci;
mod useful_macros;
mod utils;

pub use board::*;
pub use chess::Color::*;
pub use chess::Piece::*;
pub use chess::{
    get_adjacent_files, get_bishop_moves, get_file as get_file_bb, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_rank as get_rank_bb, get_rook_moves, BitBoard,
    BoardStatus, ChessMove as Move, Color, File, MoveGen, Piece, Rank, Square, ALL_COLORS,
    ALL_FILES, ALL_PIECES, ALL_RANKS, ALL_SQUARES,
};
pub use constants::atomic::*;
pub use constants::bitboard::*;
pub use constants::board_representation::*;
pub use constants::description::*;
pub use constants::engine_constants::*;
pub use constants::fen::*;
pub use constants::print_style::*;
pub use constants::square::*;
pub use constants::types::*;
pub use engine::{Engine, GoCommand};
pub use evaluate::*;
pub use failure::Fail;
pub use fxhash::FxHashMap as HashMap;
pub use itertools::Itertools;
use lazy_static::lazy_static;
pub use parse::*;
pub use polyglot::*;
pub use search::*;
pub use selfplay::self_play;
pub use sort::*;
pub use std::cmp::{self, Ordering};
pub use std::convert::From;
pub use std::env;
pub use std::fmt::{self, Display};
pub use std::fs;
pub use std::mem;
pub use std::num::ParseIntError;
pub use std::ops::{Add, AddAssign};
pub use std::str::{FromStr, ParseBoolError};
pub use std::sync::atomic::{AtomicBool, AtomicUsize};
pub use std::sync::{Arc, Mutex};
pub use std::thread;
pub use std::time::{Duration, Instant};
pub use syzygy::*;
use tests::test;
pub use timer::Timer;
pub use tt::*;
pub use uci::*;
pub use useful_macros::*;
pub use utils::bitboard_utils::*;
pub use utils::cache_table_utils::*;
pub use utils::classes::*;
pub use utils::engine_error::*;
pub use utils::engine_utils::*;
pub use utils::global_utils::*;
pub use utils::hash_utils::*;
pub use utils::info_utils::*;
pub use utils::io_utils::*;
pub use utils::move_utils::*;
pub use utils::piece_utils::*;
pub use utils::pv_utils::*;
pub use utils::square_utils::*;
pub use utils::string_utils::*;
pub use utils::time_utils::*;

// pub use std::hint;
// pub use std::num;

lazy_static! {
    pub static ref TRANSPOSITION_TABLE: TranspositionTable = TranspositionTable::default();
    pub static ref EVALUATOR: Evaluator = Evaluator::default();
    pub static ref IO_READER: IoReader = IoReader::default();
    pub static ref UCI_OPTIONS: UCIOptionsVec = UCIOptionsVec::default();
}
