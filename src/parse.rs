use super::*;
use TimecatError::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum UserCommand {
    TerminateEngine,
    EngineVersion,
    #[cfg(feature = "debug")]
    RunTest,
    ChangeToUCIMode {
        verbose: bool,
    },
    ChangeToConsoleMode {
        verbose: bool,
    },
    SetDebugMode(bool),
    PrintText(String),
    DisplayBoard,
    #[cfg(feature = "inbuilt_nnue")]
    DisplayBoardEvaluation,
    PrintUCIInfo,
    UCIMode,
    UCINewGame,
    IsReady,
    Stop,
    Help,
    Perft(Depth),
    Go(GoCommand),
    PushMoves(String),
    PopMoves(u16),
    SetFen(String),
    #[cfg(feature = "colored")]
    SetColor(bool),
    SetUCIOption {
        user_input: String,
    },
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

impl UserCommand {
    fn print_engine_uci_info(uci_state_manager: &UCIStateManager) {
        println_wasm!(
            "{}",
            format!("id name {}", get_engine_version()).colorize(INFO_MESSAGE_STYLE)
        );
        println_wasm!(
            "{}",
            format!("id author {}", ENGINE_AUTHOR).colorize(INFO_MESSAGE_STYLE)
        );
        for option in uci_state_manager.get_all_options() {
            println_wasm!("{option}");
        }
    }

    pub fn generate_help_message() -> String {
        "Sadly, the help message is till now not implemented. But type uci to go into the uci mode and visit the link \"https://backscattering.de/chess/uci/\" to know the necessary commands required to use an uci chess engine.".colorize(ERROR_MESSAGE_STYLE)
    }

    pub fn run_command(
        &self,
        engine: &mut Engine,
        uci_state_manager: &UCIStateManager,
    ) -> Result<()> {
        match self {
            Self::TerminateEngine => engine.set_termination(true),
            Self::EngineVersion => print_engine_version(),
            #[cfg(feature = "debug")]
            Self::RunTest => test.run_and_print_time(engine)?,
            &Self::ChangeToUCIMode { verbose } => GLOBAL_TIMECAT_STATE.set_uci_mode(true, verbose),
            &Self::ChangeToConsoleMode { verbose } => {
                GLOBAL_TIMECAT_STATE.set_console_mode(true, verbose)
            }
            &Self::SetDebugMode(b) => GLOBAL_TIMECAT_STATE.set_debug_mode(b),
            Self::PrintText(s) => println_wasm!("{s}"),
            Self::DisplayBoard => println_wasm!("{}", engine.get_board()),
            #[cfg(feature = "inbuilt_nnue")]
            Self::DisplayBoardEvaluation => force_println_info(
                "Current Score",
                engine.get_board_mut().evaluate().stringify(),
            ),
            Self::PrintUCIInfo => Self::print_engine_uci_info(uci_state_manager),
            Self::UCIMode => {
                Self::PrintUCIInfo.run_command(engine, uci_state_manager)?;
                println_wasm!("{}", "uciok".colorize(SUCCESS_MESSAGE_STYLE));
            }
            Self::UCINewGame => {
                Self::SetUCIOption {
                    user_input: "setoption name Clear Hash".to_string(),
                }
                .run_command(engine, uci_state_manager)?;
                Self::SetFen(STARTING_POSITION_FEN.to_string())
                    .run_command(engine, uci_state_manager)?;
            }
            Self::IsReady => println_wasm!("{}", "readyok".colorize(SUCCESS_MESSAGE_STYLE)),
            Self::Stop => {
                if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                    return Err(EngineNotRunning);
                }
            }
            Self::Help => println_wasm!("{}", Self::generate_help_message()),
            &Self::Perft(depth) => GoAndPerft::run_perft_command(engine, depth)?,
            &Self::Go(go_command) => GoAndPerft::run_go_command(engine, go_command)?,
            Self::PushMoves(user_input) => {
                let binding = Parser::sanitize_string(user_input);
                Push::push_moves(engine, &binding.split_whitespace().collect_vec())?
            }
            &Self::PopMoves(num_moves) => Pop::pop_moves(engine, num_moves)?,
            Self::SetFen(fen) => Set::set_board_fen(engine, fen)?,
            #[cfg(feature = "colored")]
            &Self::SetColor(b) => Set::set_color(b)?,
            Self::SetUCIOption { user_input } => {
                uci_state_manager.run_command(engine, user_input)?
            }
            &Self::SelfPlay(go_command) => self_play(engine, go_command, true, None)?,
        }

        Ok(())
    }
}

impl From<UserCommand> for Result<Vec<UserCommand>> {
    fn from(value: UserCommand) -> Self {
        Ok(vec![value])
    }
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

struct GoAndPerft;

impl GoAndPerft {
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

    pub fn parse_sub_commands(commands: &[&str]) -> Result<Vec<UserCommand>> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "perft" {
            UserCommand::Perft(commands.get(2).ok_or(UnknownCommand)?.parse()?).into()
        } else {
            UserCommand::Go(Self::extract_go_command(commands)?).into()
        }
    }

