use super::*;

pub fn remove_double_spaces_and_trim(s: &str) -> String {
    s.trim()
        .chars()
        .dedup_by(|&c1, &c2| c1 == ' ' && c2 == ' ')
        .join("")
}

pub fn simplify_fen(fen: &str) -> String {
    remove_double_spaces_and_trim(fen)
}

pub fn flip_board_fen(fen: &str) -> Result<String, EngineError> {
    // TODO: ep square not flipped.
    let fen = remove_double_spaces_and_trim(fen);
    let (position_fen, rest_fen) = fen.split_once(' ').ok_or(EngineError::BadFen {
        fen: fen.to_string(),
    })?;
    Ok(format!(
        "{} {rest_fen}",
        position_fen
            .chars()
            .map(|c| match c {
                c if c.is_uppercase() => c.to_ascii_lowercase(),
                c if c.is_lowercase() => c.to_ascii_uppercase(),
                _ => c,
            })
            .collect::<String>(),
    ))
}

pub trait CustomColorize {
    fn colorize(&self, style_functions: &[ColoredStringFunction]) -> String;
}

impl<T: ToString> CustomColorize for T {
    fn colorize(&self, style_functions: &[ColoredStringFunction]) -> String {
        let self_string = self.to_string();
        if style_functions.is_empty() || !is_colored_output() {
            return self_string;
        }
        let mut colorized_string = self_string.as_str().into();
        for &func in style_functions {
            colorized_string = func(colorized_string);
        }
        colorized_string.to_string()
    }
}

pub trait Stringify {
    fn stringify(&self) -> String;
}

pub trait StringifyScore {
    fn stringify_score(self) -> String;
    fn stringify_score_uci(self) -> String;
}

impl StringifyScore for Score {
    fn stringify_score(self) -> String {
        if self == INFINITY {
            return "INFINITY".to_string();
        }
        if self == -INFINITY {
            return "-INFINITY".to_string();
        }
        if is_checkmate(self) {
            let mut mate_string = String::from(if self.is_positive() { "M" } else { "-M" });
            let mate_distance = (CHECKMATE_SCORE - self.abs() + 1) / 2;
            mate_string += &mate_distance.to_string();
            return mate_string;
        }
        let to_return = self as f64 / PAWN_VALUE as f64;
        if to_return % 1.0 == 0.0 {
            format!("{}", to_return as i32)
        } else {
            format!("{:.2}", to_return)
        }
    }

    fn stringify_score_uci(self) -> String {
        if self == INFINITY {
            return "inf".to_string();
        }
        if self == -INFINITY {
            return "-inf".to_string();
        }
        if is_checkmate(self) {
            let mut mate_string = String::from("mate ");
            let mate_distance = (CHECKMATE_SCORE - self.abs() + 1) / 2;
            mate_string += &mate_distance.to_string();
            return mate_string;
        }
        format!("cp {}", (self as i32 * 100) / PAWN_VALUE as i32)
    }
}

impl Stringify for Score {
    fn stringify(&self) -> String {
        if is_in_console_mode() {
            self.stringify_score()
        } else {
            self.stringify_score_uci()
        }
    }
}

pub trait StringifyMove {
    fn uci(self) -> String;
    fn algebraic(self, board: &Board, long: bool) -> Result<String, BoardError>;
    fn san(self, board: &Board) -> Result<String, BoardError>
    where
        Self: Sized,
    {
        self.algebraic(board, false)
    }
    fn lan(self, board: &Board) -> Result<String, BoardError>
    where
        Self: Sized,
    {
        self.algebraic(board, true)
    }
    fn stringify_move(self, board: &Board) -> Result<String, BoardError>;
}

impl StringifyMove for Option<Move> {
    fn uci(self) -> String {
        match self {
            Some(m) => m.to_string(),
            None => String::from("0000"),
        }
    }

    fn algebraic(self, board: &Board, long: bool) -> Result<String, BoardError> {
        board.clone().algebraic_and_push(self, long)
    }

    fn stringify_move(self, board: &Board) -> Result<String, BoardError> {
        if is_in_console_mode() {
            self.algebraic(board, use_long_algebraic_notation())
        } else {
            Ok(self.uci())
        }
    }
}

impl StringifyMove for Move {
    fn uci(self) -> String {
        Some(self).uci()
    }

    fn algebraic(self, board: &Board, long: bool) -> Result<String, BoardError> {
        Some(self).algebraic(board, long)
    }

    fn stringify_move(self, board: &Board) -> Result<String, BoardError> {
        Some(self).stringify_move(board)
    }
}

impl Stringify for u64 {
    fn stringify(&self) -> String {
        format!("{:x}", self).to_uppercase()
    }
}

impl Stringify for BitBoard {
    fn stringify(&self) -> String {
        let mut checkers_string = String::new();
        for square in *self {
            checkers_string += &(square.to_string() + " ");
        }
        checkers_string.trim().to_uppercase()
    }
}

impl Stringify for Move {
    fn stringify(&self) -> String {
        self.uci()
    }
}

impl Stringify for WeightedMove {
    fn stringify(&self) -> String {
        format!("({}, {})", self.move_.stringify(), self.weight)
    }
}

impl<T: Stringify> Stringify for Option<T> {
    fn stringify(&self) -> String {
        match self {
            Some(t) => format!("Some({})", t.stringify()),
            None => String::from("None"),
        }
    }
}

impl<T: Stringify, E: Error> Stringify for Result<T, E> {
    fn stringify(&self) -> String {
        match self {
            Ok(t) => format!("Ok({})", t.stringify()),
            Err(e) => format!("Err({})", e),
        }
    }
}

impl<T: Stringify> Stringify for [T] {
    fn stringify(&self) -> String {
        format!("[{}]", self.iter().map(|t| t.stringify()).join(", "))
    }
}

impl<T: Stringify> Stringify for Vec<T> {
    fn stringify(&self) -> String {
        self.as_slice().stringify()
    }
}

impl Stringify for CacheTableSize {
    fn stringify(&self) -> String {
        format!("{self}")
    }
}

impl Stringify for PieceType {
    fn stringify(&self) -> String {
        match self {
            Pawn => "Pawn",
            Knight => "Knight",
            Bishop => "Bishop",
            Rook => "Rook",
            Queen => "Queen",
            King => "King",
        }
        .to_string()
    }
}

impl Stringify for Color {
    fn stringify(&self) -> String {
        match self {
            White => "White",
            Black => "Black",
        }
        .to_string()
    }
}
