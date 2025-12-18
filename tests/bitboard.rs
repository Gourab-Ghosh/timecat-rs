use timecat::*;

#[test]
fn test_bitboard_to_square() {
    assert!(
        (0..64)
            .map(|i| BitBoard::new(1 << i).to_square().unwrap())
            .zip(ALL_SQUARES)
            .all(|(a, b)| a == b)
    )
}

#[test]
fn test_bitboard_shifts() {
    macro_rules! bitboard_assert {
        ($bitboard:expr, $method: ident, $method_single: ident, $n: expr) => {
            assert_eq!(
                $bitboard.$method($n as u8),
                {
                    let mut expected = $bitboard;
                    for _ in 0..$n {
                        expected = expected.$method_single();
                    }
                    expected
                },
                "{} at n = {}",
                stringify!($method),
                $n
            );
        };
    }

    for bitboard in BB_SQUARES
        .into_iter()
        .chain(iproduct!(BB_RANKS, BB_RANKS).map(|(r, f)| r | f))
        .chain(iproduct!(BB_FILES, BB_FILES).map(|(r, f)| r | f))
        .chain(iproduct!(BB_RANKS, BB_FILES).map(|(r, f)| r | f))
        .chain([BitBoard::EMPTY, BitBoard::ALL])
    {
        for n in 0..8_usize {
            bitboard_assert!(bitboard, shift_up_n_times, shift_up, n);
            bitboard_assert!(bitboard, shift_down_n_times, shift_down, n);
            bitboard_assert!(bitboard, shift_left_n_times, shift_left, n);
            bitboard_assert!(bitboard, shift_right_n_times, shift_right, n);
        }
    }

    for i in 8..16 {
        assert_eq!(BitBoard::ALL.shift_up_n_times(i), BitBoard::EMPTY);
        assert_eq!(BitBoard::ALL.shift_down_n_times(i), BitBoard::EMPTY);
        assert_eq!(BitBoard::ALL.shift_left_n_times(i), BitBoard::EMPTY);
        assert_eq!(BitBoard::ALL.shift_right_n_times(i), BitBoard::EMPTY);
    }
}
