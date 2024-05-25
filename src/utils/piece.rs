use super::*;
pub use PieceType::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn to_string(self, color: Color) -> String {
        match color {
            White => format!("{self}").to_uppercase(),
            Black => format!("{self}"),
        }
    }

    #[inline(always)]
    pub const fn evaluate(self) -> i16 {
        // never reset knight and bishop values as some logic depends on the current values in knight bishop endgame
        match self {
            Pawn => PAWN_VALUE,
            Knight => (32 * PAWN_VALUE) / 10,
            Bishop => (33 * PAWN_VALUE) / 10,
            Rook => 5 * PAWN_VALUE,
            Queen => 9 * PAWN_VALUE,
            King => 20 * PAWN_VALUE,
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub struct Piece {
    type_: PieceType,
    color: Color,
}

impl Piece {
    #[inline(always)]
    pub const fn new(type_: PieceType, color: Color) -> Self {
        Self { type_, color }
    }

    #[inline(always)]
    pub const fn get_piece_type(self) -> PieceType {
        self.type_
    }

    #[inline(always)]
    pub const fn get_color(self) -> Color {
        self.color
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_.to_string(self.color))
    }
}
