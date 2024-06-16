use timecat::*;

#[rustfmt::skip]
#[test]
fn position_is_sane() {
    // TODO: Add some fens to test wrong fens
    let mut board = Board::default();
    let fens = [
        (STARTING_POSITION_FEN, true),
        ("Bk3n2/8/P1p1n3/2b5/1B2p3/5p2/1KP3pP/1Q1r4 w - - 2 5", true),
        ("q7/2R3K1/5n2/PNR5/3pr3/2P1P2k/1p2p3/5b1B w - - 30 50", true),
        ("8/r7/p4K2/1kPp2P1/2p1P1pb/1P1b2p1/3p4/1r6 w - - 22 51", true),
        ("5k2/p2P3K/Q7/4p3/3P2bp/2rN4/p1np2qp/8 w - - 11 23", true),
        ("4R3/4P2P/5P2/p1p2Bp1/1pP5/2p5/P6B/2k2K1Q w - - 0 1", true),
        ("rnbqkbnr/pppp2pp/4p3/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3", true),
        ("rnbqkbnr/pp1pppp1/8/1Pp4p/8/8/P1PPPPPP/RNBQKBNR w KQkq c6 0 3", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w K - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Q - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w k - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w q - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kq - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQk - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kkq - 0 1", true),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Qkq - 0 1", true),
    ];

    for (fen, expected_value) in fens {
        assert_eq!(
            board.set_fen(fen).is_ok(),
            expected_value,
            "Expected {expected_value} for position {fen}"
        );
    }
}
