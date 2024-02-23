#![allow(dead_code)]

mod board;
mod chess;
mod constants;
mod engine;
mod error;
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

pub mod prelude {
    use super::*;
    pub use board::*;
    pub use chess::*;
    pub use constants::bitboard_and_square::*;
    pub use constants::fen::*;
    pub use constants::files::*;
    pub use constants::piece::*;
    pub use constants::ranks::*;
    pub use constants::types::*;
    pub use error::*;
    pub use itertools::Itertools;
    pub use parse::*;
    pub use paste::paste;
    pub use polyglot::*;
    pub use selfplay::self_play;
    pub use utils::*;
    // pub use syzygy::*;
    // pub use std::hint;
    // pub use std::num;
}

pub use arrayvec::ArrayVec;
pub use constants::atomic::*;
pub use constants::board_representation::*;
pub use constants::color::*;
pub use constants::description::*;
pub use constants::engine_constants::*;
pub use constants::print_style::*;
pub use engine::{Engine, GoCommand};
pub use evaluate::*;
pub use failure::Fail;
pub use lazy_static::lazy_static;
pub use prelude::*;
pub use search::*;
pub use sort::*;
pub use std::cmp::Ordering;
pub use std::convert::From;
pub use std::env;
pub use std::error::Error;
pub use std::fmt;
pub use std::fs;
pub use std::mem::{self, transmute};
pub use std::num::ParseIntError;
pub use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index,
    IndexMut, Mul, MulAssign, Not, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
pub use std::str::{FromStr, ParseBoolError};
pub use std::sync::atomic::{AtomicBool, AtomicUsize};
pub use std::sync::{Arc, Mutex};
pub use std::thread;
pub use std::time::{Duration, Instant};
pub use tests::test;
pub use timer::Timer;
pub use tt::*;
pub use uci::*;

lazy_static! {
    pub static ref TRANSPOSITION_TABLE: TranspositionTable = TranspositionTable::default();
    pub static ref EVALUATOR: Evaluator = Evaluator::default();
    pub static ref IO_READER: IoReader = IoReader::default();
    pub static ref UCI_OPTIONS: UCIOptions = UCIOptions::default();
}

pub static UCI_STATE: EngineUCIState = EngineUCIState::new();
