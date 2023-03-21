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

    #[fail(display = "Colored output already set to {}! Try again!", _bool)]
    ColoredOutputUnchanged { _bool: String },

    #[fail(display = "Move Stack is enpty, pop not possible! Try again!")]
    EmptyStack,

    #[fail(display = "{}", err_msg)]
    CustomError { err_msg: String },
}

impl ParserError {
    pub fn generate_error(&self, raw_input_option: Option<&str>) -> String {
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

const EXIT_CODES: [&str; 7] = [
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
        println_info("Time", format!("{} s", elapsed_time.as_secs_f32()));
        println_info("Speed", nps);
        position_count
    }

    fn depth(engine: &mut Engine, depth: Depth) -> usize {
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
        println_info("Best Move", engine.board.san(best_move).unwrap());
        position_count
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        let depth_str = commands.get(2).ok_or(UnknownCommand)?.to_string();
        let depth = depth_str.parse().unwrap_or(0);
        if commands.get(3).is_some() {
            return Err(UnknownCommand);
        }
        if depth == 0 {
            return Err(InvalidDepth { depth: depth_str });
        }
        if second_command == "perft" {
            Self::perft(engine, depth);
            return Ok(());
        } else if second_command == "depth" {
            Self::depth(engine, depth);
            return Ok(());
        }
        Err(UnknownCommand)
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
        engine.board.set_fen(&fen);
        println!("{}", engine.board);
        Ok(())
    }

    fn color(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
        let _bool = third_command.parse()?;
        if is_colored_output() == _bool {
            return Err(ColoredOutputUnchanged {
                _bool: third_command,
            });
        }
        if _bool {
            println!();
            set_colored_output(_bool);
        } else {
            set_colored_output(_bool);
            println!();
        }
        Ok(())
    }

    pub fn parse_sub_command(engine: &mut Engine, commands: &[&str]) -> Result<(), ParserError> {
        let second_command = commands.get(1).ok_or(UnknownCommand)?.to_lowercase();
        if second_command == "board" {
            let third_command = commands.get(2).ok_or(UnknownCommand)?.to_lowercase();
            if third_command == "fen" {
                return Self::board_fen(engine, commands);
            }
        } else if second_command == "color" {
            if commands.get(3).is_some() {
                return Err(UnknownCommand);
            }
            return Self::color(engine, commands);
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
            let _move: Move;
            if second_command == "san" {
                _move = engine.board.parse_san(move_text)?;
            } else if second_command == "uci" {
                _move = engine.board.parse_uci(move_text)?;
            } else if second_command == "move" {
                _move = engine.board.parse_move(move_text)?;
            } else {
                return Err(UnknownCommand);
            }
            if !engine.board.is_legal(_move) {
                return Err(IllegalMove {
                    move_text: move_text.to_string(),
                    board_fen: engine.board.get_fen(),
                });
            }
            engine.board.push(Some(_move));
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
            let last_move = engine.board.pop().unwrap_or_default();
            println!(
                "{} {}",
                colorize("Popped move:", SUCCESS_MESSAGE_STYLE),
                colorize(engine.board.san(last_move).unwrap(), INFO_STYLE),
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

    fn split_inputs(input: &str) -> Vec<String> {
        let inputs = input.split("&&");
        let mut input_vec = Vec::new();
        for input in inputs {
            input_vec.push(input.trim().to_string());
        }
        input_vec
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
        Err(UnknownCommand)
    }

    pub fn parse_command(engine: &mut Engine, raw_input: &str) -> Result<(), ParserError> {
        let modified_raw_str = Self::parse_raw_input(raw_input);
        let user_inputs = Self::split_inputs(&modified_raw_str);
        let mut first_loop = true;
        for user_input in user_inputs {
            if !first_loop {
                println!();
            }
            first_loop = false;
            let response = Parser::run_command(engine, &user_input);
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
            let error_message = parser_error.generate_error(Some(raw_input));
            println!("{}", colorize(error_message, ERROR_MESSAGE_STYLE));
        }
        ParserLoopState::Continue
    }

    pub fn main_loop() {
        let mut engine = Engine::default();
        loop {
            let raw_input: String;
            if is_colored_output() {
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
