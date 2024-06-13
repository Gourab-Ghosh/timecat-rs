use super::*;

#[inline]
pub const fn is_checkmate(score: Score) -> bool {
    let abs_score = score.abs();
    abs_score > CHECKMATE_THRESHOLD && abs_score <= CHECKMATE_SCORE
}

#[inline]
pub fn get_upper_board_mask(rank: Rank, color: Color) -> BitBoard {
    *get_item_unchecked!(UPPER_BOARD_MASK, color.to_index(), rank.to_index())
}

#[inline]
pub fn get_lower_board_mask(rank: Rank, color: Color) -> BitBoard {
    get_upper_board_mask(rank, !color)
}
