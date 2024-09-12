use timecat::*;

#[test]
fn bitboard_to_square() {
    assert!((0..64)
        .map(|i| BitBoard::new(1 << i).to_square().unwrap())
        .zip(ALL_SQUARES)
        .all(|(a, b)| a == b))
}