    fn run_perft_command(engine: &mut Engine, depth: Depth) -> Result<()> {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_wasm!("{}\n", engine.get_board());
        }
        let clock = Instant::now();
        let position_count = engine.get_board_mut().perft(depth);
        let elapsed_time = clock.elapsed();
        let nps: String = format!(
            "{} nodes/sec",
            (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
        );
        println_wasm!();
        force_println_info("Position Count", position_count);
        force_println_info("Time", elapsed_time.stringify());
        force_println_info("Speed", nps);
        Ok(())
    }

    fn run_go_command(engine: &mut Engine, go_command: GoCommand) -> Result<()> {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_wasm!("{}\n", engine.get_board());
        }
        let clock = Instant::now();
        let response = engine.go(go_command, true);
        let Some(best_move) = response.get_best_move() else {
            return Err(BestMoveNotFound {
                fen: engine.get_board().get_fen(),
            });
        };
        let elapsed_time = clock.elapsed();
        let position_count = engine.get_num_nodes_searched();
        let nps = format!(
            "{} Nodes/sec",
            (position_count as u128 * 10u128.pow(9)) / elapsed_time.as_nanos()
        );
        let pv_string = get_pv_string(engine.get_board().get_sub_board(), response.get_pv());
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_wasm!();
        }
        println_info("Score", response.get_score().stringify());
        println_info("PV Line", pv_string);
        println_info("Position Count", position_count);
        println_info("Time", elapsed_time.stringify());
        println_info("Speed", nps);
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_info(
                "Best Move",
                best_move
                    .stringify_move(engine.get_board().get_sub_board())
                    .unwrap(),
            );
        } else {
            let mut move_text = format_info(
                "bestmove",
                best_move
                    .stringify_move(engine.get_board().get_sub_board())
                    .unwrap(),
                false,
            );
            if let Some(ponder_move) = response.get_ponder_move() {
                move_text += " ";
                move_text += &format_info(
                    "ponder",
                    ponder_move
                        .stringify_move(
                            &engine.get_board().get_sub_board().make_move_new(best_move),
                        )
                        .unwrap(),
                    false,
                );
            }
            println_wasm!("{}", move_text);
        }
        Ok(())
    }
}

struct Set;

impl Set {
    fn extract_board_fen(commands: &[&str]) -> Result<Vec<UserCommand>> {
        let fen = commands[3..].join(" ");
        if fen == "startpos" {
            return UserCommand::SetFen(STARTING_POSITION_FEN.to_string()).into();
        }
        if !Board::is_good_fen(&fen) {
            return Err(BadFen { fen });
        };
        UserCommand::SetFen(fen).into()
    }

    #[cfg(feature = "colored")]
    fn extract_color(commands: &[&str]) -> Result<Vec<UserCommand>> {
        if commands.get(3).is_some() {
            return Err(UnknownCommand);
        }
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let b = third_command.parse()?;
        UserCommand::SetColor(b).into()
    }

    pub fn parse_sub_commands(commands: &[&str]) -> Result<Vec<UserCommand>> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        match second_command.as_str() {
            "board" => {
                let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
                match third_command.as_str() {
                    "startpos" | "fen" => Self::extract_board_fen(commands),
                    _ => Err(UnknownCommand),
                }
            }
            #[cfg(feature = "colored")]
            "color" => Self::extract_color(commands),
            #[cfg(not(feature = "colored"))]
            "color" => Err(FeatureNotEnabled {
                s: "colored".to_string(),
            }),
            _ => Err(UnknownCommand),
        }
    }

    fn set_board_fen(engine: &mut Engine, fen: &str) -> Result<()> {
        if !Board::is_good_fen(fen) {
            return Err(BadFen {
                fen: fen.to_string(),
            });
        };
        engine.set_fen(fen)?;
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            println_wasm!("{}", engine.get_board());
        }
        Ok(())
    }

    #[cfg(feature = "colored")]
    fn set_color(b: bool) -> Result<()> {
        if GLOBAL_TIMECAT_STATE.is_colored() == b {
            return Err(ColoredOutputUnchanged { b });
        }
        GLOBAL_TIMECAT_STATE.set_colored(b, true);
        Ok(())
    }
}

struct Push;

impl Push {
    fn push_moves(engine: &mut Engine, commands: &[&str]) -> Result<()> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        for move_text in commands.iter().skip(2) {
            let valid_or_null_move = match second_command.as_str() {
                "san" | "sans" => engine.get_board().parse_san(move_text),
                "lan" | "lans" => engine.get_board().parse_lan(move_text),
                "uci" | "ucis" => engine.get_board().parse_uci(move_text),
                "move" | "moves" => engine.get_board().parse_move(move_text),
                _ => Err(UnknownCommand),
            }?;
            engine.get_board_mut().push(valid_or_null_move)?;
            println_info("Pushed move", move_text);
        }
        Ok(())
    }
}

struct Pop;

