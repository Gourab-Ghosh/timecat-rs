use super::*;
use EngineError::*;

#[derive(Clone, Fail, PartialEq, Eq, Debug)]
pub enum EngineError {
    #[fail(display = "No input! Please try again!")]
    NoInput,

    #[fail(display = "")]
    UnknownCommand,

    #[fail(display = "Sorry, this command is not implemented yet :(")]
    NotImplemented,

    #[fail(display = "Engine is not running! Please try again!")]
    EngineNotRunning,

    #[fail(display = "Bad FEN string: {}! Please try Again!", fen)]
    BadFen { fen: String },

    #[fail(display = "Invalid depth {}! Please try again!", depth)]
    InvalidDepth { depth: Depth },

    #[fail(
        display = "Illegal move {} in position {}! Please try again!",
        move_text, board_fen
    )]
    IllegalMove {
        move_text: String,
        board_fen: String,
    },

    #[fail(display = "Colored output already set to {}! Please try again!", b)]
    ColoredOutputUnchanged { b: bool },

    #[fail(display = "Already in Console Mode! Please try again!")]
    ConsoleModeUnchanged,

    #[fail(display = "Move Stack is empty, pop not possible! Please try again!")]
    EmptyStack,

    #[fail(display = "Best move not found in position {}! Please try again!", fen)]
    BestMoveNotFound { fen: String },

    #[fail(
        display = "Cannot apply null move in position {}, as king is in check! Please try again!",
        fen
    )]
    NullMoveInCheck { fen: String },

    #[fail(display = "You didn't mention wtime! Please try again!")]
    WTimeNotMentioned,

    #[fail(display = "You didn't mention btime! Please try again!")]
    BTimeNotMentioned,

    #[fail(display = "Game is already over! Please start a game from another position!")]
    GameAlreadyOver,

    #[fail(
        display = "Debug command {} is unknown! The possible commands are on or off! Please try again!",
        command
    )]
    UnknownDebugCommand { command: String },

    #[fail(
        display = "Cannot set value of {} to {}, the value must be from {} to {}! Please try again!",
        name, value, min, max
    )]
    InvalidSpinValue {
        name: String,
        value: Spin,
        min: Spin,
        max: Spin,
    },

    #[fail(display = "Got invalid SAN move string {}! Please try again!", s)]
    InvalidSanMoveString { s: String },

    #[fail(display = "Got invalid rank string {}! Please try again!", s)]
    InvalidRankString { s: String },

    #[fail(display = "Got invalid file string {}! Please try again!", s)]
    InvalidFileString { s: String },

    #[fail(display = "Got invalid square string {}! Please try again!", s)]
    InvalidSquareString { s: String },

    #[fail(display = "You didn't mention wtime! Please try again!")]
    InvalidUciMoveString { s: String },

    #[fail(display = "Invalid sub board generated:\n\n{:#?}", board)]
    InvalidSubBoard { board: SubBoard },

    #[fail(display = "{}", err_msg)]
    CustomError { err_msg: String },
}

impl EngineError {
    pub fn stringify_with_optional_raw_input(&self, optional_raw_input: Option<&str>) -> String {
        match self {
            Self::UnknownCommand => {
                let command_type = if is_in_console_mode() {
                    "Console"
                } else {
                    "UCI"
                };
                match optional_raw_input {
                    Some(raw_input) => format!(
                        "Unknown {command_type} Command: {:?}\nType help for more information!",
                        raw_input.trim_end_matches('\n')
                    ),
                    None => format!("Unknown {command_type} Command!\nPlease try again!"),
                }
            }
            other_err => other_err.to_string(),
        }
    }
}

impl Stringify for EngineError {
    fn stringify(&self) -> String {
        self.stringify_with_optional_raw_input(None)
    }
}

impl From<EngineError> for String {
    fn from(error: EngineError) -> Self {
        error.stringify()
    }
}

impl From<&Self> for EngineError {
    fn from(error: &Self) -> Self {
        error.clone()
    }
}

impl From<ParseBoolError> for EngineError {
    fn from(error: ParseBoolError) -> Self {
        CustomError {
            err_msg: format!("Failed to parse bool, {error}! Please try again!"),
        }
    }
}

impl From<ParseIntError> for EngineError {
    fn from(error: ParseIntError) -> Self {
        CustomError {
            err_msg: format!("Failed to parse integer, {error}! Please try again!"),
        }
    }
}

macro_rules! impl_error_convert {
    ($class:ty) => {
        impl From<$class> for EngineError {
            fn from(error: $class) -> Self {
                CustomError {
                    err_msg: format!("{error}! Please try again!"),
                }
            }
        }
    };
}

impl_error_convert!(std::io::Error);
impl_error_convert!(std::array::TryFromSliceError);

impl From<String> for EngineError {
    fn from(err_msg: String) -> Self {
        CustomError { err_msg }
    }
}

impl From<&str> for EngineError {
    fn from(err_msg: &str) -> Self {
        err_msg.to_string().into()
    }
}
