// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/dsekercioglu/blackmarlin.git

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(unused_macros)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::let_and_return)]

mod board;
mod constants;
mod engine;
mod evaluate;
mod nnue;
mod nnue_rs;
mod nnue_weights;
mod parse;
mod sort;
mod tests;
mod timer;
mod tt;
mod useful_macros;
mod utils;

use board::Board;
use cached::proc_macro::cached;
use cached::SizedCache;
use chess::Color::*;
use chess::Piece::*;
use chess::{
    get_adjacent_files, get_bishop_moves, get_file as get_file_bb, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_rank as get_rank_bb, get_rook_moves, BitBoard,
    BoardStatus, ChessMove as Move, Color, File, Piece, Rank, Square,
};
use constants::bitboard::*;
use constants::board_representation::*;
use constants::engine_constants::*;
use constants::fen::*;
use constants::print_style::*;
use constants::square::*;
use constants::types::*;
use engine::{Engine, GoCommand};
use evaluate::*;
use failure::Fail;
use fxhash::FxHashMap as HashMap;
use itertools::Itertools;
use parse::*;
use sort::*;
use std::cmp::{self, Ordering};
use std::convert::From;
use std::env;
use std::fmt::{self, Display};
use std::io::{self, Write};
use std::mem;
use std::num::ParseIntError;
use std::str::FromStr;
use std::str::ParseBoolError;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tests::test;
use timer::Timer;
use tt::*;
use useful_macros::*;
use utils::bitboard_utils::*;
use utils::classes::*;
use utils::common_utils::*;
use utils::square_utils::*;
use utils::string_utils::*;
use utils::unsafe_utils::*;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    if ["windows"].contains(&env::consts::OS) {
        set_colored_output(false);
    }
    let clock = Instant::now();
    if env::args().contains(&String::from("--test")) {
        test();
    } else {
        Parser::main_loop();
    }
    let elapsed_time = clock.elapsed().as_secs_f32();
    let precision = 3;
    println_info("\nRun Time", format!("{:.1$} s", elapsed_time, precision));
}
