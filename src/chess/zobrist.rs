use super::*;

pub struct Zobrist;

include!("zobrist_gen.rs");

impl Zobrist {
    /// Get the value for a particular piece
    #[inline]
    pub fn piece(piece: PieceType, square: Square, color: Color) -> u64 {
        *get_item_unchecked!(
            ZOBRIST_PIECES,
            color.to_index(),
            piece.to_index(),
            square.to_index(),
        )
    }

    #[inline]
    pub fn castles(castle_rights: CastleRights, color: Color) -> u64 {
        *get_item_unchecked!(
            ZOBRIST_CASTLES,
            color.to_index(),
            castle_rights.to_index(),
        )
    }

    #[inline]
    pub fn en_passant(file: File, color: Color) -> u64 {
        *get_item_unchecked!(
            ZOBRIST_EP,
            color.to_index(),
            file.to_index(),
        )
    }

    #[inline]
    pub fn color() -> u64 {
        TURN
    }
}
