#![doc = include_str!("../README.md")]
#![allow(unused_imports)]
// #![warn(missing_docs)]
#![allow(dead_code)]

pub mod board;
pub mod chess;
pub mod constants;
pub mod error;
#[cfg(feature = "experimental")]
pub mod polyglot;
#[cfg(feature = "experimental")]
pub mod syzygy;
pub mod timer;
pub mod tt;
pub mod uci;
pub mod useful_macros;
pub mod utils;

#[cfg(feature = "engine")]
pub mod engine;

#[cfg(feature = "engine")]
pub mod parse;

#[cfg(feature = "engine")]
pub mod search;

#[cfg(feature = "engine")]
pub mod selfplay;

#[cfg(feature = "engine")]
pub mod sort;

#[cfg(feature = "engine")]
#[cfg(feature = "debug")]
pub mod tests;

#[cfg(feature = "engine")]
pub mod engine_features {
    use super::*;
    pub use constants::engine::*;
    pub use parse::*;
    pub use search::*;
    pub use sort::*;
}

#[cfg(feature = "engine")]
pub use engine_features::*;

#[cfg(feature = "engine")]
#[cfg(feature = "debug")]
pub use tests::test;

#[cfg(feature = "nnue")]
pub mod evaluate;

#[cfg(feature = "nnue")]
pub mod nnue;

#[cfg(feature = "nnue")]
pub mod nnue_rs;

#[cfg(feature = "nnue")]
pub mod nnue_features {
    use super::*;
    pub use evaluate::*;
}

#[cfg(feature = "nnue")]
pub use nnue_features::*;

#[cfg(feature = "nnue")]
lazy_static! {
    pub static ref EVALUATOR: Evaluator = Evaluator::default();
}

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
    pub use itertools::*;
    pub use paste::paste;
    #[cfg(feature = "experimental")]
    pub use polyglot::*;
    pub use utils::*;
    #[cfg(feature = "experimental")]
    pub use syzygy::*;

    // pub use std::hint;
    // pub use std::num;

    #[cfg(feature = "engine")]
    pub use engine::{Engine, GoCommand};

    #[cfg(feature = "engine")]
    pub use selfplay::self_play;
}

pub use arrayvec::ArrayVec;
pub use constants::atomic::*;
pub use constants::board::*;
pub use constants::color::*;
pub use constants::description::*;
pub use constants::io::*;
pub use constants::nnue::*;
pub use constants::print_style::*;
pub use lazy_static::lazy_static;
pub use prelude::*;
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
pub use timer::Timer;
pub use tt::*;
pub use uci::*;

lazy_static! {
    pub static ref TRANSPOSITION_TABLE: TranspositionTable = TranspositionTable::default();
    pub static ref IO_READER: IoReader = IoReader::default();
    pub static ref UCI_OPTIONS: UCIOptions = UCIOptions::default();
}

pub static UCI_STATE: EngineUCIState = EngineUCIState::new();
