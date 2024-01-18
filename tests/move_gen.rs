use timecat::*;
use std::collections::HashSet;
use std::str::FromStr;

fn move_gen_perft_test(fen: String, depth: usize, result: usize) {
    let board: SubBoard = BoardBuilder::from_str(&fen).unwrap().try_into().unwrap();

    assert_eq!(MoveGen::perft_test(&board, depth), result);
    assert_eq!(MoveGen::perft_test_piecewise(&board, depth), result);
}

#[test]
fn move_gen_perft_kiwipete() {
    move_gen_perft_test(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(),
        5,
        193690690,
    );
}

#[test]
fn move_gen_perft_1() {
    move_gen_perft_test("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1".to_owned(), 6, 824064);
    // Invalid FEN
}

#[test]
fn move_gen_perft_2() {
    move_gen_perft_test("8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1".to_owned(), 6, 824064);
    // Invalid FEN
}

#[test]
fn move_gen_perft_3() {
    move_gen_perft_test("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1".to_owned(), 6, 1440467);
}

#[test]
fn move_gen_perft_4() {
    move_gen_perft_test("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1".to_owned(), 6, 1440467);
}

#[test]
fn move_gen_perft_5() {
    move_gen_perft_test("5k2/8/8/8/8/8/8/4K2R w K - 0 1".to_owned(), 6, 661072);
}

#[test]
fn move_gen_perft_6() {
    move_gen_perft_test("4k2r/8/8/8/8/8/8/5K2 b k - 0 1".to_owned(), 6, 661072);
}

#[test]
fn move_gen_perft_7() {
    move_gen_perft_test("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1".to_owned(), 6, 803711);
}

#[test]
fn move_gen_perft_8() {
    move_gen_perft_test("r3k3/8/8/8/8/8/8/3K4 b q - 0 1".to_owned(), 6, 803711);
}

#[test]
fn move_gen_perft_9() {
    move_gen_perft_test(
        "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1".to_owned(),
        4,
        1274206,
    );
}

#[test]
fn move_gen_perft_10() {
    move_gen_perft_test(
        "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1".to_owned(),
        4,
        1274206,
    );
}

#[test]
fn move_gen_perft_11() {
    move_gen_perft_test(
        "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1".to_owned(),
        4,
        1720476,
    );
}

#[test]
fn move_gen_perft_12() {
    move_gen_perft_test(
        "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1".to_owned(),
        4,
        1720476,
    );
}

#[test]
fn move_gen_perft_13() {
    move_gen_perft_test("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn move_gen_perft_14() {
    move_gen_perft_test("3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1".to_owned(), 6, 3821001);
}

#[test]
fn move_gen_perft_15() {
    move_gen_perft_test("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn move_gen_perft_16() {
    move_gen_perft_test("5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1".to_owned(), 5, 1004658);
}

#[test]
fn move_gen_perft_17() {
    move_gen_perft_test("4k3/1P6/8/8/8/8/K7/8 w - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn move_gen_perft_18() {
    move_gen_perft_test("8/k7/8/8/8/8/1p6/4K3 b - - 0 1".to_owned(), 6, 217342);
}

#[test]
fn move_gen_perft_19() {
    move_gen_perft_test("8/P1k5/K7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn move_gen_perft_20() {
    move_gen_perft_test("8/8/8/8/8/k7/p1K5/8 b - - 0 1".to_owned(), 6, 92683);
}

#[test]
fn move_gen_perft_21() {
    move_gen_perft_test("K1k5/8/P7/8/8/8/8/8 w - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn move_gen_perft_22() {
    move_gen_perft_test("8/8/8/8/8/p7/8/k1K5 b - - 0 1".to_owned(), 6, 2217);
}

#[test]
fn move_gen_perft_23() {
    move_gen_perft_test("8/k1P5/8/1K6/8/8/8/8 w - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn move_gen_perft_24() {
    move_gen_perft_test("8/8/8/8/1k6/8/K1p5/8 b - - 0 1".to_owned(), 7, 567584);
}

#[test]
fn move_gen_perft_25() {
    move_gen_perft_test("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1".to_owned(), 4, 23527);
}

#[test]
fn move_gen_perft_26() {
    move_gen_perft_test("8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1".to_owned(), 4, 23527);
}

#[test]
fn move_gen_issue_15() {
    let board =
        BoardBuilder::from_str("rnbqkbnr/ppp2pp1/4p3/3N4/3PpPp1/8/PPP3PP/R1B1KBNR b KQkq f3 0 1")
            .unwrap()
            .try_into()
            .unwrap();
    let _ = MoveGen::new_legal(&board);
}

#[cfg(test)]
fn move_of(m: &str) -> Move {
    let promo = if m.len() > 4 {
        Some(match m.as_bytes()[4] {
            b'q' => Queen,
            b'r' => Rook,
            b'b' => Bishop,
            b'n' => Knight,
            _ => panic!("unrecognized uci move: {}", m),
        })
    } else {
        None
    };
    Move::new(
        Square::from_str(&m[..2]).unwrap(),
        Square::from_str(&m[2..4]).unwrap(),
        promo,
    )
}

#[test]
fn test_masked_move_gen() {
    let board =
        SubBoard::from_str("r1bqkb1r/pp3ppp/5n2/2ppn1N1/4pP2/1BN1P3/PPPP2PP/R1BQ1RK1 w kq - 0 9")
            .unwrap();

    let mut capture_moves = MoveGen::new_legal(&board);
    let targets = *board.occupied_co(!board.side_to_move());
    capture_moves.set_iterator_mask(targets);

    let expected = vec![
        move_of("f4e5"),
        move_of("b3d5"),
        move_of("g5e4"),
        move_of("g5f7"),
        move_of("g5h7"),
        move_of("c3e4"),
        move_of("c3d5"),
    ];

    assert_eq!(
        capture_moves.collect::<HashSet<_>>(),
        expected.into_iter().collect()
    );
}
