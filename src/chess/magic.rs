#[cfg(target_feature = "bmi2")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};

use super::*;

include!("magic_gen.rs");

/// Get the rays for a bishop on a particular square.
#[inline]
pub fn get_bishop_rays(square: Square) -> BitBoard {
    unsafe { *RAYS.get_unchecked(BISHOP).get_unchecked(square.to_index()) }
}

/// Get the rays for a rook on a particular square.
#[inline]
pub fn get_rook_rays(square: Square) -> BitBoard {
    unsafe { *RAYS.get_unchecked(ROOK).get_unchecked(square.to_index()) }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(not(target_feature = "bmi2"))]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS
            .get_unchecked(ROOK)
            .get_unchecked(square.to_index());
        *MOVES.get_unchecked(
            (magic.offset as usize)
                + (magic.magic_number.wrapping_mul(blockers & magic.mask))
                    .to_size(magic.right_shift),
        ) & get_rook_rays(square)
    }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(target_feature = "bmi2")]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let bmi2_magic = *ROOK_BMI_MASK.get_unchecked(square.to_index());
        let index = (_pext_u64(blockers.get_mask(), bmi2_magic.blockers_mask.get_mask()) as usize)
            + (bmi2_magic.offset as usize);
        let result = _pdep_u64(
            *BMI_MOVES.get_unchecked(index as usize) as u64,
            get_rook_rays(square).get_mask(),
        );
        return BitBoard::new(result);
    }
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(not(target_feature = "bmi2"))]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: Magic = *MAGIC_NUMBERS
            .get_unchecked(BISHOP)
            .get_unchecked(square.to_index());
        *MOVES.get_unchecked(
            (magic.offset as usize)
                + (magic.magic_number.wrapping_mul(blockers & magic.mask))
                    .to_size(magic.right_shift),
        ) & get_bishop_rays(square)
    }
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
#[cfg(target_feature = "bmi2")]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let bmi2_magic = *BISHOP_BMI_MASK.get_unchecked(square.to_index());
        let index = (_pext_u64(blockers.get_mask(), bmi2_magic.blockers_mask.get_mask()) as usize)
            + (bmi2_magic.offset as usize);
        let result = _pdep_u64(
            *BMI_MOVES.get_unchecked(index as usize) as u64,
            get_bishop_rays(square).get_mask(),
        );
        return BitBoard::new(result);
    }
}

#[inline]
pub fn get_queen_moves(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_moves(square, blockers) ^ get_bishop_moves(square, blockers)
}

/// Get the king moves for a particular square.
#[inline]
pub fn get_king_moves(square: Square) -> BitBoard {
    unsafe { *KING_MOVES.get_unchecked(square.to_index()) }
}

/// Get the knight moves for a particular square.
#[inline]
pub fn get_knight_moves(square: Square) -> BitBoard {
    unsafe { *KNIGHT_MOVES.get_unchecked(square.to_index()) }
}

/// Get the pawn capture move for a particular square, given the pawn's color and the potential
/// victims
#[inline]
pub fn get_pawn_attacks(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    unsafe {
        *PAWN_ATTACKS
            .get_unchecked(color.to_index())
            .get_unchecked(square.to_index())
            & blockers
    }
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
    unsafe {
        if !(square.wrapping_forward(color).to_bitboard() & blockers).is_empty() {
            BB_EMPTY
        } else {
            *PAWN_MOVES
                .get_unchecked(color.to_index())
                .get_unchecked(square.to_index())
                & !blockers
        }
    }
}

/// Get all the pawn moves for a particular square, given the pawn's color and the potential
/// blocking pieces and victims.
#[inline]
pub fn get_pawn_moves(square: Square, color: Color, blockers: BitBoard) -> BitBoard {
    get_pawn_attacks(square, color, blockers) ^ get_pawn_quiets(square, color, blockers)
}

/// Get a line (extending to infinity, which in chess is 8 squares), given two squares.
/// This line does extend past the squares.
#[inline]
pub fn line(square1: Square, square2: Square) -> BitBoard {
    unsafe {
        *LINE
            .get_unchecked(square1.to_index())
            .get_unchecked(square2.to_index())
    }
}

/// Get a line between these two squares, not including the squares themselves.
#[inline]
pub fn between(square1: Square, square2: Square) -> BitBoard {
    unsafe {
        *BETWEEN
            .get_unchecked(square1.to_index())
            .get_unchecked(square2.to_index())
    }
}

/// Get a `BitBoard` that represents all the squares on a particular rank.
#[inline]
pub fn get_rank_bb(rank: Rank) -> BitBoard {
    unsafe { *RANKS.get_unchecked(rank.to_index()) }
}

/// Get a `BitBoard` that represents all the squares on a particular file.
#[inline]
pub fn get_file_bb(file: File) -> BitBoard {
    unsafe { *FILES.get_unchecked(file.to_index()) }
}

/// Get a `BitBoard` that represents the squares on the 1 or 2 files next to this file.
#[inline]
pub fn get_adjacent_files(file: File) -> BitBoard {
    unsafe { *ADJACENT_FILES.get_unchecked(file.to_index()) }
}

#[inline]
pub fn get_pawn_source_double_moves() -> BitBoard {
    PAWN_SOURCE_DOUBLE_MOVES
}

#[inline]
pub fn get_pawn_dest_double_moves() -> BitBoard {
    PAWN_DEST_DOUBLE_MOVES
}
