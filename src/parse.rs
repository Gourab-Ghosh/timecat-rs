use std::char::MAX;

use super::*;
use EngineError::*;
// use Command::*;

const EXIT_CODES: &[&str] = &[
    "q", "quit", "quit()", "quit(0)", "exit", "exit()", "exit(0)",
];

// enum Command<'a> {
//     Go(GoCommand),
//     MakeMove(&'a [Move]),
//     UndoMove(u16),
//     SetFen(String),
//     SetColor(bool),
//     SetUCIMode(bool),
//     SetHashSize(u64),
//     SetThreads(u8),
//     SetMultiPV(u8),
//     SetUCIElo(u16),
//     SetEngineMode(EngineMode),
//     // SetPrint,
//     // SetUciAnalyzeMode,
//     // SetUCIChess960,
//     // SetUCIOpponent,
//     // SetUCIShowCurrLine,
//     // SetUCIShowRefutations
// }

macro_rules! extract_value {
    ($commands:ident, $command:expr) => {
        $commands
            .iter()
            .skip_while(|&&s| s != $command)
            .skip(1)
            .next()
            .map(|s| s.parse())
            .transpose()?
    };
}

macro_rules! extract_time {
    ($commands:ident, $command:expr) => {
        extract_value!($commands, $command).map(|t| Duration::from_millis(t))
    };
}

#[derive(Debug)]
struct Go;

impl Go {
    fn perft(engine: &mut Engine, depth: Depth) -> usize {
        if !is_in_uci_mode() {
            println!("{}\n", engine.board);
        }
        let clock = Instant::now();
        let position_count = engine.board.perft(depth);
        let elapsed_time = clock.elapsed();
        let nps = format!(
            "{} Nodes/sec",
            (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
        );
        println!();
        println_info("Position Count", position_count);
        println_info("Time", elapsed_time.stringify());
        println_info("Speed", nps);
        position_count
    }

    pub fn extract_go_command(commands: &[&str]) -> Result<GoCommand, EngineError> {
        // TODO: Improve Unknown Command Detection
        if ["perft", "depth", "movetime", "infinite"]
            .iter()
            .filter(|s| commands.contains(s))
            .count()
            > 1
        {
            return Err(UnknownCommand);
        }
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "infinite" {
            if commands.get(2).is_some() {
                return Err(UnknownCommand);
            }
            return Ok(GoCommand::Infinite);
        }
        let int_str = commands.get(2).ok_or(UnknownCommand)?.to_string();
        if second_command == "depth" {
            if commands.get(3).is_some() {
                return Err(UnknownCommand);
            }
            let depth = int_str.parse()?;
            return Ok(GoCommand::Depth(depth));
        } else if second_command == "movetime" {
            if commands.get(3).is_some() {
                return Err(UnknownCommand);
            }
            let time = int_str.parse()?;
            return Ok(GoCommand::MoveTime(Duration::from_millis(time)));
        }
        Ok(GoCommand::Timed {
            wtime: extract_time!(commands, "wtime").ok_or(WTimeNotMentioned)?,
            btime: extract_time!(commands, "btime").ok_or(BTimeNotMentioned)?,
            winc: extract_time!(commands, "winc").unwrap_or(Duration::new(0, 0)),
            binc: extract_time!(commands, "binc").unwrap_or(Duration::new(0, 0)),
            moves_to_go: extract_value!(commands, "movestogo"),
        })
    }

    fn go_command(engine: &mut Engine, go_command: GoCommand) -> Result<(), EngineError> {
        if !is_in_uci_mode() {
            println!("{}\n", engine.board);
        }
        let clock = Instant::now();
        let (Some(best_move), score) = engine.go(go_command, true) else {
            return Err(BestMoveNotFound { fen: engine.board.get_fen() });
        };
        let ponder_move = engine.get_ponder_move();
        if is_in_uci_mode() {
            let best_move_text = format!(
                "{} {}",
                colorize("bestmove", INFO_STYLE),
                best_move.stringify_move(&engine.board).unwrap()
            );
            let to_print = if let Some(ponder_move) = ponder_move {
                format!(
                    "{} {} {}",
                    best_move_text,
                    colorize("ponder", INFO_STYLE),
                    ponder_move.stringify_move(&engine.board).unwrap()
                )
            } else {
                best_move_text
            };
            println!("{}", to_print);
        } else {
            println!();
            let elapsed_time = clock.elapsed();
            let position_count = engine.get_num_nodes_searched();
            let nps = format!(
                "{} Nodes/sec",
                (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
            );
            let pv_string = SearchInfo::get_pv_string(&engine.board, &engine.get_pv());
            println_info("Score", score.stringify());
            println_info("PV Line", pv_string);
            println_info("Position Count", position_count);
            println_info("Time", elapsed_time.stringify());
            println_info("Speed", nps);
            println_info(
                "Best Move",
                best_move.stringify_move(&engine.board).unwrap(),
            );
        }
        Ok(())
    }

    pub fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "perft" {
            let depth = commands.get(2).ok_or(UnknownCommand)?.to_string().parse()?;
            Self::perft(engine, depth);
            Ok(())
        } else {
            Self::go_command(engine, Self::extract_go_command(commands)?)
        }
    }
}

#[derive(Debug)]
pub struct SetOption;

impl SetOption {
    fn threads(commands: &[&str]) -> Result<(), EngineError> {
        let threads = commands.iter().skip_while(|&&s| s == "value").skip(1).next().ok_or(UnknownCommand)?.parse()?;
        if threads == 0 {
            return Err(ZeroThreads);
        }
        if threads > MAX_NUM_THREADS {
            return Err(MaxThreadsExceeded);
        }
        set_num_threads(threads, true);
        Ok(())
    }

