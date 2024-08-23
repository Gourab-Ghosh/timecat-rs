use super::*;
pub use PieceType::*;

pub mod all_pieces {
    #![allow(non_upper_case_globals)]
    use super::*;

    pub const WhitePawn: Piece = Piece::new(Pawn, White);
    pub const WhiteKnight: Piece = Piece::new(Knight, White);
    pub const WhiteBishop: Piece = Piece::new(Bishop, White);
    pub const WhiteRook: Piece = Piece::new(Rook, White);
    pub const WhiteQueen: Piece = Piece::new(Queen, White);
    pub const WhiteKing: Piece = Piece::new(King, White);
    pub const BlackPawn: Piece = Piece::new(Pawn, Black);
    pub const BlackKnight: Piece = Piece::new(Knight, Black);
    pub const BlackBishop: Piece = Piece::new(Bishop, Black);
    pub const BlackRook: Piece = Piece::new(Rook, Black);
    pub const BlackQueen: Piece = Piece::new(Queen, Black);
    pub const BlackKing: Piece = Piece::new(King, Black);
}

pub use all_pieces::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    #[inline]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn to_colored_piece(self, color: Color) -> Piece {
        Piece::new(self, color)
    }

    #[inline]
    pub fn to_colored_piece_string(self, color: Color) -> String {
        self.to_colored_piece(color).to_string()
    }

    #[inline]
    pub const fn evaluate(self) -> i16 {
        // never reset knight and bishop values as some logic depends on the current values in knight bishop endgame
        match self {
            Pawn => PAWN_VALUE,
            Knight => const { (32 * PAWN_VALUE) / 10 },
            Bishop => const { (33 * PAWN_VALUE) / 10 },
            Rook => 5 * PAWN_VALUE,
            Queen => 9 * PAWN_VALUE,
            King => 20 * PAWN_VALUE,
        }
    }
}

impl FromStr for PieceType {
    type Err = TimecatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "p" => Ok(Pawn),
            "n" => Ok(Knight),
            "b" => Ok(Bishop),
            "r" => Ok(Rook),
            "q" => Ok(Queen),
            "k" => Ok(King),
            _ => Err(TimecatError::InvalidPieceTypeString { s: s.to_string() }),
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            get_item_unchecked!(const ["p", "n", "b", "r", "q", "k"], self.to_index()),
        )
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for PieceType {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(piece_type_text) = ob.extract::<&str>() {
            if let Ok(piece_type) = Self::from_str(piece_type_text) {
                return Ok(piece_type);
            }
        }
        if let Ok(piece_type_index) = ob.extract::<usize>() {
            if let Some(&piece_type) = ALL_PIECE_TYPES.get(piece_type_index) {
                return Ok(piece_type);
            }
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub struct Piece {
    type_: PieceType,
    color: Color,
}

impl Piece {
    #[inline]
    pub const fn new(type_: PieceType, color: Color) -> Self {
        Self { type_, color }
    }

    #[inline]
    pub const fn get_piece_type(self) -> PieceType {
        self.type_
    }

    #[inline]
    pub const fn get_color(self) -> Color {
        self.color
    }

    #[inline]
    pub fn flip_color(&mut self) {
        self.color = !self.color
    }

    #[inline]
    pub fn to_int(self) -> u8 {
        //TODO: Replace with match statements
        self.to_index() as u8
    }

    #[inline]
    pub fn to_index(self) -> usize {
        //TODO: Replace with match statements
        NUM_COLORS * self.get_piece_type().to_index() + self.get_color().to_index()
    }

    #[inline]
    pub fn evaluate(self) -> Score {
        if self.get_color() == White {
            self.get_piece_type().evaluate()
        } else {
            -self.get_piece_type().evaluate()
        }
    }

    #[cfg(feature = "pyo3")]
    fn from_py_piece(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self::new(
            ob.getattr("piece_type")?.extract()?,
            ob.getattr("color")?.extract()?,
        ))
    }
}

impl FromStr for Piece {
    type Err = TimecatError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim() {
            "p" => Ok(BlackPawn),
            "n" => Ok(BlackKnight),
            "b" => Ok(BlackBishop),
            "r" => Ok(BlackRook),
            "q" => Ok(BlackQueen),
            "k" => Ok(BlackKing),
            "P" => Ok(WhitePawn),
            "N" => Ok(WhiteKnight),
            "B" => Ok(WhiteBishop),
            "R" => Ok(WhiteRook),
            "Q" => Ok(WhiteQueen),
            "K" => Ok(WhiteKing),
            _ => Err(TimecatError::InvalidPieceString { s: s.to_string() }),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.get_color() {
                White => self.get_piece_type().to_string().to_uppercase(),
                Black => self.get_piece_type().to_string(),
            }
        )
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for Piece {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(piece_text) = ob.extract::<&str>() {
            if let Ok(piece) = Self::from_str(piece_text) {
                return Ok(piece);
            }
        }
        if let Ok(piece) = Self::from_py_piece(ob) {
            return Ok(piece);
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
