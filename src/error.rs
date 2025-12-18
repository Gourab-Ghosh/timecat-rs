use super::*;
use TimecatError::*;

#[cfg(feature = "pyo3")]
#[derive(Debug)]
pub enum Pyo3Error {
    #[cfg(feature = "pyo3")]
    Pyo3TypeConversionError {
        from: Cow<'static, str>,
        to: Cow<'static, str>,
    },
}

#[cfg(feature = "pyo3")]
impl From<Pyo3Error> for PyErr {
    fn from(err: Pyo3Error) -> Self {
        match err {
            Pyo3Error::Pyo3TypeConversionError { from, to } => {
                pyo3::exceptions::PyTypeError::new_err(format!(
                    "Failed to convert {from} into {to}"
                ))
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum TimecatError {
    UnknownCommand,
    NoInput,
    NotImplemented,
    EngineNotRunning,
    BadFen {
        fen: Cow<'static, str>,
    },
    InvalidDepth {
        depth: Depth,
    },
    IllegalMove {
        valid_or_null_move: ValidOrNullMove,
        board_fen: Cow<'static, str>,
    },
    ColoredOutputUnchanged {
        b: bool,
    },
    UCIModeUnchanged,
    ConsoleModeUnchanged,
    EmptyStack,
    BestMoveNotFound {
        fen: Cow<'static, str>,
    },
    NullMoveInCheck {
        fen: Cow<'static, str>,
    },
    WTimeNotMentioned,
    BTimeNotMentioned,
    GameAlreadyOver,
    UnknownDebugCommand {
        command: Cow<'static, str>,
    },
    InvalidSpinValue {
        name: Cow<'static, str>,
        value: Spin,
        min: Spin,
        max: Spin,
    },
    SameSourceAndDestination {
        move_: Move,
    },
    InvalidPromotion {
        move_: Move,
    },
    InvalidSanOrLanMove {
        valid_or_null_move: ValidOrNullMove,
        fen: Cow<'static, str>,
    },
    InvalidSanMoveString {
        s: Cow<'static, str>,
    },
    InvalidLanMoveString {
        s: Cow<'static, str>,
    },
    InvalidMoveString {
        s: Cow<'static, str>,
    },
    InvalidRankString {
        s: Cow<'static, str>,
    },
    InvalidFileString {
        s: Cow<'static, str>,
    },
    InvalidColorString {
        s: Cow<'static, str>,
    },
    InvalidCastleRightsString {
        s: Cow<'static, str>,
    },
    InvalidSquareString {
        s: Cow<'static, str>,
    },
    InvalidPieceTypeString {
        s: Cow<'static, str>,
    },
    InvalidPieceString {
        s: Cow<'static, str>,
    },
    InvalidUciMoveString {
        s: Cow<'static, str>,
    },
    InvalidBoardPosition {
        position: Box<ChessPosition>,
    },
    InvalidGoCommand {
        s: Cow<'static, str>,
    },
    IllegalSearchMoves {
        illegal_moves: Vec<Move>,
    },
    FeatureNotEnabled {
        s: Cow<'static, str>,
    },
    BadNNUEFile,
    BadPolyglotFile,
    PolyglotTableParseError,
    DecompressionFailed {
        value: Cow<'static, str>,
        type_name: Cow<'static, str>,
    },
    CustomError {
        err_msg: Cow<'static, str>,
    },
}

impl TimecatError {
    pub fn get_custom_error<E: Error>(error: E) -> Self {
        Self::CustomError {
            err_msg: format!("{error}! Please try again!").into(),
        }
    }
}

impl fmt::Display for TimecatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnknownCommand => write!(
                f,
                "{}",
                UnknownCommand.stringify_with_optional_raw_input(None)
            ),
            NoInput => write!(f, "No input! Please try again!"),
            NotImplemented => write!(f, "Sorry, this command is not implemented yet :("),
            EngineNotRunning => write!(f, "Engine is not running! Please try again!"),
            BadFen { fen } => write!(f, "Bad FEN string: {fen}! Please try Again!"),
            InvalidDepth { depth } => write!(f, "Invalid depth {depth}! Please try again!"),
            IllegalMove {
                valid_or_null_move,
                board_fen,
            } => write!(
                f,
                "Illegal move {valid_or_null_move} in position {board_fen}! Please try again!"
            ),
            ColoredOutputUnchanged { b } => {
                write!(f, "Colored output already set to {b}! Please try again!")
            }
            UCIModeUnchanged => write!(f, "Already in UCI Mode! Please try again!"),
            ConsoleModeUnchanged => write!(f, "Already in Console Mode! Please try again!"),
            EmptyStack => write!(
                f,
                "Move Stack is empty, pop not possible! Please try again!"
            ),
            BestMoveNotFound { fen } => write!(
                f,
                "Best move not found in position {fen}! Please try again!"
            ),
            NullMoveInCheck { fen } => write!(
                f,
                "Cannot apply null move in position {fen}, as king is in check! Please try again!"
            ),
            WTimeNotMentioned => write!(f, "You didn't mention wtime! Please try again!"),
            BTimeNotMentioned => write!(f, "You didn't mention btime! Please try again!"),
            GameAlreadyOver => write!(
                f,
                "Game is already over! Please start a game from another position!"
            ),
            UnknownDebugCommand { command } => write!(
                f,
                "Debug command {command} is unknown! The possible commands are on or off! Please try again!"
            ),
            InvalidSpinValue {
                name,
                value,
                min,
                max,
            } => write!(
                f,
                "Cannot set value of {name} to {value}, the value must be from {min} to {max}! Please try again!"
            ),
            SameSourceAndDestination { move_ } => {
                write!(
                    f,
                    "The source and destination squares of the move {} cannot be the same!",
                    move_
                )
            }
            InvalidPromotion { move_ } => {
                write!(f, "Invalid promotion for the move {}!", move_)
            }
            InvalidSanOrLanMove {
                valid_or_null_move,
                fen,
            } => write!(
                f,
                "san() and lan() expect move to be legal or null, but got {} in {}",
                valid_or_null_move, fen
            ),
            InvalidSanMoveString { s } => {
                write!(f, "Got invalid SAN move string {s}! Please try again!")
            }
            InvalidLanMoveString { s } => {
                write!(f, "Got invalid LAN move string {s}! Please try again!")
            }
            InvalidMoveString { s } => write!(f, "Got invalid move string {s}! Please try again!"),
            InvalidRankString { s } => write!(f, "Got invalid rank string {s}! Please try again!"),
            InvalidFileString { s } => write!(f, "Got invalid file string {s}! Please try again!"),
            InvalidColorString { s } => {
                write!(f, "Got invalid color string {s}! Please try again!")
            }
            InvalidCastleRightsString { s } => {
                write!(f, "Got invalid castle rights string {s}! Please try again!")
            }
            InvalidSquareString { s } => {
                write!(f, "Got invalid square string {s}! Please try again!")
            }
            InvalidPieceTypeString { s } => {
                write!(f, "Got invalid piece type string {s}! Please try again!")
            }
            InvalidPieceString { s } => {
                write!(f, "Got invalid piece string {s}! Please try again!")
            }
            InvalidUciMoveString { s } => {
                write!(f, "Invalid uci move string {s}! Please try again!")
            }
            InvalidBoardPosition { position } => {
                write!(f, "Invalid position generated:\n\n{position:#?}")
            }
            InvalidGoCommand { s } => write!(f, "Got invalid go command: {s:?}! Please try again!"),
            IllegalSearchMoves { illegal_moves } => write!(
                f,
                "Got illegal search moves: {}! Please try again!",
                illegal_moves.iter().map(ToString::to_string).join(", ")
            ),
            FeatureNotEnabled { s } => write!(
                f,
                "The feature {s:?} is not enabled. Please recompile the chess engine with this feature enabled!"
            ),
            BadNNUEFile => write!(
                f,
                "The NNUE file cannot be parsed properly! Try again with a different NNUE file!"
            ),
            BadPolyglotFile => write!(
                f,
                "The Polyglot file cannot be parsed properly! Try again with a different Polyglot file!"
            ),
            PolyglotTableParseError => write!(
                f,
                "The Polyglot Table cannot be parsed properly! Try again with a different Polyglot file!"
            ),
            DecompressionFailed { value, type_name } => write!(
                f,
                "Failed to decompress value {} into {:?}",
                value, type_name
            ),
            CustomError { err_msg } => write!(f, "{err_msg}"),
        }
    }
}

impl Error for TimecatError {}

impl TimecatError {
    pub fn stringify_with_optional_raw_input(&self, optional_raw_input: Option<&str>) -> String {
        match self {
            Self::UnknownCommand => {
                let command_type = if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
                    "Console"
                } else {
                    "UCI"
                };
                optional_raw_input.map_or_else(
                    || format!("Unknown {command_type} Command!\nPlease try again!"),
                    |raw_input| {
                        format!(
                            "Unknown {command_type} Command: {:?}\nType help for more information!",
                            raw_input.trim_end_matches('\n')
                        )
                    },
                )
            }
            other_err => other_err.to_string(),
        }
    }
}

