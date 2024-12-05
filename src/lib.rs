#![doc = include_str!("../README.md")]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![allow(clippy::result_large_err)]
#![expect(clippy::needless_doctest_main)]
#![expect(clippy::too_many_arguments)]
// #![deny(missing_debug_implementations)]
// #![warn(missing_docs)]

pub mod board;
pub mod chess;
pub mod constants;
pub mod custom_engine;
pub mod error;
pub mod evaluate;
#[cfg(feature = "nnue_reader")]
pub mod nnue;
pub mod parse;
#[cfg(feature = "experimental")]
pub mod polyglot;
pub mod runner;
pub mod search;
pub mod search_controller;
pub mod selfplay;
pub mod sort;
#[cfg(feature = "experimental")]
pub mod syzygy;
#[cfg(feature = "debug")]
pub mod tests;
pub mod tt;
pub mod uci;
pub mod useful_macros;
pub mod utils;

pub mod prelude {
    use super::*;
    pub use super::{
        get_bishop_moves, get_castle_moves, get_pv_as_san, get_pv_as_uci, get_pv_string,
        get_queen_moves, get_rook_moves, is_checkmate, self_play, simplify_fen, Bishop, BitBoard,
        Black, BlackBishop, BlackKing, BlackKnight, BlackPawn, BlackQueen, BlackRook, Board,
        BoardPosition, BoardPositionBuilder, BoardStatus, CacheTable, CacheTableSize,
        CastleMoveType, CastleRights, Color, Depth, Engine, Evaluator, File, GameResult, GoCommand,
        IoReader, King, Knight, Move, MoveWeight, Pawn, Piece, PieceType, Ply, Queen, Rank,
        RepetitionTable, Rook, Score, SearchConfig, SearchInfo, SearchInfoBuilder, Square,
        TimecatError, TranspositionTable, ValidOrNullMove, WeightedMove, White, WhiteBishop,
        WhiteKing, WhiteKnight, WhitePawn, WhiteQueen, WhiteRook, Zobrist, A1, A2, A3, A4, A5, A6,
        A7, A8, ALL_FILES, ALL_PIECES, ALL_PIECE_TYPES, ALL_RANKS, ALL_SQUARES, B1, B2, B3, B4, B5,
        B6, B7, B8, BB_A1, BB_A2, BB_A3, BB_A4, BB_A5, BB_A6, BB_A7, BB_A8, BB_ALL, BB_B1, BB_B2,
        BB_B3, BB_B4, BB_B5, BB_B6, BB_B7, BB_B8, BB_BACKRANKS, BB_C1, BB_C2, BB_C3, BB_C4, BB_C5,
        BB_C6, BB_C7, BB_C8, BB_CENTER, BB_CORNERS, BB_D1, BB_D2, BB_D3, BB_D4, BB_D5, BB_D6,
        BB_D7, BB_D8, BB_DARK_SQUARES, BB_E1, BB_E2, BB_E3, BB_E4, BB_E5, BB_E6, BB_E7, BB_E8,
        BB_EDGES, BB_F1, BB_F2, BB_F3, BB_F4, BB_F5, BB_F6, BB_F7, BB_F8, BB_FILE_A, BB_FILE_B,
        BB_FILE_C, BB_FILE_D, BB_FILE_E, BB_FILE_F, BB_FILE_G, BB_FILE_H, BB_G1, BB_G2, BB_G3,
        BB_G4, BB_G5, BB_G6, BB_G7, BB_G8, BB_H1, BB_H2, BB_H3, BB_H4, BB_H5, BB_H6, BB_H7, BB_H8,
        BB_LEFT_HALF_BOARD, BB_LIGHT_SQUARES, BB_LOWER_HALF_BOARD, BB_RANK_1, BB_RANK_2, BB_RANK_3,
        BB_RANK_4, BB_RANK_5, BB_RANK_6, BB_RANK_7, BB_RANK_8, BB_RIGHT_HALF_BOARD, BB_SQUARES,
        BB_UPPER_HALF_BOARD, C1, C2, C3, C4, C5, C6, C7, C8, CHECKMATE_SCORE, CHECKMATE_THRESHOLD,
        D1, D2, D3, D4, D5, D6, D7, D8, E1, E2, E3, E4, E5, E6, E7, E8, EMPTY_FEN, ENGINE_AUTHOR,
        ENGINE_NAME, ENGINE_VERSION, F1, F2, F3, F4, F5, F6, F7, F8, G1, G2, G3, G4, G5, G6, G7,
        G8, H1, H2, H3, H4, H5, H6, H7, H8, INFINITY, PAWN_VALUE, PROMOTION_PIECES,
        STARTING_POSITION_FEN,
    };

    pub use utils::extension_traits::*;
}

pub use arrayvec::ArrayVec;
#[cfg(feature = "nnue_reader")]
pub use binread::{BinRead, BinResult};
pub use board::*;
pub use chess::*;
pub use constants::atomic::*;
pub use constants::binary::*;
pub use constants::bitboard_and_square::*;
pub use constants::board::*;
pub use constants::cache_table::*;
pub use constants::color::*;
pub use constants::default_parameters::*;
pub use constants::description::*;
pub use constants::engine::*;
pub use constants::evaluate::*;
pub use constants::fen::*;
pub use constants::files::*;
pub use constants::io::*;
pub use constants::piece::*;
pub use constants::ranks::*;
pub use constants::strings::*;
pub use constants::types::*;
pub use custom_engine::*;
pub use error::*;
pub use evaluate::*;
pub use itertools::*;
#[cfg(feature = "nnue_reader")]
pub use nnue::*;
pub use parse::*;
pub use paste::paste;
#[cfg(feature = "experimental")]
pub use polyglot::*;
#[cfg(feature = "pyo3")]
pub use pyo3::prelude::*;
pub use runner::*;
pub use search::*;
pub use search_controller::SearchController;
pub use selfplay::self_play;
#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};
pub use sort::*;
pub use std::cmp::{Ordering, Reverse};
pub use std::collections::HashSet;
pub use std::convert::From;
pub use std::env;
pub use std::error::Error;
pub use std::fmt::{self, Debug};
pub use std::fs;
pub use std::hash::{Hash, Hasher};
pub use std::io::{BufReader, Read, Seek};
pub use std::iter::Sum;
pub use std::num::{NonZeroU64, NonZeroUsize, ParseIntError, Wrapping};
pub use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref,
    DerefMut, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Not, Range, Rem, RemAssign,
    Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
pub use std::str::{FromStr, ParseBoolError};
pub use std::sync::atomic::{AtomicBool, AtomicUsize};
pub use std::sync::{Arc, LazyLock, RwLock};
pub use std::thread;
#[cfg(not(feature = "wasm"))]
pub use std::time::{Duration, Instant};
#[cfg(feature = "debug")]
pub use tests::test;
pub use tt::*;
pub use uci::*;
pub use utils::*;
#[cfg(feature = "wasm")]
pub use web_time::{Duration, Instant};

// pub use std::hint;
// pub use std::num;

pub static GLOBAL_TIMECAT_STATE: GlobalTimecatState = GlobalTimecatState::new();
