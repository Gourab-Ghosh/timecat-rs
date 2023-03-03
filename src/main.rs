// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue
// https://github.com/dsekercioglu/blackmarlin.git

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(unused_macros)]
// #![allow(private_in_public)]
// #[allow(improper_ctypes)]

mod board;
mod constants;
mod engine;
mod evaluate;
mod nnue;
mod nnue_rs;
mod nnue_weights;
mod parse;
mod useful_macros;
mod utils;

use board::Board;
use chess::Color::*;
use chess::Piece::*;
use chess::{
    get_adjacent_files, get_bishop_moves, get_file as get_file_bb, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_rank as get_rank_bb, get_rook_moves, BitBoard,
    BoardStatus, ChessMove as Move, Color, Error as ChessError, File, Piece, Rank, Square,
};
use constants::bitboard::*;
use constants::engine_constants::*;
use constants::fen::*;
use constants::piece::*;
use constants::print_style::*;
use constants::square::*;
use constants::types::*;
use engine::Engine;
use evaluate::*;
use fxhash::FxHashMap as HashMap;
use parse::*;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use useful_macros::*;
use utils::classes::*;
use utils::command_utils::*;
use utils::square_utils::*;
use utils::string_utils::*;
use utils::unsafe_utils::*;

fn self_play(depth: u8, print: bool) {
    let mut engine = Engine::default();
    println!("\n{}\n", engine.board);
    while !engine.board.is_game_over() {
        let (best_move, score) = engine.go(depth, print);
        let best_move_san = engine.board.san(best_move);
        let pv = engine.get_pv_string();
        engine.push(best_move);
        println!("\n{}\n", engine.board);
        println_info("Best Move", best_move_san);
        println_info("Score", score_to_string(score));
        println_info("Num Nodes Searched", engine.get_num_nodes_searched());
        println_info("PV Line", pv);
    }
    println!("Game PGN:\n\n{}", engine.board.get_pgn());
}

fn _main() {
    // self_play(10, false);
    Parser::parse_command(&mut Engine::default(), "go depth 10");
    // Parser::parse_command(&mut Engine::default(), "go perft 7");

    // let mut engine = Engine::default();
    // engine.board.set_fen("8/7P/2p5/p1n2k2/6R1/p4P2/7K/2r5 b - - 0 53");
    // Parser::parse_command(&mut engine, "go depth 20");

    // let mut board = Board::new();
    // println!("\n{board}");
    // for san in ["e4", "Nf6", "Be2", "Nxe4"] {
    //     let move_ = board.parse_san(san).unwrap();
    //     let move_str = board.san(move_);
    //     println!("\nPushing move {move_str}");
    //     board.push(move_);
    //     println!("\n{board}");
    // }
}

fn main() {
    let clock = Instant::now();
    // Parser::main_loop();
    _main();
    let elapsed_time = clock.elapsed().as_secs_f32();
    let precision = 3;
    println_info("\nRun Time", format!("{:.1$} s", elapsed_time, precision));
}
