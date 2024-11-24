use super::*;
#[cfg(target_feature = "bmi2")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};

include!("magic_gen.rs");

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[cfg(not(target_feature = "bmi2"))]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic =
        *get_item_unchecked!(const BISHOP_AND_ROOK_MAGIC_NUMBERS[0], square.to_index());
    *get_item_unchecked!(
        MOVES,
        magic.offset
            + (magic.magic_number.wrapping_mul(blockers & magic.mask) >> magic.right_shift)
                .to_usize(),
    ) & square.get_bishop_rays_bb()
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[cfg(target_feature = "bmi2")]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic = *get_item_unchecked!(const BISHOP_AND_ROOK_BMI_MASKS[0], square.to_index());
    let index = unsafe { _pext_u64(blockers.into_inner(), bmi2_magic.blockers_mask.into_inner()) }
        as usize
        + bmi2_magic.offset;
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_bishop_rays_bb().into_inner(),
        )
    };
    return BitBoard::new(result);
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[cfg(not(target_feature = "bmi2"))]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic =
        *get_item_unchecked!(const BISHOP_AND_ROOK_MAGIC_NUMBERS[1], square.to_index());
    *get_item_unchecked!(
        MOVES,
        magic.offset
            + (magic.magic_number.wrapping_mul(blockers & magic.mask) >> magic.right_shift)
                .to_usize(),
    ) & square.get_rook_rays_bb()
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[cfg(target_feature = "bmi2")]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic = *get_item_unchecked!(const BISHOP_AND_ROOK_BMI_MASKS[1], square.to_index());
    let index = unsafe { _pext_u64(blockers.into_inner(), bmi2_magic.blockers_mask.into_inner()) }
        as usize
        + bmi2_magic.offset;
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_rook_rays_bb().into_inner(),
        )
    };
    return BitBoard::new(result);
}

#[inline]
pub fn get_queen_moves(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_moves(square, blockers) ^ get_bishop_moves(square, blockers)
}

/// Get the legal destination castle squares for both players
#[inline]
pub fn get_castle_moves() -> BitBoard {
    const { BitBoard::new(6052837899185946708) }
}

#[inline]
pub const fn get_pawn_source_double_moves() -> BitBoard {
    const { BitBoard::new(BB_RANK_2.into_inner() ^ BB_RANK_7.into_inner()) }
}

#[inline]
pub const fn get_pawn_dest_double_moves() -> BitBoard {
    const { BitBoard::new(BB_RANK_4.into_inner() ^ BB_RANK_5.into_inner()) }
}
