use super::*;
#[cfg(target_feature = "bmi2")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};

include!("magic_gen.rs");

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(not(target_feature = "bmi2"))]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic =
        *get_item_unchecked!(const ROOK_AND_BISHOP_MAGIC_NUMBERS[0], square.to_index());
    *get_item_unchecked!(
        MOVES,
        (magic.offset as usize)
            + (magic.magic_number.wrapping_mul(blockers & magic.mask) >> magic.right_shift)
                .to_usize(),
    ) & square.get_rook_rays_bb()
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(target_feature = "bmi2")]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic = *get_item_unchecked!(ROOK_BMI_MASK, square.to_index());
    let index =
        unsafe { _pext_u64(blockers.get_mask(), bmi2_magic.blockers_mask.get_mask()) as usize }
            + (bmi2_magic.offset as usize);
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_rook_rays_bb().get_mask(),
        )
    };
    return BitBoard::new(result);
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(not(target_feature = "bmi2"))]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic =
        *get_item_unchecked!(const ROOK_AND_BISHOP_MAGIC_NUMBERS[1], square.to_index());
    *get_item_unchecked!(
        MOVES,
        (magic.offset as usize)
            + (magic.magic_number.wrapping_mul(blockers & magic.mask) >> magic.right_shift)
                .to_usize(),
    ) & square.get_bishop_rays_bb()
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(target_feature = "bmi2")]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic = *get_item_unchecked!(BISHOP_BMI_MASK, square.to_index());
    let index =
        unsafe { _pext_u64(blockers.get_mask(), bmi2_magic.blockers_mask.get_mask()) as usize }
            + (bmi2_magic.offset as usize);
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_bishop_rays_bb().get_mask(),
        )
    };
    return BitBoard::new(result);
}

#[inline]
pub fn get_queen_moves(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_moves(square, blockers) ^ get_bishop_moves(square, blockers)
}

/// Get the king moves for a particular square.
#[inline]
pub fn get_king_moves(square: Square) -> BitBoard {
    *get_item_unchecked!(KING_MOVES, square.to_index())
}

/// Get the knight moves for a particular square.
#[inline]
pub fn get_knight_moves(square: Square) -> BitBoard {
    *get_item_unchecked!(KNIGHT_MOVES, square.to_index())
}

/// Get the pawn capture move for a particular square, given the pawn's color and the potential
/// victims
#[inline]
pub fn get_pawn_attacks(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    *get_item_unchecked!(PAWN_ATTACKS, color.to_index(), square.to_index()) & blockers
}
/// Get the legal destination castle squares for both players
#[inline]
pub fn get_castle_moves() -> BitBoard {
    CASTLE_MOVES
}

/// Get the quiet pawn moves (non-captures) for a particular square, given the pawn's color and
/// the potential blocking pieces.
#[inline]
pub fn get_pawn_quiets(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    if !(square.wrapping_forward(color).to_bitboard() & blockers).is_empty() {
        BB_EMPTY
    } else {
        *get_item_unchecked!(PAWN_MOVES, color.to_index(), square.to_index()) & !blockers
    }
}

/// Get all the pawn moves for a particular square, given the pawn's color and the potential
/// blocking pieces and victims.
#[inline]
pub fn get_pawn_moves(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    get_pawn_attacks(square, color, blockers) ^ get_pawn_quiets(square, color, blockers)
}

#[inline]
pub fn get_pawn_source_double_moves() -> BitBoard {
    PAWN_SOURCE_DOUBLE_MOVES
}

#[inline]
pub fn get_pawn_dest_double_moves() -> BitBoard {
    PAWN_DEST_DOUBLE_MOVES
}
