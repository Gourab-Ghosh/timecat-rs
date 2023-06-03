use std::f32::consts::E;

use super::*;
use ParserError::*;

#[derive(Clone, Debug, Fail)]
pub enum ParserError {
    #[fail(display = "")]
    UnknownCommand,

    #[fail(display = "Sorry, this command is not implemented yet :(")]
    NotImplemented,

    #[fail(display = "Bad FEN string: {}! Try Again!", fen)]
    BadFen { fen: String },

    #[fail(display = "Invalid depth {}! Try again!", depth)]
    InvalidDepth { depth: String },

    #[fail(
        display = "Illegal move {} in position {}! Try again!",
        move_text, board_fen
    )]
    IllegalMove {
        move_text: String,
        board_fen: String,
    },

    #[fail(display = "Colored output already set to {}! Try again!", b)]
    ColoredOutputUnchanged { b: String },

    #[fail(display = "UCI mode already set to {}! Try again!", b)]
    UCIModeUnchanged { b: String },

    #[fail(display = "Move Stack is enpty, pop not possible! Try again!")]
    EmptyStack,

    #[fail(display = "Best move not found in position {}! Try again!", fen)]
    BestMoveNotFound { fen: String },

    #[fail(display = "{}", err_msg)]
    CustomError { err_msg: String },
}

impl ParserError {
    pub fn stringify(&self, raw_input_option: Option<&str>) -> String {
        match self {
            Self::UnknownCommand => match raw_input_option {
                Some(raw_input) => {
                    format!("Unknown command: {}\nPlease try again!", raw_input.trim())
                }
                None => String::from("Unknown command!\nPlease try again!"),
            },
            other_err => format!("{}", other_err),
        }
    }
}

impl From<&Self> for ParserError {
    fn from(error: &Self) -> Self {
        error.clone()
    }
}

impl From<ParseBoolError> for ParserError {
    fn from(error: ParseBoolError) -> Self {
        CustomError {
            err_msg: format!("Failed to parse bool, {}! Try again!", error),
        }
    }
}

impl From<ParseIntError> for ParserError {
    fn from(error: ParseIntError) -> Self {
        CustomError {
            err_msg: format!("Failed to parse integer, {}! Try again!", error),
        }
    }
}

impl From<chess::Error> for ParserError {
    fn from(error: chess::Error) -> Self {
        CustomError {
            err_msg: format!("{}! Try again!", error),
        }
    }
}

const EXIT_CODES: &[&str] = &[
    "q", "quit", "quit()", "quit(0)", "exit", "exit()", "exit(0)",
];

// enum Command {
//     Go,
//     Perft,
//     Depth,
//     SetFen,
//     SetPrint,
//     SetHashSize,
//     SetThreads,
//     SetTime,
//     SetDepth,
//     SetInfinite,
//     SetPonder,
//     SetMoveTime,
//     SetNodes,
//     SetMate,
//     SetMovestogo,
//     SetMultiPV,
//     SetUCI,
//     SetUciAnalyseMode,
//     SetUCIChess960,
//     SetUCIEngineAbout,
//     SetUCIEngineName,
//     SetUCIEngineAuthor,
//     SetUCILimitStrength,
//     SetUCIOpponent,
//     SetUCIShowCurrLine,
//     SetUCIShowRefutations
// }

#[derive(Debug)]
struct Go;

impl Go {
    fn perft(engine: &mut Engine, depth: Depth) -> usize {
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
        println_info("Time", format!("{} s", elapsed_time.as_secs_f64()));
        println_info("Speed", nps);
        position_count
    }

