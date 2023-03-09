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
    BoardStatus, CacheTable, ChessMove as Move, Color, File, Piece, Rank, Square,
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
use std::cmp::Ordering;
use std::env;
use std::fmt::{self, Display};
use std::mem;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use useful_macros::*;
use utils::classes::*;
use utils::common_utils::*;
use utils::square_utils::*;
use utils::string_utils::*;
use utils::unsafe_utils::*;

fn self_play(engine: &mut Engine, depth: Depth, print: bool) {
    println!("\n{}\n", engine.board);
    while !engine.board.is_game_over() {
        let clock = Instant::now();
        let (best_move, score) = engine.go(depth, print);
        let time_elapsed = clock.elapsed();
        let best_move_san = engine.board.san(best_move);
        let pv = engine.get_pv_string();
        engine.push(best_move);
        let nps =
            (engine.get_num_nodes_searched() as u128 * 10u128.pow(9)) / time_elapsed.as_nanos();
        println!("\n{}\n", engine.board);
        println_info("Best Move", best_move_san);
        println_info("Score", score_to_string(score));
        println_info("Num Nodes Searched", engine.get_num_nodes_searched());
        println_info("PV Line", pv);
        println_info("Time Taken", format!("{:.3} s", time_elapsed.as_secs_f32()));
        println_info("Nodes per second", format!("{} nodes/s", nps));
    }
    println!("Game PGN:\n\n{}", engine.board.get_pgn());
}

pub fn parse_command(engine: &mut Engine, raw_input: &str) {
    match Parser::parse_command(engine, raw_input) {
        Ok(_) => (),
        Err(err) => panic!("{}", err.generate_error(Some(raw_input))),
    }
}

