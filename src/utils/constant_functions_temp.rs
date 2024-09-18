// into_inner, BB_SQUARES, BB_RANKS, BB_FILES, to_int, to_index

use super::*;

#[inline(always)]
pub(crate) const fn bb_not(bb: BitBoard) -> BitBoard {
    BitBoard::new(!bb.into_inner())
}

#[inline(always)]
pub(crate) const fn bb_and(bb1: BitBoard, bb2: BitBoard) -> BitBoard {
    BitBoard::new(bb1.into_inner() & bb2.into_inner())
}

#[inline(always)]
pub(crate) const fn bb_or(bb1: BitBoard, bb2: BitBoard) -> BitBoard {
    BitBoard::new(bb1.into_inner() | bb2.into_inner())
}

#[inline(always)]
pub(crate) const fn bb_xor(bb1: BitBoard, bb2: BitBoard) -> BitBoard {
    BitBoard::new(bb1.into_inner() ^ bb2.into_inner())
}

#[inline(always)]
pub(crate) const fn bb_eq(bb1: BitBoard, bb2: BitBoard) -> bool {
    bb1.into_inner() == bb2.into_inner()
}

#[inline(always)]
pub(crate) const fn bb_neq(bb1: BitBoard, bb2: BitBoard) -> bool {
    bb1.into_inner() == bb2.into_inner()
}

#[inline(always)]
pub(crate) const fn bb_contains(bb: BitBoard, square: Square) -> bool {
    !bb_and(bb, square_to_bitboard(square)).is_empty()
}

#[inline(always)]
pub(crate) const fn square_to_bitboard(square: Square) -> BitBoard {
    BB_SQUARES[square.to_index()]
}
#[inline(always)]
pub(crate) const fn square_eq(square1: Square, square2: Square) -> bool {
    square1.to_int() == square2.to_int()
}
#[inline(always)]
pub(crate) const fn square_neq(square1: Square, square2: Square) -> bool {
    square1.to_int() != square2.to_int()
}

#[inline(always)]
pub(crate) const fn u8_cmp(int1: u8, int2: u8) -> Ordering {
    if int1 > int2 {
        return Ordering::Greater;
    }
    if int1 < int2 {
        return Ordering::Less;
    }
    Ordering::Equal
}

#[inline(always)]
pub(crate) const fn rank_to_bitboard(rank: Rank) -> BitBoard {
    BB_RANKS[rank.to_index()]
}

#[inline(always)]
pub(crate) const fn file_to_bitboard(file: File) -> BitBoard {
    BB_FILES[file.to_index()]
}

#[inline(always)]
pub(crate) const fn bitboard_to_square_unchecked(bb: BitBoard) -> Square {
    ALL_SQUARES[bb.to_square_index_unchecked()]
}

#[inline(always)]
pub(crate) const fn bitboard_to_square(bb: BitBoard) -> Option<Square> {
    if let Some(square_index) = bb.to_square_index() {
        Some(ALL_SQUARES[square_index])
    } else {
        None
    }
}

#[inline(always)]
pub(crate) const fn unwrap_option<T>(option: &Option<T>) -> &T {
    match option {
        Some(value) => value,
        None => panic!("Tried to unwrap None variant!"),
    }
}