    fn go_command(engine: &mut Engine, go_command: GoCommand) -> Result<(), ParserError> {
        println!("{}\n", engine.board);
        let clock = Instant::now();
        let (Some(best_move), score) = engine.go(go_command, true) else {
            return Err(BestMoveNotFound { fen: engine.board.get_fen() });
        };
        if is_uci_mode() {
            println!("{} {}", colorize("bestmove", INFO_STYLE), engine.board.stringify_move(best_move).unwrap());
        } else {
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
            println_info("Time", format!("{:.3} s", elapsed_time.as_secs_f64()));
            println_info("Speed", nps);
            println_info("Best Move", engine.board.stringify_move(best_move).unwrap());
        }
        Ok(())
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let input = commands.join(" ");
        let modified_input = UCIParser::parse_uci_go_input(engine, &input)?;
        let commands = modified_input.split(" ").collect_vec();
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "infinite" {
            if commands.get(2).is_some() {
                return Err(UnknownCommand);
            }
            return Self::go_command(engine, GoCommand::Infinite);
        }
        let depth_str = commands.get(2).ok_or(UnknownCommand)?.to_string();
        if commands.get(3).is_some() {
            return Err(UnknownCommand);
        }
        if second_command == "perft" {
            let depth = depth_str.parse()?;
            Self::perft(engine, depth);
            return Ok(());
        } else if second_command == "depth" {
            let depth = depth_str.parse()?;
            return Self::go_command(engine, GoCommand::Depth(depth));
        } else if second_command == "movetime" {
            let time = depth_str.parse()?;
            return Self::go_command(engine, GoCommand::Time(Duration::from_millis(time)));
        }
        return Err(UnknownCommand);
    }
}

#[derive(Debug)]
struct Set;

impl Set {
    fn board_fen(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let fen = commands[3..commands.len()].join(" ");
        if !Board::is_good_fen(&fen) {
            return Err(BadFen { fen });
        };
        engine.set_fen(&fen);
        println!("{}", engine.board);
        Ok(())
    }

    fn color(commands: &[&str]) -> Result<(), ParserError> {
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let b = third_command.parse()?;
        if is_colored_output() == b {
            return Err(ColoredOutputUnchanged { b: third_command });
        }
        if b {
            println!();
            set_colored_output(b, true);
        } else {
            set_colored_output(b, true);
            println!();
        }
        Ok(())
    }

    fn ucimode(commands: &[&str]) -> Result<(), ParserError> {
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let b = third_command.parse()?;
        if is_uci_mode() == b {
            return Err(UCIModeUnchanged { b: third_command });
        }
        set_uci_mode(b, true);
        Ok(())
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "board" {
            let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
            if third_command == "startpos" {
                let commands = ["set", "board", "fen", STARTING_FEN];
                return Self::board_fen(engine, &commands);
            } else if third_command == "fen" {
                return Self::board_fen(engine, commands);
            }
        } else if second_command == "color" {
            if commands.get(3).is_some() {
                return Err(UnknownCommand);
            }
            return Self::color(commands);
        } else if second_command == "ucimode" {
            if commands.get(3).is_some() {
                return Err(UnknownCommand);
            }
            return Self::ucimode(commands);
        }
        Err(UnknownCommand)
    }
}

#[derive(Debug)]
struct Push;

impl Push {
    fn moves(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        for move_text in commands.iter().skip(2) {
            let option_move = if second_command == "san" {
                engine.board.parse_san(move_text)?
            } else if second_command == "uci" {
                engine.board.parse_uci(move_text)?
            } else if ["move", "moves"].contains(&second_command.as_str()) {
                engine.board.parse_move(move_text)?
            } else {
                return Err(UnknownCommand);
            };
            if let Some(move_) = option_move {
                if !engine.board.is_legal(move_) {
                    return Err(IllegalMove {
                        move_text: move_text.to_string(),
                        board_fen: engine.board.get_fen(),
                    });
                }
                engine.push(move_);
            } else {
                engine.push(None);
            }
            println!(
                "{} {}",
                colorize("Pushed move:", SUCCESS_MESSAGE_STYLE),
                colorize(move_text, INFO_STYLE),
            );
        }
        Ok(())
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        Self::moves(engine, commands)
    }
}

#[derive(Debug)]
struct Pop;

impl Pop {
    fn n_times(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let second_command = commands.get(1).unwrap_or(&"1");
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        let num_pop = second_command.parse()?;
        for _ in 0..num_pop {
            if engine.board.has_empty_stack() {
                return Err(EmptyStack);
            }
            let last_move = engine.pop();
            println!(
                "{} {}",
                colorize("Popped move:", SUCCESS_MESSAGE_STYLE),
                colorize(engine.board.stringify_move(last_move).unwrap(), INFO_STYLE),
            );
        }
        Ok(())
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        Self::n_times(engine, commands)
    }
}

#[derive(Debug)]
enum ParserLoopState {
    Continue,
    Break,
}

