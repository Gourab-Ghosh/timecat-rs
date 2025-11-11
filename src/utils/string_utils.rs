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
    let (position_fen, rest_fen) = fen.split_once(' ').ok_or_else(|| TimecatError::BadFen {
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
        if style_functions.is_empty() || !GLOBAL_TIMECAT_STATE.is_colored_output() {
            return self_string;
        }
        let mut colorized_string = self_string.into();
        for &func in style_functions {
            colorized_string = func(colorized_string);
        }
        colorized_string.to_string()
    }
}

#[cfg(not(feature = "colored"))]
impl<T: ToString> CustomColorize for T {
    #[inline]
    fn colorize(&self, _: &[ColoredStringFunction]) -> String {
        self.to_string()
    }
}

impl StringifyScore for Score {
    fn stringify_score_console<'a>(self) -> Cow<'a, str> {
        if self == INFINITY {
            return "INFINITY".into();
        }
        if self == -INFINITY {
            return "-INFINITY".into();
        }
        if is_checkmate(self) {
            let mut mate_string = String::from(if self.is_positive() { "M" } else { "-M" });
            let mate_distance = (CHECKMATE_SCORE - self.abs() + 1) / 2;
            write_unchecked!(mate_string, "{}", mate_distance);
            return mate_string.colorize(CHECKMATE_SCORE_STYLE).into();
        }
        let to_return = self as f64 / PAWN_VALUE as f64;
        if to_return.is_integer() {
            format!("{}", to_return as i32).into()
        } else {
            format!("{:.2}", to_return).into()
        }
    }

    fn stringify_score_uci<'a>(self) -> Cow<'a, str> {
        if self == INFINITY {
            return "inf".into();
        }
        if self == -INFINITY {
            return "-inf".into();
        }
        if is_checkmate(self) {
            let mut mate_string = String::from("mate ");
            let mut mate_distance = (CHECKMATE_SCORE - self.abs() + 1) / 2;
            if self.is_negative() {
                mate_distance = -mate_distance;
            }
            write_unchecked!(mate_string, "{}", mate_distance);
            return mate_string.into();
        }
        format!("cp {}", (self as i32 * 100) / PAWN_VALUE as i32).into()
    }

    #[inline]
    fn stringify_score<'a>(self) -> Cow<'a, str> {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            self.stringify_score_console()
        } else {
            self.stringify_score_uci()
        }
    }
}

impl Stringify for Score {
    #[inline]
    fn stringify<'a>(&self) -> Cow<'a, str> {
        self.stringify_score()
    }
}

impl Stringify for TimecatError {
    #[inline]
    fn stringify<'a>(&self) -> Cow<'a, str> {
        self.stringify_with_optional_raw_input(None).into()
    }
}

impl StringifyMove for Move {
    fn uci<'a>(self) -> Cow<'a, str> {
        self.to_string().into()
    }

    fn algebraic<'a>(self, position: &ChessPosition, long: bool) -> Result<Cow<'a, str>> {
        Ok(self.algebraic_and_new_position(position, long)?.0.into())
    }

    fn stringify_move<'a>(self, position: &ChessPosition) -> Result<Cow<'a, str>> {
        Some(self).stringify_move(position)
    }
}

impl StringifyMove for Option<Move> {
    fn uci<'a>(self) -> Cow<'a, str> {
        match self {
            Some(m) => m.uci().into(),
            None => "0000".into(),
        }
    }

    fn algebraic<'a>(self, position: &ChessPosition, long: bool) -> Result<Cow<'a, str>> {
        match self {
            Some(valid_or_null_move) => valid_or_null_move.algebraic(position, long),
            None => Ok("--".into()),
        }
    }

    fn stringify_move<'a>(self, position: &ChessPosition) -> Result<Cow<'a, str>> {
        match GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            true => self.algebraic(position, GLOBAL_TIMECAT_STATE.use_long_algebraic_notation()),
            false => Ok(self.uci()),
        }
    }
}

impl StringifyHash for u64 {
    fn stringify_hash(&self) -> String {
        format!("{:X}", self)
    }
}

impl Stringify for Duration {
    fn stringify<'a>(&self) -> Cow<'a, str> {
        if GLOBAL_TIMECAT_STATE.is_in_uci_mode() {
            return self.as_millis().to_string().into();
        }
        if self < &Duration::from_secs(1) {
            return format!("{} ms", self.as_millis()).into();
        }
        let precision = 3;
        let total_secs = self.as_secs_f64();
        for (threshold, unit) in [(86400.0, "day"), (3600.0, "hr"), (60.0, "min")] {
            if total_secs >= threshold {
                let time_unit = total_secs as u128 / threshold as u128;
                let secs = total_secs % threshold;
                let mut string = format!("{} {}", time_unit, unit);
                if time_unit > 1 {
                    string.push('s');
                }
                if secs >= 10.0_f64.powi(-(precision as i32)) {
                    string.push(' ');
                    string += &Duration::from_secs_f64(secs).stringify();
                }
                return string.into();
            }
        }
        let total_secs_rounded = total_secs.round();
        let mut string = if (total_secs - total_secs_rounded).abs() < 1e-5 {
            format!("{} sec", total_secs_rounded)
        } else {
            format!("{:.1$} sec", total_secs, precision)
        };
        if total_secs > 1.0 {
            string.push('s');
        }
        string.into()
    }
}

macro_rules! implement_stringify {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Stringify for $type {
                fn stringify<'a>(&self) -> Cow<'a, str> {
                    self.to_string().into()
                }
            }
        )*
    };
}

implement_stringify!(Move, ValidOrNullMove, WeightedMove, Color, PieceType, Piece);

impl<T: Stringify> Stringify for Option<T> {
    fn stringify<'a>(&self) -> Cow<'a, str> {
        match self {
            Some(t) => t.stringify(),
            None => STRINGIFY_NONE.into(),
        }
    }
}

impl<T: Stringify, E: Error> Stringify for std::result::Result<T, E> {
    fn stringify<'a>(&self) -> Cow<'a, str> {
        match self {
            Ok(t) => format!("Ok({})", t.stringify()).into(),
            Err(e) => format!("Err({})", e).into(),
        }
    }
}

impl<T: Stringify> Stringify for [T] {
    fn stringify<'a>(&self) -> Cow<'a, str> {
        format!("[{}]", self.iter().map(|t| t.stringify()).join(", ")).into()
    }
}

impl<T: Stringify> Stringify for Vec<T> {
    fn stringify<'a>(&self) -> Cow<'a, str> {
        self.as_slice().stringify()
    }
}
