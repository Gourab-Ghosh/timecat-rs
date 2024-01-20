use super::*;

#[inline(always)]
pub fn square_mirror(square: Square) -> Square {
    get_item_unchecked!(SQUARES_180, square.to_index())
}

#[inline(always)]
pub fn get_square_bb(square: Square) -> BitBoard {
    get_item_unchecked!(BB_SQUARES, square.to_index())
}

pub fn square_distance(square1: Square, square2: Square) -> u8 {
    let (file1, rank1) = (square1.get_file(), square1.get_rank());
    let (file2, rank2) = (square2.get_file(), square2.get_rank());
    let file_distance = (file1 as i8).abs_diff(file2 as i8);
    let rank_distance = (rank1 as i8).abs_diff(rank2 as i8);
    file_distance.max(rank_distance)
}