macro_rules! extract_time {
    ($commands:ident, $command:expr) => {
        $commands
            .iter()
            .skip_while(|&&s| s != $command)
            .skip(1)
            .next()
            .map(|s| s.parse())
            .transpose()?
            .map(|t| Duration::from_millis(t))
    };
}

pub fn get_movetime(
    engine: &Engine,
    wtime: Option<Duration>,
    btime: Option<Duration>,
    winc: Option<Duration>,
    binc: Option<Duration>,
    _movestogo: Option<u32>,
) -> Result<Duration, ParserError> {
    let (time, inc) = match engine.board.turn() {
        White => (wtime, winc),
        Black => (btime, binc),
    };
    let search_time = match (time, inc) {
        (Some(time), Some(inc)) => time / 30 + inc / 2,
        (Some(time), None) => time / 30,
        _ => return Err(UnknownCommand),
    }.checked_sub(Duration::from_millis(100)).unwrap_or(Duration::from_millis(0));
    let search_time = search_time / match_interpolate!(10, 1, 32, 2, engine.board.get_num_pieces()) as u32;
    Ok(search_time)
}

struct UCIParser;

impl UCIParser {
    fn parse_uci_go_input(engine: &Engine, input: &str) -> Result<String, ParserError> {
        let lowercased_input = input.to_lowercase();
        let commands = lowercased_input.split_whitespace().collect_vec();
        if commands.first() != Some(&"go")
            || ["perft", "depth", "movetime", "infinite"]
                .iter()
                .filter(|s| commands.contains(&s))
                .count()
                > 1
        {
            return Err(UnknownCommand);
        }
        if commands.contains(&"perft") {
            return Ok(input.to_string());
        }
        if commands.contains(&"infinite") {
            return Ok("go infinite".to_string());
        }
        for command in ["depth", "movetime"] {
            if commands.contains(&command) {
                let mut new_input = format!("go {command} ");
                new_input += commands
                    .iter()
                    .skip_while(|&&s| s != command)
                    .skip(1)
                    .next()
                    .ok_or(UnknownCommand)?;
                return Ok(new_input);
            }
        }
        let wtime = extract_time!(commands, "wtime");
        let btime = extract_time!(commands, "btime");
        let winc = extract_time!(commands, "winc");
        let binc = extract_time!(commands, "binc");
        let movestogo = commands
            .iter()
            .skip_while(|&&s| s != "movestogo")
            .skip(1)
            .next()
            .map(|s| s.parse())
            .transpose()?;
        let movetime = get_movetime(engine, wtime, btime, winc, binc, movestogo)?;
        Ok(format!("go movetime {}", movetime.as_millis()))
    }

    fn parse_uci_position_input(input: &str) -> Result<String, ParserError> {
        let commands = input.split_whitespace().collect_vec();
        if commands.first() != Some(&"position") {
            return Err(UnknownCommand);
        }
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        let mut new_input = String::from("set board fen ");
        match second_command.as_str() {
            "startpos" => {
                new_input += STARTING_FEN;
            }
            "fen" => {
                let fen = commands
                    .iter()
                    .skip(2)
                    .take_while(|&&s| s != "moves")
                    .join(" ");
                new_input += &fen;
            }
            _ => return Err(UnknownCommand),
        }
        let moves = commands
            .iter()
            .skip_while(|&&s| s != "moves")
            .skip(1)
            .join(" ");
        if !moves.is_empty() {
            new_input += " && push moves ";
            new_input += &moves;
        }
        Ok(new_input)
    }

    fn parse_uci_input(engine: &Engine, input: &str) -> Result<String, ParserError> {
        let modified_input = input.trim().to_lowercase();
        if modified_input.starts_with("go") {
            return Self::parse_uci_go_input(engine, input);
        } else if modified_input.starts_with("position") {
            return Self::parse_uci_position_input(input);
        }
        Err(UnknownCommand)
    }

    fn run_parsed_input(engine: &mut Engine, parsed_input: &str) -> Result<(), ParserError> {
        let user_inputs = parsed_input.split("&&").map(|s| s.trim()).collect_vec();
        for user_input in user_inputs {
            Parser::run_command(engine, user_input)?;
        }
        Ok(())
    }

