use super::*;

fn simplify_command(user_input: &str) -> String {
    let user_input = user_input.trim();
    let mut user_input = user_input.to_string();
    for _char in [",", ":"] {
        user_input = user_input.replace(_char, " ")
    }
    user_input = remove_double_spaces(&user_input);
    user_input
}

struct Go;

impl Go {
    pub fn perft(board: &mut Board, depth: u8) -> usize {
        println!("{}\n", board);
        let clock = Instant::now();
        let position_count = board.perft(depth);
        let elapsed_time = clock.elapsed();
        let nps = format!(
            "{} Nodes/sec",
            (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
        );
        println!();
        println_info("Position Count", position_count);
        println_info("Time", format!("{} s", elapsed_time.as_secs_f32()));
        println_info("Speed", nps);
        position_count
    }

    pub fn depth(engine: &mut Engine, depth: u8) -> usize {
        println!("{}\n", engine.board);
        let clock = Instant::now();
        let (best_move, score) = engine.go(depth, true);
        let elapsed_time = clock.elapsed();
        let position_count = engine.get_num_nodes_searched();
        let nps = format!(
            "{} Nodes/sec",
            (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
        );
        println!();
        println_info("Best Move", engine.board.san(best_move));
        println_info("Score", score_to_string(score));
        println_info("PV Line", engine.get_pv_string());
        println_info("Position Count", position_count);
        println_info("Time", format!("{} s", elapsed_time.as_secs_f32()));
        println_info("Speed", nps);
        position_count
    }
}

pub fn parse_command(engine: &mut Engine, user_input: &str) -> Option<String> {
    let DEFAULT_ERROR = Some(String::new());
    let NOT_IMPLEMENTED_ERROR = Some(String::from(
        "Sorry, this command is not implemented yet :(",
    ));
    let BAD_FEN_ERROR = Some(String::from("The given fen is wrong fen! Try again!"));
    let BAD_BOOL_ERROR = Some(String::from(
        "The given boolean value is wrong fen! Try again!",
    ));
    let ILLEGAL_MOVE_ERROR = Some(String::from(
        "The move you are trying to make is illegal! Try again!",
    ));

    let user_input = simplify_command(user_input);
    let user_input = user_input.as_str();
    let mut commands = user_input.split(' ');
    let first_command = match commands.next() {
        Some(second_command) => second_command,
        None => return DEFAULT_ERROR,
    }
    .to_lowercase();

    if user_input.to_lowercase() == "d" {
        println!("{}", engine.board);
        return None;
    }

    if ["go", "do"].contains(&first_command.as_str()) {
        let second_command = match commands.next() {
            Some(second_command) => second_command,
            None => return DEFAULT_ERROR,
        }
        .to_lowercase();
        let depth_str = match commands.next() {
            Some(depth_str) => depth_str,
            None => return DEFAULT_ERROR,
        };
        let depth: u8 = depth_str.parse().unwrap_or(0);
        if commands.next().is_some() {
            return DEFAULT_ERROR;
        }
        if depth == 0 {
            return Some("Invalid depth {depth_str}! Try again!".to_string());
        }
        if second_command == "perft" {
            Go::perft(&mut engine.board, depth);
        } else if second_command == "depth" {
            Go::depth(engine, depth);
        } else {
            return DEFAULT_ERROR;
        }
        return None;
    }

    if first_command == "set" {
        let second_command = match commands.next() {
            Some(command) => command,
            None => return DEFAULT_ERROR,
        }
        .to_lowercase();
        if second_command == "board" {
            let third_command = match commands.next() {
                Some(command) => command,
                None => return DEFAULT_ERROR,
            }
            .to_lowercase();
            if third_command == "fen" {
                let mut fen = String::new();
                for fen_part in commands {
                    fen.push_str(fen_part);
                    fen.push(' ');
                }
                if !Board::is_good_fen(&fen) {
                    return BAD_FEN_ERROR;
                }
                engine.board.set_fen(&fen);
                println!("{}", engine.board);
                return None;
            }
        } else if second_command == "color" {
            let third_command = match commands.next() {
                Some(command) => command,
                None => return DEFAULT_ERROR,
            }
            .to_lowercase();
            if commands.next().is_some() {
                return DEFAULT_ERROR;
            }
            if third_command == "true" {
                set_colored_output(true);
            } else if third_command == "false" {
                set_colored_output(false);
            } else {
                return BAD_BOOL_ERROR;
            }
            return None;
        }
    }

    if first_command == "push" {
        let second_command = match commands.next() {
            Some(command) => command,
            None => return DEFAULT_ERROR,
        }
        .to_lowercase();
        for move_text in commands {
            let possible_move: Result<Move, ChessError>;
            if second_command == "san" {
                possible_move = engine.board.parse_san(move_text);
            } else if second_command == "uci" {
                possible_move = engine.board.parse_uci(move_text);
            } else {
                return DEFAULT_ERROR;
            }
            let _move = match possible_move {
                Ok(_move) => _move,
                Err(e) => return Some(e.to_string() + "! Try again!"),
            };
            if !engine.board.is_legal(_move) {
                return ILLEGAL_MOVE_ERROR;
            }
            engine.board.push(_move);
            println!(
                "{} {}",
                colorize("Made move:", SUCCESS_MESSAGE_STYLE),
                colorize(move_text, INFO_STYLE),
            );
        }
        return None;
    }

    DEFAULT_ERROR
}
