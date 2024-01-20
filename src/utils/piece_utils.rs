use super::*;

#[inline(always)]
pub const fn evaluate_piece(piece: PieceType) -> i16 {
    // never set knight and bishop values as same for knight bishop endgame
    match piece {
        Pawn => PAWN_VALUE,
        Knight => (32 * PAWN_VALUE) / 10,
        Bishop => (33 * PAWN_VALUE) / 10,
        Rook => 5 * PAWN_VALUE,
        Queen => 9 * PAWN_VALUE,
        King => 20 * PAWN_VALUE,
    }
}

pub trait PieceTypeTrait {
    type PieceType;

    fn get_type(self) -> Self::PieceType;
}

impl PieceTypeTrait for PieceType {
    type PieceType = u8;

    fn get_type(self) -> Self::PieceType {
        Some(self).get_type()
    }
}

impl PieceTypeTrait for Option<PieceType> {
    type PieceType = u8;

    fn get_type(self) -> Self::PieceType {
        match self {
            Some(piece) => piece.to_index() as Self::PieceType + 1,
            None => 0,
        }
    }
}