    fn hash(commands: &[&str]) -> Result<(), EngineError> {
        let hash = commands.iter().skip_while(|&&s| s == "value").skip(1).next().ok_or(UnknownCommand)?.parse()?;
        if hash == 0 {
            return Err(ZeroThreads);
        }
        if hash > MAX_NUM_THREADS {
            return Err(MaxThreadsExceeded);
        }
        set_num_threads(hash, true);
        Ok(())
    }

    fn parse_sub_commands(commands: &[&str]) -> Result<(), EngineError> {
        // setoption name NAME value VALUE
        if commands.get(1).ok_or(UnknownCommand)?.to_lowercase() != "name" {
            return Err(UnknownCommand);
        }
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        if ["thread", "threads"].contains(&third_command.as_str()) {
            return Self::threads(commands);
        }
        if third_command == "hash" {
            return Err(NotImplemented);
        }
        if ["MultiPV"].contains(&third_command.as_str()) {
            return Err(NotImplemented);
        }
        Err(UnknownCommand)
    }

    fn parse_commands(commands: &[&str]) -> Result<(), EngineError> {
        if commands.get(0).ok_or(UnknownCommand)?.to_lowercase() != "setoption" {
            return Err(UnknownCommand);
        }
        Self::parse_sub_commands(&commands)
    }

    fn parse_input(input: &str) {
        let sanitized_input = Parser::sanitize_string(input);
        let commands: Vec<&str> = sanitized_input.split_ascii_whitespace().collect_vec();
        if let Err(error) = Self::parse_commands(&commands) {
            Parser::parse_error(error, Some(input));
        }
    }
}

#[derive(Debug)]
struct Set;

impl Set {
    fn board_fen(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let fen = commands[3..commands.len()].join(" ");
        if !Board::is_good_fen(&fen) {
            return Err(BadFen { fen });
        };
        engine.set_fen(&fen)?;
        if !is_in_uci_mode() {
            println!("{}", engine.board);
        }
        Ok(())
    }

