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
