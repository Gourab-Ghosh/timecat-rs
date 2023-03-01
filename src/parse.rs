use super::*;

const UNKNOWN_COMMAND_ERROR: &str = "";
const NOT_IMPLEMENTED_ERROR: &str = "Sorry, this command is not implemented yet :(";
const BAD_FEN_ERROR: &str = "The given fen is wrong fen! Try again!";
const BAD_BOOL_ERROR: &str = "The given boolean value is wrong fen! Try again!";
const BAD_INETGER_ERROR: &str = "The given integer value is wrong fen! Try again!";
const ILLEGAL_MOVE_ERROR: &str = "The move you are trying to make is illegal! Try again!";
const EMPTY_STACK_ERROR: &str = "Move Stack is enpty, pop not possible! Try again!";

fn generate_error<T: ToString>(error_message: T) -> Option<String> {
    return Some(error_message.to_string());
}

fn parse_raw_input(user_input: &str) -> String {
    let user_input = user_input.trim();
    let mut user_input = user_input.to_string();
    for _char in [",", ":"] {
        user_input = user_input.replace(_char, " ")
    }
    user_input = remove_double_spaces(&user_input);
    user_input
}

pub struct Parser;

impl Parser {
    pub fn main_loop() {
        let mut engine = Engine::default();
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
                if user_input.is_empty() && is_colored_output() {
                    println!();
                }
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
            match parse_command(&mut engine, &user_input) {
                Some(e) => {
                    let error_message = if e.is_empty() {
                        format!("Unknown command: {}\nPlease try again!", user_input.trim())
                    } else {
                        e
                    };
                    println!("{}", colorize(error_message, ERROR_MESSAGE_STYLE));
                }
                None => continue,
            }
        }
    }
}

struct Go;

impl Go {
    fn perft(engine: &mut Engine, depth: u8) -> usize {
        println!("{}\n", engine.board);
        let clock = Instant::now();
        let position_count = engine.board.perft(depth);
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

    fn depth(engine: &mut Engine, depth: u8) -> usize {
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
        println_info("Score", score_to_string(score));
        println_info("PV Line", engine.get_pv_string());
        println_info("Position Count", position_count);
        println_info("Time", format!("{} s", elapsed_time.as_secs_f32()));
        println_info("Speed", nps);
        println_info("Best Move", engine.board.san(best_move));
        position_count
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        let second_command = match commands.get(1) {
            Some(second_command) => second_command.to_lowercase(),
            None => return generate_error(UNKNOWN_COMMAND_ERROR),
        };
        let depth_str = match commands.get(2) {
            Some(depth_str) => depth_str,
            None => return generate_error(UNKNOWN_COMMAND_ERROR),
        };
        let depth: u8 = depth_str.parse().unwrap_or(0);
        if commands.get(3).is_some() {
            return generate_error(UNKNOWN_COMMAND_ERROR);
        }
        if depth == 0 {
            return Some(format!("Invalid depth {depth_str}! Try again!"));
        }
        if second_command == "perft" {
            Self::perft(engine, depth);
            return None;
        } else if second_command == "depth" {
            Self::depth(engine, depth);
            return None;
        }
        generate_error(UNKNOWN_COMMAND_ERROR)
    }
}

struct Set;

impl Set {
    fn board_fen(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        let fen = commands[3..commands.len()].join(" ");
        if !Board::is_good_fen(&fen) {
            return generate_error(BAD_FEN_ERROR);
        };
        engine.board.set_fen(&fen);
        println!("{}", engine.board);
        return None;
    }

