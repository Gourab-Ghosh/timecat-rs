use super::*;
pub use PieceType::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    #[inline(always)]
    pub fn to_int(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub fn to_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn to_string(self, color: Color) -> String {
        match color {
            White => format!("{self}").to_uppercase(),
            Black => format!("{self}"),
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Pawn => "p",
                Knight => "n",
                Bishop => "b",
                Rook => "r",
                Queen => "q",
                King => "k",
            }
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub struct Piece {
    type_: PieceType,
    color: Color,
}

impl Piece {
    #[inline(always)]
    pub fn new(type_: PieceType, color: Color) -> Self {
        Self { type_, color }
    }

    #[inline(always)]
    pub fn get_piece_type(self) -> PieceType {
        self.type_
    }

    #[inline(always)]
    pub fn get_color(self) -> Color {
        self.color
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_.to_string(self.color))
    }
}
