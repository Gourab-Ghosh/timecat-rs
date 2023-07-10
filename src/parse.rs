use std::char::MAX;

use super::*;
use EngineError::*;
// use Command::*;

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
        let nps: String = format!(
            "{} nodes/sec",
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
            .filter(|&s| commands.contains(s))
            .count()
            > 1
        {
            return Err(UnknownCommand);
        }
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        for (string, index) in [("depth", 3), ("movetime", 3), ("infinite", 2)] {
            if second_command == string && commands.get(index).is_some() {
                return Err(UnknownCommand);
            }
        }
        match second_command.as_str() {
            "depth" => Ok(GoCommand::Depth(
                commands.get(2).ok_or(UnknownCommand)?.parse()?,
            )),
            "movetime" => Ok(GoCommand::from_millis(
                commands.get(2).ok_or(UnknownCommand)?.parse()?,
            )),
            "infinite" => Ok(GoCommand::Infinite),
            _ => Ok(GoCommand::Timed {
                wtime: extract_time!(commands, "wtime").ok_or(WTimeNotMentioned)?,
                btime: extract_time!(commands, "btime").ok_or(BTimeNotMentioned)?,
                winc: extract_time!(commands, "winc").unwrap_or(Duration::new(0, 0)),
                binc: extract_time!(commands, "binc").unwrap_or(Duration::new(0, 0)),
                moves_to_go: extract_value!(commands, "movestogo"),
            }),
        }
    }

    fn go_command(engine: &mut Engine, go_command: GoCommand) -> Result<(), EngineError> {
        if !is_in_uci_mode() {
            println!("{}\n", engine.board);
        }
        let clock = Instant::now();
        let response = engine.go(go_command, true);
        let Some(best_move) = response.get_best_move() else {
            return Err(BestMoveNotFound { fen: engine.board.get_fen() });
        };
        if is_in_uci_mode() {
            let mut move_text =
                format_info("bestmove", best_move.stringify_move(&engine.board).unwrap());
            if let Some(ponder_move) = response.get_ponder_move() {
                move_text += " ";
                move_text +=
                    &format_info("ponder", ponder_move.stringify_move(&engine.board).unwrap());
            }
            println!("{}", move_text);
        } else {
            println!();
            let elapsed_time = clock.elapsed();
            let position_count = engine.get_num_nodes_searched();
            let nps = format!(
                "{} Nodes/sec",
                (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
            );
            let pv_string = get_pv_string(&engine.board, response.get_pv());
            println_info("Score", response.get_score().stringify());
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
            let depth = commands.get(2).ok_or(UnknownCommand)?.parse()?;
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
    fn extract_value<T, E>(commands: &[&str]) -> Result<T, EngineError>
    where
        T: std::str::FromStr<Err = E>,
        EngineError: From<E>,
    {
        commands
            .iter()
            .skip_while(|&&s| s != "value")
            .nth(1)
            .ok_or(EngineError::UnknownCommand)?
            .parse()
            .map_err(EngineError::from)
    }

    fn threads(commands: &[&str]) -> Result<(), EngineError> {
        let threads = Self::extract_value(commands)?;
        if threads == 0 {
            return Err(ZeroThreads);
        }
        if threads < MIN_NUM_THREADS {
            return Err(MinThreadsExceeded);
        }
        if threads > MAX_NUM_THREADS {
            return Err(MaxThreadsExceeded);
        }
        set_num_threads(threads, true);
        Ok(())
    }

    fn hash(commands: &[&str]) -> Result<(), EngineError> {
        let size = CacheTableSize::Max(Self::extract_value(commands)?);
        if size < MIN_T_TABLE_SIZE {
            return Err(MinHashTableSizeExceeded {
                range: MIN_T_TABLE_SIZE.stringify(),
            });
        }
        if size > MAX_T_TABLE_SIZE {
            return Err(MaxHashTableSizeExceeded {
                range: MAX_T_TABLE_SIZE.stringify(),
            });
        }
        set_t_table_size(size);
        Ok(())
    }

    fn move_overhead(commands: &[&str]) -> Result<(), EngineError> {
        let overhead = Duration::from_millis(Self::extract_value(commands)?);
        if overhead < MIN_MOVE_OVERHEAD {
            return Err(MinMoveOverheadExceeded {
                range: MIN_MOVE_OVERHEAD.stringify(),
            });
        }
        if overhead > MAX_MOVE_OVERHEAD {
            return Err(MaxMoveOverheadExceeded {
                range: MAX_MOVE_OVERHEAD.stringify(),
            });
        }
        set_move_overhead(overhead);
        Ok(())
    }

    fn parse_sub_commands(commands: &[&str]) -> Result<(), EngineError> {
        // setoption name NAME value VALUE
        if commands.get(1).ok_or(UnknownCommand)?.to_lowercase() != "name" {
            return Err(UnknownCommand);
        }
        let command_name = commands
            .iter()
            .skip(2)
            .take_while(|&&c| c != "value")
            .join(" ")
            .to_lowercase();
        match command_name.as_str() {
            "thread" | "threads" => Self::threads(commands),
            "hash" => Self::hash(commands),
            "move overhead" => Self::move_overhead(commands),
            "clear hash" => {
                clear_all_hash_tables();
                Ok(())
            }
            "multipv" => Err(NotImplemented),
            _ => Err(UnknownCommand),
        }
    }

    fn parse_commands(commands: &[&str]) -> Result<(), EngineError> {
        if commands.first().ok_or(UnknownCommand)?.to_lowercase() != "setoption" {
            return Err(UnknownCommand);
        }
        Self::parse_sub_commands(commands)
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
        let mut fen = commands[3..].join(" ");
        if fen == "startpos" {
            fen = STARTING_BOARD_FEN.to_string();
        }
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
        set_colored_output(b, true);
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
        match second_command.as_str() {
            "board" => {
                let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
                match third_command.as_str() {
                    "startpos" | "fen" => Self::board_fen(engine, commands),
                    _ => Err(UnknownCommand),
                }
            }
            "color" => Self::color(commands),
            "ucimode" => Self::ucimode(commands),
            _ => Err(UnknownCommand),
        }
    }
}

#[derive(Debug)]
struct Push;

impl Push {
    fn moves(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        for move_text in commands.iter().skip(2) {
            let optional_move = match second_command.as_str() {
                "san" => engine.board.parse_san(move_text)?,
                "uci" => engine.board.parse_uci(move_text)?,
                "move" | "moves" => engine.board.parse_move(move_text)?,
                _ => return Err(UnknownCommand),
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
                println_info("Pushed move", move_text);
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
                println_info(
                    "Popped move",
                    last_move.stringify_move(&engine.board).unwrap(),
                );
            }
        }
        Ok(())
    }

    pub fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        Self::n_times(engine, commands)
    }
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
                new_input += STARTING_BOARD_FEN;
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
        if modified_input.starts_with("position") {
            return Self::parse_uci_position_input(input);
        }
        Err(UnknownCommand)
    }

    fn run_parsed_input(engine: &mut Engine, parsed_input: &str) -> Result<(), EngineError> {
        let user_inputs = parsed_input.split("&&").map(|s| s.trim()).collect_vec();
        for user_input in user_inputs {
            Parser::run_single_command(engine, user_input)?;
        }
        Ok(())
    }

    fn print_info() {
        println!("id name {}", get_engine_version());
        println!("id author {}", ENGINE_AUTHOR);
        println!(
            "option name Threads type spin default {} min {} max {}",
            DEFAULT_NUM_THREADS, MIN_NUM_THREADS, MAX_NUM_THREADS
        );
        println!(
            "option name Hash type spin default {} min {} max {}",
            DEFAULT_T_TABLE_SIZE.stringify(),
            MIN_T_TABLE_SIZE.stringify(),
            MAX_T_TABLE_SIZE.stringify(),
        );
        println!("option name Clear Hahs type button");
        println!(
            "option name Move Overhead type spin default {} min {} max {}",
            DEFAULT_MOVE_OVERHEAD.stringify(),
            MIN_MOVE_OVERHEAD.stringify(),
            MAX_MOVE_OVERHEAD.stringify(),
        );
        // println!("option name OwnBook type check default {DEFAULT_USE_OWN_BOOK}");
        println!("uciok");
    }

    fn parse_command(engine: &mut Engine, user_input: &str) -> Result<(), EngineError> {
        let commands = user_input.split_whitespace().collect_vec();
        let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
        match first_command.as_str() {
            "go" | "setoption" | "stop" => Parser::run_single_command(engine, user_input),
            "uci" => {
                Self::print_info();
                Ok(())
            }
            "isready" => Ok(println!("readyok")),
            "ucinewgame" => Self::run_parsed_input(
                engine,
                &format!("setoption name Clear Hash && set board fen {STARTING_BOARD_FEN}"),
            ),
            "position" => Self::run_parsed_input(engine, &Self::parse_uci_input(user_input)?),
            _ => Err(UnknownCommand),
        }
    }
}

