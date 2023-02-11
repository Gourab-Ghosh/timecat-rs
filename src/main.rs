// https://github.com/glinscott/nnue-pytorch
// https://hxim.github.io/Stockfish-Evaluation-Guide/
// https://github.com/topics/nnue

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
// #![allow(private_in_public)]
// #[allow(improper_ctypes)]

mod board;
mod constants;
mod engine;
mod evaluate;
mod nnue;
mod nnue_weights;
mod user_defined_macros;
mod utils;

use board::perft_test::perft;
use board::Board;
use chess::Color::*;
use chess::Piece::*;
use chess::{BitBoard, BoardStatus, ChessMove as Move, Color, Error as ChessError, Piece, Square};
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
use std::fmt;
use std::sync::{Arc, Mutex};
use utils::classes::*;
use utils::command_utils::*;
use utils::string_utils::*;
use utils::unsafe_utils::*;

fn main_loop(board: &mut Board) {
    let mut board = Board::from(board);
    let exit_codes = [
        "q", "quit", "quit()", "quit(0)", "exit", "exit()", "exit(0)",
    ];
    loop {
        let user_input: String;
        if is_colored_output() {
            println!();
            user_input = input(colorize("Enter Command: ", INPUT_MESSAGE_STYLE));
            println!();
        } else {
            user_input = input("");
        }
        if user_input.is_empty() || exit_codes.contains(&user_input.to_lowercase().trim()) {
            println!(
                "{}",
                colorize("Program ended successfully!", SUCCESS_MESSAGE_STYLE)
            );
            break;
        }
        if user_input.trim().is_empty() {
            let error_message = colorize("No input! Please try again!", ERROR_MESSAGE_STYLE);
            println!("{error_message}");
            continue;
        }
        match parse_command(&mut board, &user_input) {
            Some(e) => {
                let error_message: String;
                if e.is_empty() {
                    error_message =
                        format!("Unknown command: {}\nPlease try again!", user_input.trim());
                } else {
                    error_message = e;
                }
                println!("{}", colorize(error_message, ERROR_MESSAGE_STYLE));
            }
            None => continue,
        }
    }
}

fn self_play(depth: u8) {
    let mut engine = Engine::default();
    println!("\n{}\n", engine.board);
    while !engine.board.is_game_over() {
        let (_move, score) = engine.get_best_move_and_score(depth);
        let pv = engine.get_pv_string();
        engine.push(_move);
        println!("\n{}\n", engine.board);
        println!(
            "{_move}: {}, searched {} nodes\nPV line: {}",
            colorize(score_to_string(score), INFO_STYLE),
            colorize(engine.get_num_nodes_searched(), INFO_STYLE),
            colorize(pv, INFO_STYLE),
        );
    }
    println!("Game PGN:\n\n{}", engine.board.get_pgn());
}

fn main() {
    self_play(7);

    // let mut engine = Engine::default();
    // let (best_move, score) = engine.get_best_move_and_score(8);
    // println!("{}: {}\nnum nodes searched: {}", best_move, score_to_string(score), engine.get_num_nodes_searched());

    // parse_command(&mut Board::new(), &"go perft 7".to_string());

    // let weights = load_stockfish_nnue("/home/gg8576/linux/MyFiles/github_files/rust_tutorial/timecat/stockfish_nnue/nn-bc24c101ada0.nnue");
    // println!("{}", weights[0]);

    // println!("{board}");
    // main_loop(&mut board);

    // board.push_sans(vec!("e4", "e5", "Ba6", "bxa6"));

    // let board = Board::new();
    // let eval1 = board.evaluate();
    // let eval2 = evaluate_nnue_from_fen(&board.get_fen());
    // println!("{eval1} {eval2}");

    // let res = unsafe{add_numbers(2.0, 3.0)};
    // print!("{}", res);

    // let cmd = "go perft 6";
    // parse_command(&mut board, &cmd.to_string());

    // board.push_sans(vec!("e4", "e5", "d4", "Bb4"));

    // println!("{}", board);
    // println!("{}", board.to_unicode_string());

    // go_perft(7);

    // for square in BB_RANK_2 {
    //     println!("{}", square.to_index());
    // }
    // println!("{}", BB_RANK_2);

    // let v = generator!(x | x in 1..10, x%2 == 0);
    // for i in v.iter(){
    //     println!("{}", i);
    // }

    // for i in BB_RANK_1 {
    //     println!("{}", i)
    // }
}