    fn color(commands: &[&str]) -> Result<(), EngineError> {
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

    fn ucimode(commands: &[&str]) -> Result<(), EngineError> {
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let b = third_command.parse()?;
        if is_in_uci_mode() == b {
            return Err(UCIModeUnchanged { b: third_command });
        }
        set_uci_mode(b, true);
        Ok(())
    }

    pub fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
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
    fn moves(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        for move_text in commands.iter().skip(2) {
            let optional_move = if second_command == "san" {
                engine.board.parse_san(move_text)?
            } else if second_command == "uci" {
                engine.board.parse_uci(move_text)?
            } else if ["move", "moves"].contains(&second_command.as_str()) {
                engine.board.parse_move(move_text)?
            } else {
                return Err(UnknownCommand);
            };
            if let Some(move_) = optional_move {
                if !engine.board.is_legal(move_) {
                    return Err(IllegalMove {
                        move_text: move_text.to_string(),
                        board_fen: engine.board.get_fen(),
                    });
                }
                engine.board.push(move_);
            } else {
                if engine.board.is_check() {
                    return Err(NullMoveInCheck {
                        fen: engine.board.get_fen(),
                    });
                }
                engine.board.push(None);
            }
            if !is_in_uci_mode() {
                println!(
                    "{} {}",
                    colorize("Pushed move:", SUCCESS_MESSAGE_STYLE),
                    colorize(move_text, INFO_STYLE),
                );
            }
        }
        Ok(())
    }

    pub fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        Self::moves(engine, commands)
    }
}

#[derive(Debug)]
struct Pop;

impl Pop {
    fn n_times(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let second_command = commands.get(1).unwrap_or(&"1");
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        let num_pop = second_command.parse()?;
        for _ in 0..num_pop {
            if engine.board.has_empty_stack() {
                return Err(EmptyStack);
            }
            let last_move = engine.board.pop();
            if !is_in_uci_mode() {
                println!(
                    "{} {}",
                    colorize("Popped move:", SUCCESS_MESSAGE_STYLE),
                    colorize(last_move.stringify_move(&engine.board).unwrap(), INFO_STYLE),
                );
            }
        }
        Ok(())
    }

    pub fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        Self::n_times(engine, commands)
    }
}

#[derive(Debug)]
enum ParserLoopState {
    Continue,
    Break,
}

struct UCIParser;

impl UCIParser {
    fn parse_uci_position_input(input: &str) -> Result<String, EngineError> {
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

    fn parse_uci_input(input: &str) -> Result<String, EngineError> {
        let modified_input = input.trim().to_lowercase();
        if modified_input.starts_with("go") {
            return Ok(modified_input);
        }
        if modified_input.starts_with("position") {
            return Self::parse_uci_position_input(input);
        }
        Err(UnknownCommand)
    }

    fn run_parsed_input(engine: &mut Engine, parsed_input: &str) -> Result<(), EngineError> {
        let user_inputs = parsed_input.split("&&").map(|s| s.trim()).collect_vec();
        for user_input in user_inputs {
            Parser::run_command(engine, user_input)?;
        }
        Ok(())
    }

    fn print_info() {
        println!("id name {}", INFO_TEXT);
        println!("id author {}", ENGINE_AUTHOR);
        println!(
            "option name Threads type spin default {} min 1 max {}",
            INITIAL_NUM_THREADS, MAX_NUM_THREADS
        );
        // println!(
        //     "option name Hash type spin default {} min 1 max {}",
        //     INITIAL_T_TABLE_SIZE.to_cache_table_memory_size::<TranspositionTableEntry>(),
        //     MAX_T_TABLE_SIZE.to_cache_table_memory_size::<TranspositionTableEntry>(),
        // );
        println!("uciok");
    }

    fn parse_command(engine: &mut Engine, user_input: &str) -> Result<(), EngineError> {
        let commands = user_input.split_whitespace().collect_vec();
        let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
        if first_command == "uci" {
            Self::print_info();
            return Ok(());
        } else if first_command == "isready" {
            println!("readyok");
            return Ok(());
        } else if first_command == "ucinewgame" {
            return Parser::run_command(engine, &format!("set board fen {}", STARTING_FEN));
        } else if ["position", "go"].contains(&first_command.as_str()) {
            let parsed_input = Self::parse_uci_input(user_input)?;
            return Self::run_parsed_input(engine, &parsed_input);
        }
        Err(UnknownCommand)
    }
}

struct SelfPlay;

impl SelfPlay {
    fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let go_command = if commands.get(1).is_some() {
            Go::extract_go_command(commands)?
        } else {
            GoCommand::MoveTime(Duration::from_secs(3))
        };
        self_play(engine, go_command, true, None)
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

    fn sanitize_string(user_input: &str) -> String {
        let user_input = user_input.trim();
        let mut user_input = user_input.to_string();
        for _char in [",", ":"] {
            user_input = user_input.replace(_char, " ")
        }
        user_input = remove_double_spaces_and_trim(&user_input);
        user_input
    }