    fn parse_command(engine: &mut Engine, user_input: &str) -> Result<(), ParserError> {
        let commands = user_input.split_whitespace().collect_vec();
        let first_command = commands.get(0).ok_or(UnknownCommand)?.to_lowercase();
        if first_command == "uci" {
            println!("id name {} {}", ENGINE_NAME, ENGINE_VERSION);
            println!("id author {}", ENGINE_AUTHOR);
            println!("uciok");
            return Ok(());
        } else if first_command == "isready" {
            println!("readyok");
            return Ok(());
        } else if first_command == "ucinewgame" {
            return Parser::run_command(engine, "set board fen startpos");
        } else if first_command == "go" {
            return Parser::run_command(engine, user_input);
        } else if ["position"].contains(&first_command.as_str()) {
            let parsed_input = Self::parse_uci_input(engine, user_input)?;
            return Self::run_parsed_input(engine, &parsed_input);
        }
        Err(UnknownCommand)
    }
}

#[derive(Debug)]
pub struct Parser;

impl Parser {
    fn get_input<T: Display>(q: T) -> String {
        if !q.to_string().is_empty() {
            print!("{q}");
            std::io::stdout().flush().unwrap();
        }
        let mut user_input = String::new();
        std::io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line!");
        user_input
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

    fn run_command(engine: &mut Engine, user_input: &str) -> Result<(), ParserError> {
        let commands = Vec::from_iter(user_input.split(' '));
        let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
        if user_input.to_lowercase() == "d" {
            println!("{}", engine.board);
            return Ok(());
        }
        if first_command == "help" {
            return Err(NotImplemented);
        }
        if first_command == "go" {
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
        Err(UnknownCommand)
    }

    pub fn parse_command(engine: &mut Engine, raw_input: &str) -> Result<(), ParserError> {
        let modified_raw_input = Self::parse_raw_input(raw_input);
        let first_command = modified_raw_input
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase();
        if ["uci", "ucinewgame", "isready", "position"].contains(&first_command.as_str()) {
            set_colored_output(false, false);
            set_uci_mode(true, false);
        }
        if is_uci_mode() {
            let message = format!(
                "Unknown UCI command: {}, Trying to find command within default commands!",
                raw_input.trim()
            );
            match UCIParser::parse_command(engine, &modified_raw_input) {
                Err(UnknownCommand) => println!("{}", colorize(message, WARNING_MESSAGE_STYLE)),
                anything_else => return anything_else,
            }
        }
        let user_inputs = modified_raw_input
            .split("&&")
            .map(|s| s.trim())
            .collect_vec();
        let mut first_loop = true;
        for user_input in user_inputs {
            if !first_loop {
                println!();
            }
            first_loop = false;
            let response = Parser::run_command(engine, user_input);
            response?;
        }
        Ok(())
    }

    fn run_raw_input_checked(engine: &mut Engine, raw_input: &str) -> ParserLoopState {
        if raw_input.is_empty() || EXIT_CODES.contains(&raw_input.to_lowercase().trim()) {
            if raw_input.is_empty() && is_colored_output() {
                println!();
            }
            println!(
                "{}",
                colorize("Program ended successfully!", SUCCESS_MESSAGE_STYLE)
            );
            return ParserLoopState::Break;
        }
        if raw_input.trim().is_empty() {
            let error_message = colorize("No input! Please try again!", ERROR_MESSAGE_STYLE);
            println!("{error_message}");
            return ParserLoopState::Continue;
        }
        if let Err(parser_error) = Self::parse_command(engine, raw_input) {
            let error_message = parser_error.stringify(Some(raw_input));
            println!("{}", colorize(error_message, ERROR_MESSAGE_STYLE));
        }
        ParserLoopState::Continue
    }

    pub fn main_loop() {
        let mut engine = Engine::default();
        loop {
            let raw_input: String;
            if !is_uci_mode() {
                println!();
                raw_input = Self::get_input(colorize("Enter Command: ", INPUT_MESSAGE_STYLE));
                println!();
            } else {
                raw_input = Self::get_input("");
            }
            match Self::run_raw_input_checked(&mut engine, &raw_input) {
                ParserLoopState::Break => break,
                ParserLoopState::Continue => continue,
            }
        }
    }
}