struct SelfPlay;

impl SelfPlay {
    fn parse_sub_commands(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let mut commands = commands.to_vec();
        commands[0] = "go";
        let go_command = if commands.get(1).is_some() {
            Go::extract_go_command(&commands)?
        } else {
            DEFAULT_SELFPLAY_COMMAND
        };
        self_play(engine, go_command, true, None)
    }
}

#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub const EXIT_CODES: [&str; 7] = [
        "q", "quit", "quit()", "quit(0)", "exit", "exit()", "exit(0)",
    ];

    fn get_input<T: Display>(q: T) -> String {
        print_line(q);
        IO_READER.read_line()
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

    fn run_single_command(engine: &mut Engine, user_input: &str) -> Result<(), EngineError> {
        let res = match user_input {
            "d" => Ok(println!("{}", engine.board)),
            "eval" => {
                println_info("Current Score", engine.board.evaluate().stringify());
                Ok(())
            }
            "reset board" => engine
                .set_fen(STARTING_BOARD_FEN)
                .map_err(EngineError::from),
            "stop" => {
                if is_in_uci_mode() {
                    Ok(())
                } else {
                    Err(EngineNotRunning)
                }
            }
            _ => Err(UnknownCommand),
        };
        if res != Err(UnknownCommand) {
            return res;
        }
        let commands = Vec::from_iter(user_input.split(' '));
        let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
        match first_command.as_str() {
            "help" => Err(NotImplemented),
            "go" => Go::parse_sub_commands(engine, &commands),
            "set" => Set::parse_sub_commands(engine, &commands),
            "setoption" => SetOption::parse_sub_commands(&commands),
            "push" => Push::parse_sub_commands(engine, &commands),
            "pop" => Pop::parse_sub_commands(engine, &commands),
            "selfplay" => SelfPlay::parse_sub_commands(engine, &commands),
            _ => Err(UnknownCommand),
        }
    }

    pub fn parse_command(engine: &mut Engine, raw_input: &str) -> Result<(), EngineError> {
        let sanitized_input = Self::sanitize_string(raw_input);
        if Self::EXIT_CODES.contains(&sanitized_input.as_str()) {
            set_engine_termination(true);
            return Ok(());
        }
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
            Self::run_single_command(engine, user_input)?;
        }
        Ok(())
    }

    fn parse_error(error: EngineError, optional_raw_input: Option<&str>) {
        let error_message = error.stringify_with_optional_raw_input(optional_raw_input);
        println!("{}", colorize(error_message, ERROR_MESSAGE_STYLE));
    }

    fn run_raw_input_checked(engine: &mut Engine, raw_input: &str) {
        if raw_input.is_empty() {
            if !is_in_uci_mode() {
                println!();
            }
            set_engine_termination(true);
            return;
        }
        if raw_input.trim().is_empty() {
            Self::parse_error(NoInput, None);
            return;
        }
        if let Err(engine_error) = Self::parse_command(engine, raw_input) {
            Self::parse_error(engine_error, Some(raw_input));
        }
    }

    fn print_exit_message() {
        println!(
            "{}",
            colorize("Program ended successfully!", SUCCESS_MESSAGE_STYLE)
        );
    }

    pub fn main_loop() {
        let mut engine = Engine::default();
        loop {
            if terminate_engine() {
                Self::print_exit_message();
                break;
            }
            let raw_input = if is_in_uci_mode() {
                Self::get_input("")
            } else {
                println!();
                let message = colorize("Enter Command: ", INPUT_MESSAGE_STYLE);
                let raw_input = Self::get_input(if is_in_uci_mode() { "" } else { &message });
                println!();
                raw_input
            };
            Self::run_raw_input_checked(&mut engine, &raw_input);
        }
    }

    pub fn uci_loop() {
        enable_uci_and_disable_color();
        Self::main_loop();
    }

    pub fn parse_args_and_run_main_loop(args: &[&str]) {
        thread::spawn(|| IO_READER.start_reader());
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
                .nth(1)
                .unwrap_or(&"")
                .parse()
                .unwrap_or(DEFAULT_NUM_THREADS);
            set_num_threads(num_threads, false);
        }
        if args.contains(&"--help") {
            println!("{}", Self::get_help_text());
        }
        if args.contains(&"--version") {
            print_engine_version(false);
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
        print_engine_info();
        Self::main_loop();
    }

    pub fn get_help_text() -> String {
        String::new()
    }
}
