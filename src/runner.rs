use super::*;
use TimecatError::*;

enum TimecatBuilderAction {
    PrintHelpCommand,
    PrintEngineVersion,
    #[cfg(feature = "debug")]
    RunTest,
    RunCommand(String),
}

#[derive(Default)]
pub struct TimecatBuilder {
    actions: Vec<TimecatBuilderAction>,
    engine: Option<Engine>,
}

impl TimecatBuilder {
    pub fn build(self) -> Timecat {
        let io_reader = IoReader::default();
        Timecat {
            actions: self.actions,
            engine: self
                .engine
                .unwrap_or_default()
                .with_io_reader(io_reader.clone()),
            io_reader,
            uci_options: UCIOptions::default(),
        }
    }

    pub fn parse_args(mut self, args: &[&str]) -> Self {
        if args.contains(&"--uci") {
            GLOBAL_UCI_STATE.set_to_uci_mode();
        }
        #[cfg(feature = "colored_output")]
        if args.contains(&"--no-color") {
            GLOBAL_UCI_STATE.set_colored_output(false, false);
        }
        if args.contains(&"--threads") {
            let num_threads = args
                .iter()
                .skip_while(|&arg| !arg.starts_with("--threads"))
                .nth(1)
                .unwrap_or(&"")
                .parse()
                .unwrap_or(GlobalUCIState::default().get_num_threads());
            GLOBAL_UCI_STATE.set_num_threads(num_threads, false);
        }
        if args.contains(&"--help") {
            self.actions.push(TimecatBuilderAction::PrintHelpCommand);
            return self;
        }
        if args.contains(&"--version") {
            self.actions.push(TimecatBuilderAction::PrintEngineVersion);
            return self;
        }
        #[cfg(feature = "debug")]
        if args.contains(&"--test") {
            self.actions.push(TimecatBuilderAction::RunTest);
            return self;
        }
        if args.contains(&"-c") || args.contains(&"--command") {
            let command = args
                .iter()
                .skip_while(|&arg| !["-c", "--command"].contains(arg))
                .skip(1)
                .take_while(|&&arg| !arg.starts_with("--"))
                .join(" ");
            self.actions.push(TimecatBuilderAction::RunCommand(command));
            return self;
        }
        self
    }
}

pub struct Timecat {
    actions: Vec<TimecatBuilderAction>,
    engine: Engine,
    io_reader: IoReader,
    uci_options: UCIOptions,
}

