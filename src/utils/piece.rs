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
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
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
    pub fn to_string(self, color: Color) -> String {
        match color {
            White => format!("{self}").to_uppercase(),
            Black => format!("{self}"),
        }
    }

    #[inline]
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
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.type_.to_string(self.color))
    }
}