fn _main() {
    let could_have_played_better_move = [ // ig
        "5rk1/6pp/p1p5/1p1pqn2/1P6/2NP3P/2PQ1PP1/R5K1 w - - 0 26",
    ];

    let time_consuming_fens = [
        "r2qrbk1/2p2ppp/b1p2n2/p2p4/4PB2/P1NB4/1PP2PPP/R2QR1K1 w - - 3 13",
        "2qr2k1/2p2pp1/2p4p/p3b3/8/P6P/1PPBQPP1/4R1K1 w - - 9 23",
        "r2qkb1r/p4pp1/2p4p/8/2n3n1/2NP4/PP2NPP1/R1BQK2R b KQkq - 1 14",
        "r2qk2r/p4pp1/2p4p/2b5/2n3n1/2NP4/PP2NPP1/R1BQK2R w KQkq - 2 15",
        "8/7R/8/8/8/8/2k3K1/8 w - - 4 3",
        "r3r3/3q1pk1/2pn2pp/pp1pR3/3P1P2/P6P/1P2QPP1/3NR1K1 b - - 10 33",
    ];

    let mut engine = Engine::default();
    // engine.board.set_fen("8/8/8/1R5K/3k4/8/8/5rq1 b - - 1 96");
    // engine.board.push_sans("e4 e6 d4 d5"); // caro cann defense
    // engine.board.push_sans("d4 d5 c4"); // queens gambit
    // engine.board.push_sans("d4 d5 c4 dxc4"); // queens gambit accepted
    // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5"); // fried liver attack
    // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5"); // traxer counter attack
    // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5 Nxf7"); // traxer counter attack with Nxf7
    // engine.board.set_fen("8/6k1/3r4/7p/7P/4R1P1/5P1K/8 w - - 3 59"); // endgame improvement 1
    // engine.board.set_fen("8/7R/8/8/8/7K/k7/8 w - - 0 1"); // endgame improvement 2
    self_play(&mut engine, 12, false);

    // parse_command(&mut Engine::default(), "go depth 14");
    // parse_command(&mut Engine::default(), "go perft 7");

    // let mut engine = Engine::default();
    // // engine.board.set_fen("6k1/5p2/6p1/1K6/8/8/3r4/7q b - - 1 88"); // test if engine can find mate in 3
    // // engine.board.set_fen("7R/r7/3K4/8/5k2/8/8/8 b - - 80 111"); // test t_table -> nodes initially: 3203606
    // // engine.board.set_fen("8/8/K5k1/2q5/8/1Q6/8/8 b - - 20 105"); // gives incomplete pv line
    // engine.board.set_fen("r7/2p4p/r1p2pk1/P2p4/R5P1/PK6/2P2P2/3R4 b - - 3 31");
    // // engine.board.push("Rb8+ Rb4 Rba8 Rb3 Rb8+ Rb2 Rba8 Rb1 Rb8+ Rb2 Rba8 Rb3 Rb8+ Rb4 Rba8 Rb5 Rb8+ Rb6 Rba8 Rb7 Rb8+ Rb8 Rba8 Rb7 Rb8+ Rb6 Rba8 Rb5 Rb8+ Rb4 Rba8 Rb3 Rb8+ Rb2 Rba8 Rb1 Rb8+ Rb2 Rba8 Rb3 Rb8+ Rb4 Rba8 Rb5 Rb8+ Rb6 Rba8 Rb7 Rb8+ Rb8 Rba8 Rb7 Rb8+ Rb6 Rba8 Rb5 Rb8+ Rb4 Rba8 Rb3 Rb8+ Rb2 Rba8 Rb1 Rb8+ Rb2 Rba8 Rb3 Rb8+ Rb4 Rba8 Rb5 Rb8+ Rb6 Rba8 Rb7 Rb8+ Rb8 Rba8 Rb7 Rb8+ Rb6 Rba8 Rb5 Rb8+ Rb4 Rba8 Rb3 Rb8+ Rb2 Rba8 Rb1 Rb8+ Rb2 Rba8 Rb3 Rb8+ Rb4 Rba8 Rb5 Rb8+ Rb6 Rba8 Rb7 Rb8+ Rb8 Rba8 Rb7 Rb8+ Rb6 Rba8 Rb5 Rb8+ Rb4 Rba8 Rb3 Rb8+ Rb2 Rba8 Rb1 Rb8+ Rb2 Rba8 Rb3 Rb8+ Rb4 Rba8 Rb5 Rb8+ Rb6 Rba8 Rb7 Rb8+ Rb8 Rba8 Rb7 Rb8+ Rb6 Rba8 Rb5 Rb8+ Rb4 Rba8 Rb3 Rb8+ Rb2 Rba8 Rb1 Rb8+ Rb2 Rba8 Rb3 Rb8+ Rb4 Rba8 Rb5 Rb8");
    // engine.board.push_sans("Rb8+ Rb4 Rba8 Ra4 Rb8+ Rb4 Rba8");
    // parse_command(&mut engine, "go depth 12");
    // engine.board.push_san("Ra4");
    // println!("{}", engine.board.is_game_over());

    // let mut engine = Engine::default();
    // engine.board.set_fen("rnbqkbnr/pP4pp/8/2pppp2/8/8/1PPPPPPP/RNBQKBNR w KQkq - 0 5");
    // for _move in engine.board.generate_legal_moves() {
    //     println!("{}", engine.board.san(_move));
    // }

    // let mut board = Board::new();
    // println!("\n{board}");
    // for san in ["e4", "Nf6", "Be2", "Nxe4"] {
    //     let _move = board.parse_san(san).unwrap();
    //     let move_str = board.san(_move);
    //     println!("\nPushing move {move_str}");
    //     board.push(_move);
    //     println!("\n{board}");
    // }

    // let mut engine = Engine::default();
    // engine.board.set_fen("rnbqkbnr/ppp1p1pp/3p4/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    // let _move = engine.board.parse_uci("e5f6").unwrap();
    // println!("{}", engine.board.is_en_passant(_move));
    // println!("{}", engine.board.san(_move));

    // let mut engine = Engine::default();
    // engine.board.set_fen("rnbqkbnr/ppp1pppp/8/4P3/2Pp4/8/PP1P1PPP/RNBQKBNR b KQkq c3 0 3");
    // let _move = engine.board.parse_uci("d4c3").unwrap();
    // println!("{}", engine.board.is_en_passant(_move));
    // println!("{}", engine.board.san(_move));
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    if ["windows"].contains(&env::consts::OS) {
        set_colored_output(false);
    }
    let clock = Instant::now();
    // Parser::main_loop();
    _main();
    let elapsed_time = clock.elapsed().as_secs_f32();
    let precision = 3;
    println_info("\nRun Time", format!("{:.1$} s", elapsed_time, precision));
}
