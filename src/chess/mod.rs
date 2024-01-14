#![allow(unused_imports)]
#![allow(dead_code)]

mod bitboard;
mod board;
mod board_builder;
mod castle;
mod color;
mod constants;
mod error;
mod files;
mod moves;
mod piece;
mod ranks;
mod square;
mod zobrist;
mod magic;

pub use itertools::*;
pub use std::fmt;
pub use std::mem::transmute;
pub use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index, IndexMut, Mul,
    MulAssign, Not,
};
pub use std::str::FromStr;
pub use Square::*;
pub use bitboard::*;
pub use board::*;
pub use board_builder::*;
pub use castle::*;
pub use color::*;
pub use constants::*;
pub use error::*;
pub use files::*;
pub use moves::*;
pub use piece::*;
pub use ranks::*;
pub use square::*;
pub use zobrist::*;
pub use magic::*;