impl Timecat {
    pub fn run(mut self) -> Result<()> {
        self.io_reader.start_reader();
        #[allow(clippy::never_loop)]
        for action in self.actions.into_iter() {
            match action {
                TimecatBuilderAction::PrintHelpCommand => {
                    println!("{}", Parser::get_help_message());
                    return Ok(());
                }
                TimecatBuilderAction::PrintEngineVersion => {
                    print_engine_version(false);
                    return Ok(());
                }
                #[cfg(feature = "debug")]
                TimecatBuilderAction::RunTest => {
                    test.run_and_print_time(&mut self.engine)?;
                    return Ok(());
                }
                TimecatBuilderAction::RunCommand(command) => {
                    println!();
                    Parser::run_raw_input_checked(&mut self.engine, &command);
                    return Ok(());
                }
            }
        }
        print_engine_info(self.engine.get_transposition_table());
        Parser::main_loop.run_and_print_time(&mut self.engine, &self.io_reader);
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum UserCommand {
    TerminateEngine,
    SetUCIMode(bool),
    SetDebugMode(bool),
    DisplayBoard,
    EvaluateBoard,
    UCI,
    UCINewGame,
    IsReady,
    Stop,
    Help,
    Perft(Depth),
    Go(GoCommand),
    MakeMoves(String, Vec<String>),
    UndoMoves(u16),
    SetFen(String),
    SetColor(bool),
    SetUCIOption { name: String, value: String },
    SelfPlay(GoCommand),
    // SetHashSize(u64),
    // SetThreads(u8),
    // SetMultiPV(u8),
    // SetUCIElo(u16),
    // SetEngineMode(EngineMode),
    // SetPrint,
    // SetUciAnalyzeMode,
    // SetUCIChess960,
    // SetUCIOpponent,
    // SetUCIShowCurrLine,
    // SetUCIShowRefutations
}

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

struct Go;

impl Go {
    fn extract_depth(depth_str: &str) -> Result<Depth> {
        let depth: Depth = depth_str.parse()?;
        if depth.is_negative() {
            return Err(InvalidDepth { depth });
        }
        Ok(depth)
    }

    fn extract_go_command(commands: &[&str]) -> Result<GoCommand> {
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

    pub fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "perft" {
            Ok(UserCommand::Perft(
                commands.get(2).ok_or(UnknownCommand)?.parse()?,
            ))
        } else {
            Ok(UserCommand::Go(Self::extract_go_command(commands)?))
        }
    }
}

struct Set;

impl Set {
    fn extract_board_fen(commands: &[&str]) -> Result<UserCommand> {
        let fen = commands[3..].join(" ");
        if fen == "startpos" {
            return Ok(UserCommand::SetFen(STARTING_POSITION_FEN.to_string()));
        }
        if !Board::is_good_fen(&fen) {
            return Err(BadFen { fen });
        };
        Ok(UserCommand::SetFen(fen))
    }

    #[cfg(feature = "colored_output")]
    fn extract_color(commands: &[&str]) -> Result<UserCommand> {
        if commands.get(3).is_some() {
            return Err(UnknownCommand);
        }
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let b = third_command.parse()?;
        Ok(UserCommand::SetColor(b))
    }

    pub fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        match second_command.as_str() {
            "board" => {
                let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
                match third_command.as_str() {
                    "startpos" | "fen" => Self::extract_board_fen(commands),
                    _ => Err(UnknownCommand),
                }
            }
            #[cfg(feature = "colored_output")]
            "color" => Self::extract_color(commands),
            #[cfg(not(feature = "colored_output"))]
            "color" => Err(FeatureNotEnabled {
                s: "colored_output".to_string(),
            }),
            _ => Err(UnknownCommand),
        }
    }
}

struct SetOption;

impl SetOption {
    fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        if commands.first().ok_or(UnknownCommand)?.to_lowercase() != "setoption" {
            return Err(UnknownCommand);
        }
        if commands.get(1).ok_or(UnknownCommand)?.to_lowercase() != "name" {
            return Err(UnknownCommand);
        }
        let name = commands
            .iter()
            .skip(2)
            .take_while(|&&c| c != "value")
            .join(" ")
            .to_lowercase();
        let value = commands
            .iter()
            .skip_while(|&&s| s != "value")
            .skip(1)
            .join(" ");
        Ok(UserCommand::SetUCIOption { name, value })
    }
}

struct Push;

impl Push {
    fn moves(commands: &[&str]) -> Result<UserCommand> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if !["san", "lan", "uci", "move", "moves"].contains(&second_command.as_str()) {
            return Err(UnknownCommand);
        }
        Ok(UserCommand::MakeMoves(
            second_command,
            commands
                .iter()
                .skip(2)
                .map(|&move_text| move_text.to_string())
                .collect_vec(),
        ))
    }

    pub fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        Self::moves(commands)
    }
}

struct Pop;

impl Pop {
    pub fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        let second_command = commands.get(1).unwrap_or(&"1");
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        let num_pop = second_command.parse()?;
        Ok(UserCommand::UndoMoves(num_pop))
    }
}

struct SelfPlay;

impl SelfPlay {
    fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        let mut commands = commands.to_vec();
        commands[0] = "go";
        let go_command = if commands.get(1).is_some() {
            Go::extract_go_command(&commands)?
        } else {
            DEFAULT_SELFPLAY_COMMAND
        };
        Ok(UserCommand::SelfPlay(go_command))
    }
}

struct DebugMode;

impl DebugMode {
    fn get_debug_mode(second_command: &str) -> Result<bool> {
        match second_command {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err(UnknownDebugCommand {
                command: second_command.to_string(),
            }),
        }
    }

    fn parse_sub_commands(commands: &[&str]) -> Result<UserCommand> {
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        let second_command = &commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        let debug_mode = Self::get_debug_mode(&second_command)?;
        Ok(UserCommand::SetDebugMode(debug_mode))
    }
}

struct Position;

