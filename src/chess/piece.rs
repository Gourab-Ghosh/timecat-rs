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

pub const NUM_PIECE_TYPES: usize = 6;

pub const ALL_PIECE_TYPES: [PieceType; NUM_PIECE_TYPES] = [Pawn, Knight, Bishop, Rook, Queen, King];

pub const NUM_PROMOTION_PIECES: usize = 4;

pub const PROMOTION_PIECES: [PieceType; NUM_PROMOTION_PIECES] = [Queen, Knight, Rook, Bishop];

impl PieceType {
    pub fn to_int(self) -> u8 {
        self as u8
    }

    pub fn to_index(self) -> usize {
        self as usize
    }

    pub fn to_string(self, color: Color) -> String {
        match color {
            Color::White => format!("{self}").to_uppercase(),
            Color::Black => format!("{self}"),
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pawn => "p",
                Self::Knight => "n",
                Self::Bishop => "b",
                Self::Rook => "r",
                Self::Queen => "q",
                Self::King => "k",
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
    pub fn new(type_: PieceType, color: Color) -> Self {
        Self { type_, color }
    }

    pub fn get_piece_type(self) -> PieceType {
        self.type_
    }

    pub fn get_color(self) -> Color {
        self.color
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_.to_string(self.color),)
    }
}
