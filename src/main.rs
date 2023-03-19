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

mod board;
mod constants;
mod engine;
mod evaluate;
mod nnue;
mod nnue_rs;
mod nnue_weights;
mod parse;
mod sort;
mod tt;
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
use constants::board_representation::*;
use constants::engine_constants::*;
use constants::fen::*;
use constants::print_style::*;
use constants::square::*;
use constants::types::*;
use engine::Engine;
use evaluate::*;
use failure::Fail;
use fxhash::FxHashMap as HashMap;
use parse::*;
use sort::*;
use std::cmp::{self, Ordering};
use std::convert::From;
use std::env;
use std::fmt::{self, Display};
use std::io::Write;
use std::mem;
use std::num::ParseIntError;
use std::str::FromStr;
use std::str::ParseBoolError;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tt::*;
use useful_macros::*;
use utils::classes::*;
use utils::common_utils::*;
use utils::square_utils::*;
use utils::string_utils::*;
use utils::unsafe_utils::*;

fn calculate_prediction_accuracy(x: f32) -> f32 {
    // (-rms / 5.0).exp()
    let res = 1.0 - 1.0 / (1.0 + ((30.0 - x) / 5.0).exp());
    res * 100.0
}

fn self_play(engine: &mut Engine, depth: Depth, print: bool, move_limit: Option<u16>) {
    let mut time_taken_vec = Vec::new();
    let mut prediction_score_vec = Vec::new();
    println!("\n{}\n", engine.board);
    while !engine.board.is_game_over()
        && engine.board.get_fullmove_number() < move_limit.unwrap_or(u16::MAX)
    {
        let clock = Instant::now();
        let (best_move, score) = engine.go(depth, print);
        let time_elapsed = clock.elapsed();
        let best_move_san = engine.board.san(best_move).unwrap();
        let pv = engine.get_pv_string();
        engine.push(best_move);
        time_taken_vec.push(time_elapsed.as_secs_f32());
        prediction_score_vec.push(score as f32 / PAWN_VALUE as f32);
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
    let mean = time_taken_vec.iter().sum::<f32>() / time_taken_vec.len() as f32;
    let std_err = (time_taken_vec
        .iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f32>()
        / (time_taken_vec.len() - 1) as f32)
        .sqrt();
    let max_time_taken = time_taken_vec
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let min_time_taken = time_taken_vec
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_abs_score = prediction_score_vec
        .iter()
        .map(|x| (x * 100.0) as Score)
        .max()
        .unwrap();
    let min_abs_score = prediction_score_vec
        .iter()
        .map(|x| (x * 100.0) as Score)
        .min()
        .unwrap();
    // let prediction_accuracy = prediction_score_vec.iter().filter(|x| x.abs() < 1.5).count() as f32 / prediction_score_vec.len() as f32 * 100.0;
    let prediction_score_rms = prediction_score_vec
        .iter()
        .map(|&x| x.powi(2))
        .sum::<f32>()
        .sqrt();
    let prediction_accuracy = calculate_prediction_accuracy(prediction_score_rms);
    println!(
        "\n{}:\n\n{}",
        colorize("Game PGN", INFO_STYLE),
        engine.board.get_pgn(),
    );
    println!(
        "\n{}:\n\n{:?}",
        colorize("Time taken for all moves", INFO_STYLE),
        time_taken_vec
            .iter()
            .map(|x| (x * 1000.0).round() / 1000.0)
            .collect::<Vec<f32>>(),
    );
    println!(
        "\n{}:\n\n{:?}\n",
        colorize("Pediction Scores", INFO_STYLE),
        prediction_score_vec,
    );
    println_info("Depth Searched", format!("{:.3}", depth));
    println_info(
        "Time taken per move",
        format!("{:.3} \u{00B1} {:.3} s", mean, std_err),
    );
    println_info("Coefficient of Variation", format!("{:.3}", std_err / mean));
    println_info(
        "Prediction Score RMS",
        format!("{:.3}", prediction_score_rms),
    );
    println_info(
        "Prediction Accuracy",
        format!("{:.1} %", prediction_accuracy),
    );
    println_info("Max time taken", format!("{:.3} s", max_time_taken));
    println_info("Min time taken", format!("{:.3} s", min_time_taken));
    println_info("Max prediction magnitude", score_to_string(max_abs_score));
    println_info("Min prediction magnitude", score_to_string(min_abs_score));
}

pub fn parse_command(engine: &mut Engine, raw_input: &str) {
    Parser::parse_command(engine, raw_input)
        .unwrap_or_else(|err| panic!("{}", err.generate_error(Some(raw_input))))
}

fn _main() {
    // open_tablebase("directory", true, true, None, Board::new());
    let could_have_probably_played_better_move = [
        "5rk1/6pp/p1p5/1p1pqn2/1P6/2NP3P/2PQ1PP1/R5K1 w - - 0 26",
        "4b2k/N7/p1P1rn2/7p/1r1p1p1P/1P3P2/3K4/R2B2R1 b - - 0 42",
    ];

    let time_consuming_fens = [
        "r2qrbk1/2p2ppp/b1p2n2/p2p4/4PB2/P1NB4/1PP2PPP/R2QR1K1 w - - 3 13",
        "2qr2k1/2p2pp1/2p4p/p3b3/8/P6P/1PPBQPP1/4R1K1 w - - 9 23",
        "r2qkb1r/p4pp1/2p4p/8/2n3n1/2NP4/PP2NPP1/R1BQK2R b KQkq - 1 14",
        "r2qk2r/p4pp1/2p4p/2b5/2n3n1/2NP4/PP2NPP1/R1BQK2R w KQkq - 2 15",
        "8/7R/8/8/8/8/2k3K1/8 w - - 4 3",
        "r3r3/3q1pk1/2pn2pp/pp1pR3/3P1P2/P6P/1P2QPP1/3NR1K1 b - - 10 33",
        "4b3/8/8/2K5/8/8/1k6/q7 w - - 0 115", // Taking really long to best move at depth 12
        "6k1/8/8/8/2q5/8/8/1K6 b - - 89 164", // Taking really long to best move at depth 12
        "8/8/8/8/1K6/5k2/8/5q2 b - - 1 75",   // Taking really long to best move at depth 12
        "8/8/q7/2K5/8/5k2/8/8 b - - 3 76",    // Taking really long to best move at depth 12
        "6R1/8/5K2/5N2/8/2k5/8/8 b - - 0 68", // Taking really long to best move at depth 14
    ];

    let mut engine = Engine::default();
    // engine.board.set_fen("8/8/8/1R5K/3k4/8/8/5rq1 b - - 1 96");
    // engine.board.push_sans("e4 e5 Nf3 Nc6"); // e4 opwning
    // engine.board.push_sans("e4 e6 d4 d5"); // caro cann defense
    // engine.board.push_sans("d4 d5 c4"); // queens gambit
    // engine.board.push_sans("d4 d5 c4 dxc4"); // queens gambit accepted
    // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5"); // fried liver attack
    // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5"); // traxer counter attack
    // engine.board.push_sans("e4 e5 Nf3 Nc6 Bc4 Nf6 Ng5 Bc5 Nxf7"); // traxer counter attack with Nxf7
    // engine.board.set_fen("8/6k1/3r4/7p/7P/4R1P1/5P1K/8 w - - 3 59"); // endgame improvement 1
    // engine.board.set_fen("8/7R/8/8/8/7K/k7/8 w - - 0 1"); // endgame improvement 2
    // self_play(&mut engine, 16, false, Some(100));
    self_play(&mut engine, 12, false, None);

    // parse_command(&mut Engine::default(), "go depth 14");
    // parse_command(&mut Engine::default(), "go perft 7");

    // let mut engine = Engine::default();
    // // engine.board.set_fen("6k1/5p2/6p1/1K6/8/8/3r4/7q b - - 1 88"); // test if engine can find mate in 3
    // // engine.board.set_fen("7R/r7/3K4/8/5k2/8/8/8 b - - 80 111"); // test t_table -> nodes initially: 3203606
    // // engine.board.set_fen("8/8/K5k1/2q5/8/1Q6/8/8 b - - 20 105"); // gives incomplete pv line
    // engine.board.set_fen(time_consuming_fens[7]);
    // parse_command(&mut engine, "go depth 14");

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