impl Position {
    fn parse_sub_commands(commands: &[&str]) -> Result<Vec<UserCommand>> {
        if commands.first() != Some(&"position") {
            return Err(UnknownCommand);
        }
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        let mut user_commands = Vec::with_capacity(2);
        user_commands.push(match second_command.as_str() {
            "startpos" => UserCommand::SetFen(STARTING_POSITION_FEN.to_string()),
            "fen" => {
                let fen = commands
                    .iter()
                    .skip(2)
                    .take_while(|&&s| s != "moves")
                    .join(" ");
                UserCommand::SetFen(fen)
            }
            _ => return Err(UnknownCommand),
        });
        let move_texts = commands
            .iter()
            .skip_while(|&&s| s != "moves")
            .skip(1)
            .map(|&move_text| move_text.to_string())
            .collect_vec();
        if !move_texts.is_empty() {
            user_commands.push(UserCommand::MakeMoves("moves".to_string(), move_texts));
        }
        Ok(user_commands)
    }
}

pub struct Parser_;

impl Parser_ {
    pub const EXIT_CODES: [&'static str; 7] = [
        "q", "quit", "quit()", "quit(0)", "exit", "exit()", "exit(0)",
    ];

    pub fn sanitize_string(user_input: &str) -> String {
        let user_input = user_input.trim();
        let mut user_input = user_input.to_string();
        for _char in [",", ":"] {
            user_input = user_input.replace(_char, " ")
        }
        user_input = remove_double_spaces_and_trim(&user_input);
        user_input
    }

    fn parse_single_command(single_input: &str) -> Result<Vec<UserCommand>> {
        if Self::EXIT_CODES.contains(&single_input) {
            return Ok(vec![UserCommand::TerminateEngine]);
        }
        let commands = single_input.split_whitespace().collect_vec();
        let first_command = commands.get(0).ok_or(UnknownCommand)?.to_lowercase();
        if ["uci", "ucinewgame"].contains(&first_command.as_str()) {
            if single_input.split_whitespace().nth(1).is_some() {
                return Err(UnknownCommand);
            }
            return Ok(vec![
                UserCommand::SetUCIMode(true),
                match first_command.as_str() {
                    "uci" => UserCommand::UCI,
                    "ucinewgame" => UserCommand::UCINewGame,
                    _ => unreachable!(),
                },
            ]);
        }
        if first_command == "console" {
            if single_input.split_whitespace().nth(1).is_some() {
                return Err(UnknownCommand);
            }
            if GLOBAL_UCI_STATE.is_in_console_mode() {
                return Err(ConsoleModeUnchanged);
            }
            return Ok(vec![UserCommand::SetUCIMode(false)]);
        }
        match single_input.to_lowercase().as_str() {
            "isready" => Ok(vec![UserCommand::IsReady]),
            "d" => Ok(vec![UserCommand::DisplayBoard]),
            "eval" => Ok(vec![UserCommand::EvaluateBoard]),
            "reset board" => Ok(vec![UserCommand::SetFen(STARTING_POSITION_FEN.to_owned())]),
            "stop" => Ok(vec![UserCommand::Stop]),
            "help" => Ok(vec![UserCommand::Help]),
            _ => match first_command.as_str() {
                "go" => Ok(vec![Go::parse_sub_commands(&commands)?]),
                "set" => Ok(vec![Set::parse_sub_commands(&commands)?]),
                "setoption" => Ok(vec![SetOption::parse_sub_commands(&commands)?]),
                "push" => Ok(vec![Push::parse_sub_commands(&commands)?]),
                "pop" => Ok(vec![Pop::parse_sub_commands(&commands)?]),
                "position" => Position::parse_sub_commands(&commands),
                "selfplay" => Ok(vec![SelfPlay::parse_sub_commands(&commands)?]),
                "debug" => Ok(vec![DebugMode::parse_sub_commands(&commands)?]),
                _ => Err(UnknownCommand),
            },
        }
    }

    fn parse_command(user_input: &str) -> Result<Vec<UserCommand>> {
        if user_input.is_empty() {
            if GLOBAL_UCI_STATE.is_in_console_mode() {
                println!("\n");
            }
            GLOBAL_UCI_STATE.set_engine_termination(true);
            return Ok(vec![UserCommand::TerminateEngine]);
        }
        if user_input.trim().is_empty() {
            return Err(NoInput);
        }
        Self::sanitize_string(user_input)
            .split("&&")
            .map(|s| s.trim())
            .map(|single_input| Self::parse_single_command(single_input.trim()))
            .fold(Ok(Vec::new()), |acc, res| match (acc, res) {
                (Ok(mut vec), Ok(contents)) => {
                    vec.extend(contents);
                    Ok(vec)
                }
                (_, Err(e)) => Err(e),
                (Err(e), _) => Err(e),
            })
    }
}
