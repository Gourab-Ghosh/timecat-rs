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
        if UCI_STATE.is_in_console_mode() {
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
        force_println_info("Position Count", position_count);
        force_println_info("Time", elapsed_time.stringify());
        force_println_info("Speed", nps);
        position_count
    }

    fn extract_depth(depth_str: &str) -> Result<Depth, EngineError> {
        let depth: Depth = depth_str.parse()?;
        if depth.is_negative() {
            return Err(InvalidDepth { depth });
        }
        Ok(depth)
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
            "depth" => Ok(GoCommand::Depth(Self::extract_depth(
                commands.get(2).ok_or(UnknownCommand)?,
            )?)),
            "movetime" => Ok(GoCommand::from_millis(
                commands.get(2).ok_or(UnknownCommand)?.parse()?,
            )),
            "infinite" => Ok(GoCommand::Infinite),
            "ponder" => Ok(GoCommand::Ponder),
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
        if UCI_STATE.is_in_console_mode() {
            println!("{}\n", engine.board);
        }
        let clock = Instant::now();
        let response = engine.go(go_command, true);
        let Some(best_move) = response.get_best_move() else {
            return Err(BestMoveNotFound {
                fen: engine.board.get_fen(),
            });
        };
        let elapsed_time = clock.elapsed();
        let position_count = engine.get_num_nodes_searched();
        let nps = format!(
            "{} Nodes/sec",
            (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
        );
        let pv_string = get_pv_string(&engine.board, response.get_pv());
        if UCI_STATE.is_in_console_mode() {
            println!();
        }
        println_info("Score", response.get_score().stringify());
        println_info("PV Line", pv_string);
        println_info("Position Count", position_count);
        println_info("Time", elapsed_time.stringify());
        println_info("Speed", nps);
        if UCI_STATE.is_in_console_mode() {
            println_info(
                "Best Move",
                best_move.stringify_move(&engine.board).unwrap(),
            );
        } else {
            let mut move_text =
                format_info("bestmove", best_move.stringify_move(&engine.board).unwrap());
            if let Some(ponder_move) = response.get_ponder_move() {
                move_text += " ";
                let mut new_board = engine.board.clone();
                new_board.push(best_move);
                move_text +=
                    &format_info("ponder", ponder_move.stringify_move(&new_board).unwrap());
            }
            println!("{}", move_text);
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
    fn parse_sub_commands(commands: &[&str]) -> Result<(), EngineError> {
        if commands.first().ok_or(UnknownCommand)?.to_lowercase() != "setoption" {
            return Err(UnknownCommand);
        }
        if commands.get(1).ok_or(UnknownCommand)?.to_lowercase() != "name" {
            return Err(UnknownCommand);
        }
        let command_name = commands
            .iter()
            .skip(2)
            .take_while(|&&c| c != "value")
            .join(" ")
            .to_lowercase();
        let value_string = commands
            .iter()
            .skip_while(|&&s| s != "value")
            .skip(1)
            .join(" ");
        UCI_OPTIONS.set_option(&command_name, value_string)
    }
}

#[derive(Debug)]
struct Set;

impl Set {
    fn board_fen(engine: &mut Engine, commands: &[&str]) -> Result<(), EngineError> {
        let mut fen = commands[3..].join(" ");
        if fen == "startpos" {
            fen = STARTING_POSITION_FEN.to_string();
        }
        if !Board::is_good_fen(&fen) {
            return Err(BadFen { fen });
        };
        engine.set_fen(&fen)?;
        if UCI_STATE.is_in_console_mode() {
            println!("{}", engine.board);
        }
        Ok(())
    }

    fn color(commands: &[&str]) -> Result<(), EngineError> {
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let b = third_command.parse()?;
        if UCI_STATE.is_colored_output() == b {
            return Err(ColoredOutputUnchanged { b });
        }
        UCI_STATE.set_colored_output(b, true);
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
            println_info("Pushed move", move_text);
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
            println_info(
                "Popped move",
                last_move.stringify_move(&engine.board).unwrap(),
            );
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
                new_input += STARTING_POSITION_FEN;
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
        println!(
            "{}",
            format!("id name {}", get_engine_version()).colorize(INFO_MESSAGE_STYLE)
        );
        println!(
            "{}",
            format!("id author {}", ENGINE_AUTHOR).colorize(INFO_MESSAGE_STYLE)
        );
        for option in UCI_OPTIONS.get_all_options() {
            println!("{}", option.to_string().colorize(INFO_MESSAGE_STYLE));
        }
        println!("{}", "uciok".colorize(SUCCESS_MESSAGE_STYLE));
    }

    fn parse_command(engine: &mut Engine, user_input: &str) -> Result<(), EngineError> {
        let commands = user_input.split_whitespace().collect_vec();
        let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
        match first_command.as_str() {
            "go" | "setoption" | "stop" | "help" | "d" | "eval" | "debug" => {
                Parser::run_single_command(engine, user_input)
            }
            "uci" => {
                Self::print_info();
                Ok(())
            }
            "isready" => {
                println!("readyok");
                Ok(())
            }
            "ucinewgame" => {
                Self::run_parsed_input(engine, "setoption name Clear Hash && reset board")
            }
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

struct DebugMode;

impl DebugMode {
    fn get_debug_mode(second_command: &str) -> Result<bool, EngineError> {
        match second_command {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err(UnknownDebugCommand {
                command: second_command.to_string(),
            }),
        }
    }

    fn parse_sub_commands(commands: &[&str]) -> Result<(), EngineError> {
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        UCI_STATE.set_debug_mode(Self::get_debug_mode(
            &commands.get(1).ok_or(UnknownCommand)?.to_lowercase(),
        )?);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Parser;

impl Parser {
    pub const EXIT_CODES: [&'static str; 7] = [
        "q", "quit", "quit()", "quit(0)", "exit", "exit()", "exit(0)",
    ];

    fn get_input<T: fmt::Display>(q: T) -> String {
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
        let res = match user_input.to_lowercase().as_str() {
            "d" => Ok(println!("{}", engine.board)),
            "eval" => {
                force_println_info("Current Score", engine.board.evaluate().stringify());
                Ok(())
            }
            "reset board" => engine
                .set_fen(STARTING_POSITION_FEN)
                .map_err(EngineError::from),
            "stop" => Err(EngineNotRunning),
            "help" => {
                println!("{}", Self::get_help_text());
                Ok(())
            }
            _ => Err(UnknownCommand),
        };
        if res != Err(UnknownCommand) {
            return res;
        }
        let commands = Vec::from_iter(user_input.split(' '));
        let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
        match first_command.as_str() {
            "go" => Go::parse_sub_commands(engine, &commands),
            "set" => Set::parse_sub_commands(engine, &commands),
            "setoption" => SetOption::parse_sub_commands(&commands),
            "push" => Push::parse_sub_commands(engine, &commands),
            "pop" => Pop::parse_sub_commands(engine, &commands),
            "selfplay" => SelfPlay::parse_sub_commands(engine, &commands),
            "debug" => DebugMode::parse_sub_commands(&commands),
            _ => Err(UnknownCommand),
        }
    }

    pub fn parse_command(engine: &mut Engine, raw_input: &str) -> Result<(), EngineError> {
        let sanitized_input = Self::sanitize_string(raw_input);
        if UCI_STATE.is_in_console_and_debug_mode() {
            println!();
        }
        if Self::EXIT_CODES.contains(&sanitized_input.as_str()) {
            UCI_STATE.set_engine_termination(true);
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
            UCI_STATE.set_console_mode(false, false);
            UCIParser::parse_command(engine, &first_command)?;
            return Ok(());
        }
        if first_command == "console" {
            if sanitized_input.split_whitespace().nth(1).is_some() {
                return Err(UnknownCommand);
            }
            if UCI_STATE.is_in_console_mode() {
                return Err(ConsoleModeUnchanged);
            }
            UCI_STATE.set_console_mode(true, false);
            return Ok(());
        }
        let user_inputs = sanitized_input.split("&&").map(|s| s.trim()).collect_vec();
        let mut first_loop = true;
        for user_input in user_inputs {
            if UCI_STATE.is_in_console_mode() {
                if !first_loop {
                    println!();
                    first_loop = false;
                }
                Self::run_single_command(engine, user_input)?;
            } else {
                UCIParser::parse_command(engine, user_input)?;
            }
        }
        Ok(())
    }

    fn parse_error_and_print(error: EngineError, optional_raw_input: Option<&str>) {
        let mut error_message = error.stringify_with_optional_raw_input(optional_raw_input);
        if !UCI_STATE.is_in_console_mode() {
            error_message = "info string ".to_string() + &error_message.to_lowercase();
        }
        println!("{}", error_message.colorize(ERROR_MESSAGE_STYLE));
    }

    fn run_raw_input_checked(engine: &mut Engine, raw_input: &str) {
        if raw_input.is_empty() {
            if UCI_STATE.is_in_console_mode() {
                println!("\n");
            }
            UCI_STATE.set_engine_termination(true);
            return;
        }
        if raw_input.trim().is_empty() {
            Self::parse_error_and_print(NoInput, None);
            return;
        }
        if let Err(engine_error) = Self::parse_command(engine, raw_input) {
            Self::parse_error_and_print(engine_error, Some(raw_input));
        }
    }

    fn print_exit_message() {
        println!(
            "{}",
            "Program ended successfully!".colorize(SUCCESS_MESSAGE_STYLE)
        );
    }

    pub fn main_loop() {
        thread::spawn(|| IO_READER.start_reader());
        let mut engine = Engine::default();
        loop {
            if UCI_STATE.terminate_engine() {
                Self::print_exit_message();
                break;
            }
            let raw_input = if UCI_STATE.is_in_console_mode() {
                println!();
                Self::get_input("Enter Command: ".colorize(INPUT_MESSAGE_STYLE))
            } else {
                Self::get_input("")
            };
            Self::run_raw_input_checked(&mut engine, &raw_input);
        }
    }

    pub fn uci_loop() {
        UCI_STATE.set_console_mode(false, false);
        Self::main_loop.run_and_print_time();
    }

    pub fn parse_args_and_run_main_loop(args: &[&str]) {
        if args.contains(&"--uci") {
            UCI_STATE.set_console_mode(false, false);
        }
        if args.contains(&"--no-color") {
            UCI_STATE.set_colored_output(false, false);
        }
        if args.contains(&"--threads") {
            let num_threads = args
                .iter()
                .skip_while(|&arg| !arg.starts_with("--threads"))
                .nth(1)
                .unwrap_or(&"")
                .parse()
                .unwrap_or(UCI_DEFAULT_STATE.get_num_threads());
            UCI_STATE.set_num_threads(num_threads, false);
        }
        if args.contains(&"--help") {
            println!("{}", Self::get_help_text());
            return;
        }
        if args.contains(&"--version") {
            print_engine_version(false);
            return;
        }
        if args.contains(&"--test") {
            test.run_and_print_time().unwrap();
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
        Self::main_loop.run_and_print_time();
    }

    pub fn get_help_text() -> String {
        let help_text = "Sadly, the help text is till now not implemented. But type uci to go into the uci mode and visit the link \"https://backscattering.de/chess/uci/\" to know the necessary commands required to use an uci chess engine.";
        help_text.colorize(ERROR_MESSAGE_STYLE).to_string()
    }
}