impl Pop {
    fn pop_moves(engine: &mut Engine, num_moves: u16) -> Result<()> {
        for _ in 0..num_moves {
            if engine.get_board().has_empty_stack() {
                return Err(EmptyStack);
            }
            let last_move = engine.get_board_mut().pop();
            println_info(
                "Popped move",
                last_move
                    .stringify_move(engine.get_board().get_sub_board())
                    .unwrap(),
            );
        }
        Ok(())
    }

    pub fn parse_sub_commands(commands: &[&str]) -> Result<Vec<UserCommand>> {
        let second_command = commands.get(1).unwrap_or(&"1");
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        let num_pop = second_command.parse()?;
        UserCommand::PopMoves(num_pop).into()
    }
}

struct SelfPlay;

impl SelfPlay {
    fn parse_sub_commands(commands: &[&str]) -> Result<Vec<UserCommand>> {
        let mut commands = commands.to_vec();
        commands[0] = "go";
        let go_command = if commands.get(1).is_some() {
            GoAndPerft::extract_go_command(&commands)?
        } else {
            DEFAULT_SELFPLAY_COMMAND
        };
        UserCommand::SelfPlay(go_command).into()
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

    fn parse_sub_commands(commands: &[&str]) -> Result<Vec<UserCommand>> {
        if commands.get(2).is_some() {
            return Err(UnknownCommand);
        }
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        let debug_mode = Self::get_debug_mode(&second_command)?;
        UserCommand::SetDebugMode(debug_mode).into()
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
        let move_texts_joined = commands
            .iter()
            .skip_while(|&&s| s != "moves")
            .skip(1)
            .join(" ");
        if !move_texts_joined.is_empty() {
            user_commands.push(UserCommand::PushMoves(format!(
                "push moves {move_texts_joined}"
            )));
        }
        Ok(user_commands)
    }
}

pub struct Parser;

impl Parser {
    pub fn sanitize_string(raw_input: &str) -> String {
        let user_input = raw_input.trim();
        let mut user_input = user_input.to_string();
        for _char in [",", ":"] {
            user_input = user_input.replace(_char, " ")
        }
        user_input = remove_double_spaces_and_trim(&user_input);
        user_input
    }

    fn parse_single_command(single_input: &str) -> Result<Vec<UserCommand>> {
        match single_input.to_lowercase().as_str() {
            "q" | "quit" | "quit()" | "quit(0)" | "exit" | "exit()" | "exit(0)" => {
                UserCommand::TerminateEngine.into()
            }
            "uci" | "ucinewgame" => Ok(vec![
                UserCommand::ChangeToUCIMode { verbose: false },
                match single_input {
                    "uci" => UserCommand::UCIMode,
                    "ucinewgame" => UserCommand::UCINewGame,
                    _ => unreachable!(),
                },
            ]),
            "ucimode" => {
                if GLOBAL_TIMECAT_STATE.is_in_uci_mode() {
                    Err(UCIModeUnchanged)
                } else {
                    UserCommand::ChangeToUCIMode { verbose: true }.into()
                }
            }
            "console" | "consolemode" => {
                if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                    Err(ConsoleModeUnchanged)
                } else {
                    UserCommand::ChangeToConsoleMode { verbose: true }.into()
                }
            }
            "isready" => UserCommand::IsReady.into(),
            "d" => UserCommand::DisplayBoard.into(),
            "eval" => UserCommand::DisplayBoardEvaluation.into(),
            "reset board" => UserCommand::SetFen(STARTING_POSITION_FEN.to_owned()).into(),
            "stop" => UserCommand::Stop.into(),
            "help" => UserCommand::Help.into(),
            _ => {
                let commands = single_input.split_whitespace().collect_vec();
                let first_command = commands.first().ok_or(UnknownCommand)?.to_lowercase();
                match first_command.as_str() {
                    "go" => GoAndPerft::parse_sub_commands(&commands),
                    "set" => Set::parse_sub_commands(&commands),
                    "setoption" => UserCommand::SetUCIOption {
                        user_input: single_input.to_string(),
                    }
                    .into(),
                    "push" => UserCommand::PushMoves(single_input.to_string()).into(),
                    "pop" => Pop::parse_sub_commands(&commands),
                    "position" => Position::parse_sub_commands(&commands),
                    "selfplay" => SelfPlay::parse_sub_commands(&commands),
                    "debug" => DebugMode::parse_sub_commands(&commands),
                    _ => Err(UnknownCommand),
                }
            }
        }
    }

    pub fn parse_command(raw_input: &str) -> Result<Vec<UserCommand>> {
        if raw_input.is_empty() {
            return Ok(vec![
                UserCommand::PrintText("".to_string()),
                UserCommand::TerminateEngine,
            ]);
        }
        if raw_input.trim().is_empty() {
            return Err(NoInput);
        }
        Self::sanitize_string(raw_input)
            .split("&&")
            .map(|single_input| Self::parse_single_command(single_input.trim()))
            .try_fold(Vec::new(), |mut vec, res| match res {
                Ok(contents) => {
                    vec.extend(contents);
                    Ok(vec)
                }
                Err(e) => Err(e),
            })
    }
}