    fn color(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        let third_command = match commands.get(2) {
            Some(command) => command.to_lowercase(),
            None => return generate_error(UNKNOWN_COMMAND_ERROR),
        };
        let b = match third_command.parse() {
            Ok(b) => b,
            Err(_) => return generate_error(BAD_BOOL_ERROR),
        };
        if is_colored_output() == b {
            return generate_error(format!("Colored output already set to {}! Try again!", b));
        }
        if b {
            println!();
            set_colored_output(b);
            return None;
        } else {
            set_colored_output(b);
            println!();
            return None;
        }
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        let second_command = match commands.get(1) {
            Some(command) => command.to_lowercase(),
            None => return generate_error(UNKNOWN_COMMAND_ERROR),
        };
        if second_command == "board" {
            let third_command = match commands.get(2) {
                Some(command) => command.to_lowercase(),
                None => return generate_error(UNKNOWN_COMMAND_ERROR),
            };
            if third_command == "fen" {
                return Self::board_fen(engine, commands);
            }
        } else if second_command == "color" {
            if commands.get(3).is_some() {
                return generate_error(UNKNOWN_COMMAND_ERROR);
            }
            return Self::color(engine, commands);
        }
        generate_error(UNKNOWN_COMMAND_ERROR)
    }
}

struct Push;

impl Push {
    fn moves(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        let second_command = match commands.get(1) {
            Some(command) => command.to_lowercase(),
            None => return generate_error(UNKNOWN_COMMAND_ERROR),
        };
        for i in 2..commands.len() {
            let move_text = commands[i];
            let possible_move: Result<Move, ChessError>;
            if second_command == "san" {
                possible_move = engine.board.parse_san(move_text);
            } else if second_command == "uci" {
                possible_move = engine.board.parse_uci(move_text);
            } else if second_command == "move" {
                possible_move = engine.board.parse_move(move_text);
            } else {
                return generate_error(UNKNOWN_COMMAND_ERROR);
            }
            let _move = match possible_move {
                Ok(_move) => _move,
                Err(e) => return Some(e.to_string() + "! Try again!"),
            };
            if !engine.board.is_legal(_move) {
                return generate_error(ILLEGAL_MOVE_ERROR);
            }
            engine.board.push(_move);
            println!(
                "{} {}",
                colorize("Pushed move:", SUCCESS_MESSAGE_STYLE),
                colorize(move_text, INFO_STYLE),
            );
        }
        None
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        Self::moves(engine, commands)
    }
}

struct Pop;

impl Pop {
    fn n_times(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        let second_command = commands.get(1).unwrap_or(&"1");
        if commands.get(2).is_some() {
            return generate_error(UNKNOWN_COMMAND_ERROR);
        }
        let num_pop = match second_command.parse() {
            Ok(p) => p,
            Err(_) => return generate_error(BAD_INETGER_ERROR),
        };
        for _ in 0..num_pop {
            if engine.board.has_empty_stack() {
                return generate_error(EMPTY_STACK_ERROR);
            }
            let last_move = engine.board.pop();
            println!(
                "{} {}",
                colorize("Popped move:", SUCCESS_MESSAGE_STYLE),
                colorize(engine.board.san(last_move), INFO_STYLE),
            );
        }
        None
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &Vec<&str>) -> Option<String> {
        Self::n_times(engine, commands)
    }
}

fn parse_user_input(engine: &mut Engine, user_input: &str) -> Option<String> {
    let commands = Vec::from_iter(user_input.split(' '));
    let first_command = match commands.get(0) {
        Some(command) => command.to_lowercase(),
        None => return generate_error(UNKNOWN_COMMAND_ERROR),
    };
    if user_input.to_lowercase() == "d" {
        println!("{}", engine.board);
        return None;
    }
    if ["go", "do"].contains(&first_command.as_str()) {
        return Go::parse_sub_command(engine, &commands);
    }
    if first_command == "set" {
        return Set::parse_sub_command(engine, &commands);
    }
    if first_command == "push" {
        return Push::parse_sub_command(engine, &commands);
    }
    if first_command == "pop" {
        return Pop::parse_sub_command(engine, &commands);
    }
    generate_error(UNKNOWN_COMMAND_ERROR)
}

fn parse_input(raw_input: &str) -> Vec<String> {
    let modified_raw_str = parse_raw_input(raw_input);
    let inputs = modified_raw_str.split("&&");
    let mut input_vec = Vec::new();
    for input in inputs {
        input_vec.push(input.trim().to_string());
    }
    return input_vec;
}

fn parse_command(engine: &mut Engine, raw_input: &str) -> Option<String> {
    let user_inputs = parse_input(raw_input);
    let mut first_loop = true;
    for user_input in user_inputs {
        if !first_loop {
            println!();
        }
        first_loop = false;
        let response = parse_user_input(engine, &user_input);
        if response.is_some() {
            return response;
        }
    }
    None
}