    fn run_command(engine: &mut Engine, user_input: &str) -> Result<(), EngineError> {
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
            return Go::parse_sub_commands(engine, &commands);
        }
        if first_command == "set" {
            return Set::parse_sub_commands(engine, &commands);
        }
        if first_command == "setoption" {
            return SetOption::parse_sub_commands(&commands);
        }
        if first_command == "push" {
            return Push::parse_sub_commands(engine, &commands);
        }
        if first_command == "pop" {
            return Pop::parse_sub_commands(engine, &commands);
        }
        if user_input == "eval" {
            println_info("Current Score", engine.board.evaluate().stringify());
            return Ok(());
        }
        if user_input == "reset board" {
            engine.set_fen(STARTING_FEN)?;
            return Ok(());
        }
        if first_command == "selfplay" {
            return SelfPlay::parse_sub_commands(engine, &commands);
        }
        Err(UnknownCommand)
    }

    pub fn parse_command(engine: &mut Engine, raw_input: &str) -> Result<(), EngineError> {
        let sanitized_input = Self::sanitize_string(raw_input);
        let first_command = sanitized_input
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_lowercase();
        if ["uci", "ucinewgame"].contains(&first_command.as_str()) {
            if sanitized_input.split_whitespace().nth(1).is_some() {
                return Err(UnknownCommand);
            }
            enable_uci_and_disable_color();
        }
        if is_in_uci_mode() {
            let message = format!(
                "Unknown UCI command: {}, Trying to find command within default commands!",
                raw_input.trim()
            );
            match UCIParser::parse_command(engine, &sanitized_input) {
                Err(UnknownCommand) => println!("{}", colorize(message, WARNING_MESSAGE_STYLE)),
                anything_else => return anything_else,
            }
        }
        let user_inputs = sanitized_input.split("&&").map(|s| s.trim()).collect_vec();
        let mut first_loop = true;
        for user_input in user_inputs {
            if !first_loop {
                println!();
                first_loop = false;
            }
            Self::run_command(engine, user_input)?;
        }
        Ok(())
    }

    fn parse_error(error: EngineError, optional_raw_input: Option<&str>) {
        let error_message = error.stringify(optional_raw_input);
        println!("{}", colorize(error_message, ERROR_MESSAGE_STYLE));
    }

    fn run_raw_input_checked(engine: &mut Engine, raw_input: &str) -> ParserLoopState {
        if raw_input.is_empty() || EXIT_CODES.contains(&raw_input.to_lowercase().trim()) {
            if raw_input.is_empty() && is_colored_output() && !is_in_uci_mode() {
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
        if let Err(engine_error) = Self::parse_command(engine, raw_input) {
            Self::parse_error(engine_error, Some(raw_input));
        }
        ParserLoopState::Continue
    }

    pub fn main_loop() {
        let mut engine = Engine::default();
        loop {
            let raw_input = if is_in_uci_mode() {
                Self::get_input("")
            } else {
                println!();
                let message = colorize("Enter Command: ", INPUT_MESSAGE_STYLE);
                let raw_input = Self::get_input(if is_in_uci_mode() { "" } else { &message });
                println!();
                raw_input
            };
            match Self::run_raw_input_checked(&mut engine, &raw_input) {
                ParserLoopState::Break => break,
                ParserLoopState::Continue => continue,
            }
        }
    }

    pub fn uci_loop() {
        enable_uci_and_disable_color();
        Self::main_loop();
    }

    pub fn parse_args_and_run_main_loop(args: &[&str]) {
        if args.contains(&"--uci") {
            set_uci_mode(true, false);
        }
        if args.contains(&"--no-color") {
            set_colored_output(false, false);
        }
        if args.contains(&"--threads") {
            let num_threads = args
                .iter()
                .skip_while(|&arg| !arg.starts_with("--threads"))
                .skip(1)
                .next()
                .unwrap_or(&"1")
                .parse()
                .unwrap_or(1);
            set_num_threads(num_threads, false);
        }
        if args.contains(&"--help") {
            println!("{}", Self::get_help_text());
        }
        if args.contains(&"--version") {
            println!("{}", INFO_TEXT);
        }
        if args.contains(&"--test") {
            test().unwrap();
            return;
        }
        if args.contains(&"-c") || args.contains(&"--command") {
            let command = args
                .iter()
                .skip_while(|&arg| !["-c", "--command"].contains(arg))
                .skip(1)
                .take_while(|&&arg| !arg.starts_with("--"))
                .join(" ");
            let mut engine = Engine::default();
            println!();
            Self::run_raw_input_checked(&mut engine, &command);
            return;
        }
        println!("{}\n", colorize(INFO_TEXT, SUCCESS_MESSAGE_STYLE));
        Self::main_loop();
    }

    pub fn get_help_text() -> String {
        String::new()
    }
}
