use super::*;

#[inline]
pub const fn is_checkmate(score: Score) -> bool {
    let abs_score = score.abs();
    abs_score > CHECKMATE_THRESHOLD && abs_score <= CHECKMATE_SCORE
}
