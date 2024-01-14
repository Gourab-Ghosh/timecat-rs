use super::*;

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
    pub fn to_index(self) -> usize {
        self as usize
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

    pub fn get_piece_type(&self) -> PieceType {
        self.type_
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.color {
                Color::White => self.type_.to_string().to_uppercase(),
                Color::Black => self.type_.to_string(),
            }
        )
    }
}
