use super::*;

#[inline]
pub fn remove_double_spaces_and_trim(s: &str) -> String {
    s.trim()
        .chars()
        .dedup_by(|&c1, &c2| c1 == ' ' && c2 == ' ')
        .join("")
}

#[inline]
pub fn simplify_fen(fen: &str) -> String {
    remove_double_spaces_and_trim(fen)
}

pub fn flip_board_fen(fen: &str) -> Result<String> {
    // TODO: ep square not flipped.
    let fen = remove_double_spaces_and_trim(fen);
    let (position_fen, rest_fen) = fen.split_once(' ').ok_or(TimecatError::BadFen {
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

#[cfg(feature = "colored")]
impl<T: ToString> CustomColorize for T {
    fn colorize(&self, style_functions: &[ColoredStringFunction]) -> String {
        let self_string = self.to_string();
        if style_functions.is_empty() || !GLOBAL_TIMECAT_STATE.is_colored() {
            return self_string;
        }
        let mut colorized_string = self_string.as_str().into();
        for &func in style_functions {
            colorized_string = func(colorized_string);
        }
        colorized_string.to_string()
    }
}

#[cfg(not(feature = "colored"))]
impl<T: ToString> CustomColorize for T {
    #[inline]
    fn colorize(&self, _: &[fn(String) -> String]) -> String {
        self.to_string()
    }
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
            return mate_string.colorize(CHECKMATE_SCORE_STYLE);
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
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            self.stringify_score()
        } else {
            self.stringify_score_uci()
        }
    }
}

impl StringifyMove for Move {
    fn uci(self) -> String {
        self.to_string()
    }

    fn algebraic(self, sub_board: &SubBoard, long: bool) -> Result<String> {
        Ok(self.algebraic_and_new_sub_board(sub_board, long)?.0)
    }

    fn stringify_move(self, sub_board: &SubBoard) -> Result<String> {
        Some(self).stringify_move(sub_board)
    }
}

impl StringifyMove for Option<Move> {
    fn uci(self) -> String {
        match self {
            Some(m) => m.uci(),
            None => String::from("0000"),
        }
    }

    fn algebraic(self, sub_board: &SubBoard, long: bool) -> Result<String> {
        match self {
            Some(valid_or_null_move) => valid_or_null_move.algebraic(sub_board, long),
            None => Ok("--".to_string()),
        }
    }

    fn stringify_move(self, sub_board: &SubBoard) -> Result<String> {
        match GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            true => self.algebraic(
                sub_board,
                GLOBAL_TIMECAT_STATE.use_long_algebraic_notation(),
            ),
            false => Ok(self.uci()),
        }
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

impl<T: Stringify, E: Error> Stringify for std::result::Result<T, E> {
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

impl Stringify for Duration {
    fn stringify(&self) -> String {
        if GLOBAL_TIMECAT_STATE.is_in_uci_mode() {
            return self.as_millis().to_string();
        }
        if self < &Duration::from_secs(1) {
            return self.as_millis().to_string() + " ms";
        }
        let precision = 3;
        let total_secs = self.as_secs_f64();
        for (threshold, unit) in [(86400.0, "day"), (3600.0, "hr"), (60.0, "min")] {
            if total_secs >= threshold {
                let time_unit = total_secs as u128 / threshold as u128;
                let secs = total_secs % threshold;
                let mut string = format!("{} {}", time_unit, unit);
                if time_unit > 1 {
                    string += "s";
                }
                if secs >= 10.0_f64.powi(-(precision as i32)) {
                    string += " ";
                    string += &Duration::from_secs_f64(secs).stringify();
                }
                return string;
            }
        }
        let total_secs_rounded = total_secs.round();
        let mut string = if (total_secs - total_secs_rounded).abs() < 1e-5 {
            format!("{} sec", total_secs_rounded)
        } else {
            format!("{:.1$} sec", total_secs, precision)
        };
        if total_secs > 1.0 {
            string += "s";
        }
        string
    }
}
