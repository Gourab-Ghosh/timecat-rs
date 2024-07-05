use std::collections::HashSet;
use timecat::*;

fn move_generator_perft_test(fen: &str, depth: usize, expected_result: usize) {
    let sub_board = SubBoard::from_str(fen).unwrap();
    let result = MoveGenerator::perft_test(&sub_board, depth);
    assert_eq!(
        result, expected_result,
        "Expected result {expected_result} but got {result} in position {fen}"
    );
    let result = MoveGenerator::perft_test_piecewise(&sub_board, depth);
    assert_eq!(
        result, expected_result,
        "Expected result {expected_result} but got {result} in position {fen}"
    );
}

macro_rules! generate_move_generator_functions {
    ($func_name: ident, $fen: expr, $depth: expr, $expected_result: expr) => {
        #[test]
        fn $func_name() {
            move_generator_perft_test($fen, $depth, $expected_result);
        }
    };

    ($func_name: ident, $fen: expr, $depth: expr, $expected_result: expr,) => {
        generate_move_generator_functions!($func_name, $fen, $depth, $expected_result);
    };
}

generate_move_generator_functions!(
    move_generator_perft_kiwipete,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    5,
    193690690,
);
generate_move_generator_functions!(
    move_generator_perft_1,
    "8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1",
    8,
    76172334,
);
generate_move_generator_functions!(
    move_generator_perft_2,
    "8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1",
    8,
    76172334,
);
generate_move_generator_functions!(
    move_generator_perft_3,
    "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1",
    8,
    144302151,
);
generate_move_generator_functions!(
    move_generator_perft_4,
    "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1",
    8,
    144302151,
);
generate_move_generator_functions!(
    move_generator_perft_5,
    "5k2/8/8/8/8/8/8/4K2R w K - 0 1",
    8,
    73450134,
);
generate_move_generator_functions!(
    move_generator_perft_6,
    "4k2r/8/8/8/8/8/8/5K2 b k - 0 1",
    8,
    73450134,
);
generate_move_generator_functions!(
    move_generator_perft_7,
    "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1",
    8,
    91628014,
);
generate_move_generator_functions!(
    move_generator_perft_8,
    "r3k3/8/8/8/8/8/8/3K4 b q - 0 1",
    8,
    91628014,
);
generate_move_generator_functions!(
    move_generator_perft_9,
    "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1",
    5,
    31912360,
);
generate_move_generator_functions!(
    move_generator_perft_10,
    "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1",
    5,
    31912360,
);
generate_move_generator_functions!(
    move_generator_perft_11,
    "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1",
    5,
    58773923,
);
generate_move_generator_functions!(
    move_generator_perft_12,
    "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1",
    5,
    58773923,
);
generate_move_generator_functions!(
    move_generator_perft_13,
    "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1",
    7,
    60651209,
);
generate_move_generator_functions!(
    move_generator_perft_14,
    "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1",
    7,
    60651209,
);
generate_move_generator_functions!(
    move_generator_perft_15,
    "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1",
    7,
    197013195,
);
generate_move_generator_functions!(
    move_generator_perft_16,
    "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1",
    7,
    197013195,
);
generate_move_generator_functions!(
    move_generator_perft_17,
    "4k3/1P6/8/8/8/8/K7/8 w - - 0 1",
    8,
    20625698,
);
generate_move_generator_functions!(
    move_generator_perft_18,
    "8/k7/8/8/8/8/1p6/4K3 b - - 0 1",
    8,
    20625698,
);
generate_move_generator_functions!(
    move_generator_perft_19,
    "8/P1k5/K7/8/8/8/8/8 w - - 0 1",
    8,
    8110830
);
generate_move_generator_functions!(
    move_generator_perft_20,
    "8/8/8/8/8/k7/p1K5/8 b - - 0 1",
    8,
    8110830
);
generate_move_generator_functions!(
    move_generator_perft_21,
    "K1k5/8/P7/8/8/8/8/8 w - - 0 1",
    11,
    85822924
);
generate_move_generator_functions!(
    move_generator_perft_22,
    "8/8/8/8/8/p7/8/k1K5 b - - 0 1",
    11,
    85822924
);
generate_move_generator_functions!(
    move_generator_perft_23,
    "8/k1P5/8/1K6/8/8/8/8 w - - 0 1",
    9,
    37109897,
);
generate_move_generator_functions!(
    move_generator_perft_24,
    "8/8/8/8/1k6/8/K1p5/8 b - - 0 1",
    9,
    37109897,
);
generate_move_generator_functions!(
    move_generator_perft_25,
    "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1",
    7,
    104644508,
);
generate_move_generator_functions!(
    move_generator_perft_26,
    "8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1",
    7,
    104644508,
);

#[test]
fn move_generator_issue_15() {
    let sub_board = SubBoardBuilder::from_str(
        "rnbqkbnr/ppp2pp1/4p3/3N4/3PpPp1/8/PPP3PP/R1B1KBNR b KQkq f3 0 1",
    )
    .unwrap()
    .try_into()
    .unwrap();
    let _ = MoveGenerator::new_legal(&sub_board);
}

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
    .unwrap()
}

#[test]
fn test_masked_move_generator() {
    let sub_board =
        SubBoard::from_str("r1bqkb1r/pp3ppp/5n2/2ppn1N1/4pP2/1BN1P3/PPPP2PP/R1BQ1RK1 w kq - 0 9")
            .unwrap();

    let attackers = sub_board.get_piece_mask(Knight);
    let targets = sub_board.occupied_co(!sub_board.turn());
    let masked_moves = sub_board.generate_masked_legal_moves(attackers, targets);

    let expected = vec![
        move_of("g5e4"),
        move_of("g5f7"),
        move_of("g5h7"),
        move_of("c3e4"),
        move_of("c3d5"),
    ];

    assert_eq!(
        masked_moves.collect::<HashSet<_>>(),
        expected.into_iter().collect()
    );
}