impl From<TimecatError> for String {
    fn from(error: TimecatError) -> Self {
        error.stringify().into()
    }
}

impl From<&Self> for TimecatError {
    fn from(error: &Self) -> Self {
        error.clone()
    }
}

impl From<ParseBoolError> for TimecatError {
    fn from(error: ParseBoolError) -> Self {
        CustomError {
            err_msg: format!("Failed to parse bool, {error}! Please try again!").into(),
        }
    }
}

impl From<ParseIntError> for TimecatError {
    fn from(error: ParseIntError) -> Self {
        CustomError {
            err_msg: format!("Failed to parse integer, {error}! Please try again!").into(),
        }
    }
}

macro_rules! impl_error_convert {
    ($class:ty) => {
        impl From<$class> for TimecatError {
            fn from(error: $class) -> Self {
                Self::get_custom_error(error)
            }
        }
    };
}

impl_error_convert!(std::io::Error);
impl_error_convert!(std::array::TryFromSliceError);

impl From<String> for TimecatError {
    fn from(err_msg: String) -> Self {
        Cow::from(err_msg).into()
    }
}

impl From<&'static str> for TimecatError {
    fn from(err_msg: &'static str) -> Self {
        Cow::from(err_msg).into()
    }
}

impl From<Cow<'static, str>> for TimecatError {
    fn from(err_msg: Cow<'static, str>) -> Self {
        CustomError { err_msg }
    }
}

#[cfg(feature = "pyo3")]
impl From<TimecatError> for PyErr {
    fn from(err: TimecatError) -> Self {
        pyo3::exceptions::PyRuntimeError::new_err(format!("TimecatError occurred: {:?}", err))
    }
}
