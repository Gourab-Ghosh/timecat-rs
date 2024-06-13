use super::*;

pub struct Zobrist;

include!("zobrist_gen.rs");

impl Zobrist {
    /// Get the value for a particular piece
    #[inline]
    pub fn piece(piece: PieceType, square: Square, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_PIECES
                .get_unchecked(color.to_index())
                .get_unchecked(piece.to_index())
                .get_unchecked(square.to_index())
        }
    }

    #[inline]
    pub fn castles(castle_rights: CastleRights, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_CASTLES
                .get_unchecked(color.to_index())
                .get_unchecked(castle_rights.to_index())
        }
    }

    #[inline]
    pub fn en_passant(file: File, color: Color) -> u64 {
        unsafe {
            *ZOBRIST_EP
                .get_unchecked(color.to_index())
                .get_unchecked(file.to_index())
        }
    }

    #[inline]
    pub fn color() -> u64 {
        TURN
    }
}
