use timecat::*;

macro_rules! test_draws_and_material_related_functions {
    ($name:ident, $func:ident, $array:expr,) => {
        #[test]
        fn $name() {
            let mut board = Board::default();
            for (fen, expected_return) in $array {
                for modified_fen in [fen.to_string(), flip_board_fen(fen).unwrap()] {
                    board.set_fen(&modified_fen).unwrap();
                    let returned_value = board.$func();
                    assert_eq!(
                        returned_value,
                        expected_return,
                        "{} returned {returned_value} in position {modified_fen}",
                        stringify!($func),
                    );
                }
            }
        }
    };
}

#[rustfmt::skip]
test_draws_and_material_related_functions!(
    board_has_insufficient_material,
    is_insufficient_material,
    [
        ("8/b7/3k4/6b1/8/2K5/5B2/8 w - - 0 1", true),
        ("8/8/3k4/1b6/8/2K5/5B2/8 w - - 0 1", false),
        ("8/8/3k4/1b6/B7/2K5/5B2/8 w - - 0 1", false),
        ("8/b7/3k4/6b1/8/2K3N1/5B2/8 w - - 0 1", false),
        ("8/8/3k4/6b1/8/2K3N1/5B2/8 w - - 0 1", false),
        ("8/b7/3k4/8/8/2K5/5B2/8 w - - 0 1", true),
        ("8/8/3k4/8/8/2K5/5B2/8 w - - 0 1", true),
        ("8/8/3k4/2b5/8/2K5/8/8 w - - 0 1", true),
        ("8/8/3k2n1/8/8/2K5/Q7/8 w - - 0 1", false),
        ("8/8/3k4/8/8/2K2N2/8/8 w - - 0 1", true),
    ],
);

#[rustfmt::skip]
test_draws_and_material_related_functions!(
    board_has_only_same_colored_bishop,
    has_only_same_colored_bishop,
    [
        ("8/b7/3k4/6b1/8/2K5/5B2/8 w - - 0 1", true),
        ("8/8/3k4/1b6/8/2K5/5B2/8 w - - 0 1", false),
        ("8/8/3k4/1b6/B7/2K5/5B2/8 w - - 0 1", false),
        ("8/b7/3k4/6b1/8/2K3N1/5B2/8 w - - 0 1", false),
        ("8/8/3k4/6b1/8/2K3N1/5B2/8 w - - 0 1", false),
        ("8/b7/3k4/8/8/2K5/5B2/8 w - - 0 1", true),
        ("8/8/3k4/8/8/2K5/5B2/8 w - - 0 1", true),
        ("8/8/3k4/2b5/8/2K5/8/8 w - - 0 1", true),
        ("8/8/3k2n1/8/8/2K5/Q7/8 w - - 0 1", false),
        ("8/8/3k4/8/8/2K2N2/8/8 w - - 0 1", false),
    ],
);

macro_rules! test_repetition_and_checkmate {
    ($func:ident, $array:expr,) => {
        paste! {
            #[test]
            fn [<move_$func>]() {
                for (fen, moves, move_, returned_value) in $array {
                    let mut board = Board::from_fen(fen).expect(&format!("Failed to set board FEN {fen}"));
                    board.push_sans(moves).expect(&format!("Failed to push sans {moves:?} in position {board}"));
                    assert_eq!(
                        board.$func(Move::from_san(&board, move_).expect(&format!("Failed to parse san {move_} in position {board}"))),
                        returned_value,
                        "Returned {returned_value} in position {fen} with moves {moves} and move {move_}"
                    );
                }
            }
        }
    }
}

#[rustfmt::skip]
test_repetition_and_checkmate!(
    gives_repetition,
    [
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2", "Ke7", true),
        ("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36", "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3 Ke5 Kd2 Kd5 Ke3", "Ke5", true),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7", "Ke1", false),
    ],
);

#[rustfmt::skip]
test_repetition_and_checkmate!(
    gives_threefold_repetition,
    [
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7 Ke1 Ke8 Ke2", "Ke7", true),
        ("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36", "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3 Ke5 Kd2 Kd5 Ke3 Ke5 Kd2 Kf6 Ke3", "Ke5", true),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7", "Ke1", false),
    ],
);

#[rustfmt::skip]
test_repetition_and_checkmate!(
    gives_claimable_threefold_repetition,
    [
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7 Ke1 Ke8", "Ke2", true),
        ("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36", "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3 Ke5 Kd2 Kd5 Ke3 Ke5 Kd2 Kf6", "Ke3", true),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2", "Ke7", false),
    ],
);

#[rustfmt::skip]
test_repetition_and_checkmate!(
    gives_check,
    [
        (STARTING_POSITION_FEN, "e4 b5 Qf3 e6", "Qxf7+", true),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2", "Ke7", false),
    ],
);

#[rustfmt::skip]
test_repetition_and_checkmate!(
    gives_checkmate,
    [
        (STARTING_POSITION_FEN, "e4 e5 Bc4 Nc6 Qh5 Nf6", "Qxf7#", true),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2", "Ke7", false),
    ],
);

#[test]
fn set_and_get_board_fen() {
    let mut board = Board::default();
    let fens = [
        STARTING_POSITION_FEN,
        "Bk3n2/8/P1p1n3/2b5/1B2p3/5p2/1KP3pP/1Q1r4 w - - 2 5",
        "q7/2R3K1/5n2/PNR5/3pr3/2P1P2k/1p2p3/5b1B w - - 30 50",
        "8/r7/p4K2/1kPp2P1/2p1P1pb/1P1b2p1/3p4/1r6 w - - 22 51",
        "5k2/p2P3K/Q7/4p3/3P2bp/2rN4/p1np2qp/8 w - - 11 23",
        "4R3/4P2P/5P2/p1p2Bp1/1pP5/2p5/P6B/2k2K1Q w - - 0 1",
        "rnbqkbnr/pppp2pp/4p3/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        "rnbqkbnr/pp1pppp1/8/1Pp4p/8/8/P1PPPPPP/RNBQKBNR w KQkq c6 0 3",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w K - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Q - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w k - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w q - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQk - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Qkq - 0 1",
    ];

    for fen in fens {
        board.set_fen(fen).unwrap();
        assert_eq!(board.get_fen(), fen);
    }
}

#[test]
fn specified_color_has_insufficient_material() {
    let mut board = Board::default();
    let fens = vec![
        ("8/2n2k2/8/8/R7/2K5/7Q/8 w - - 0 1", White, false),
        ("8/2n2k2/8/8/R7/2K5/7Q/8 w - - 0 1", Black, true),
    ];
    for (fen, color, expected_return) in fens {
        for (modified_fen, modified_color) in [
            (fen.to_string(), color),
            (flip_board_fen(fen).unwrap(), !color),
        ] {
            board.set_fen(&modified_fen).unwrap();
            let returned_value = board.has_insufficient_material(modified_color);
            assert_eq!(
                returned_value, expected_return,
                "Returned {returned_value} in position {modified_fen}"
            );
        }
    }
}

#[rustfmt::skip]
#[test]
fn test_passed_pawn_detection() {
    let moves = vec![
        ("8/5k1p/1p6/8/2K5/8/4P3/8 w - - 0 1", vec![E2, B6, H7]),
        ("8/5k1p/1p6/8/2K5/8/P3P3/8 w - - 0 1", vec![B6, H7]),
        ("8/5k1p/1p6/5P2/2K5/8/P3P3/8 w - - 0 1", vec![F5, B6, H7]),
        ("8/4nkNp/1p6/5P2/2K5/8/P3P3/8 w - - 0 1", vec![F5, B6, H7]),
        ("8/4nkNp/1p6/5P2/2K5/8/P3P3/8 w - - 0 1", vec![F5, B6, H7]),
        ("8/4nkNp/1p6/5P2/2K3p1/8/P3P3/8 w - - 0 1", vec![F5, B6, H4, H7]),
        ("8/5k1p/1p6/8/2K5/8/4P3/8 b - - 0 1", vec![E2, B6, H7]),
        ("8/5k1p/1p6/8/2K5/8/P3P3/8 b - - 0 1", vec![B6, H7]),
        ("8/5k1p/1p6/5P2/2K5/8/P3P3/8 b - - 0 1", vec![F5, B6, H7]),
        ("8/4nkNp/1p6/5P2/2K5/8/P3P3/8 b - - 0 1", vec![F5, B6, H7]),
        ("8/4nkNp/1p6/5P2/2K5/8/P3P3/8 b - - 0 1", vec![F5, B6, H7]),
        ("8/4nkNp/1p6/5P2/2K3p1/8/P3P3/8 b - - 0 1", vec![F5, B6, H4, H7]),
    ];
    for (fen, squares_vec) in moves {
        let board = Board::from_fen(fen).unwrap();
        for (modified_board, mut expected_value) in [
            (
                {
                    let mut board_clone = board.clone();
                    board_clone.flip_vertical();
                    board_clone
                },
                squares_vec
                    .iter()
                    .copied()
                    .map(Square::horizontal_mirror)
                    .collect_vec(),
            ),
            (board, squares_vec),
        ] {
            expected_value.sort_by_key(|square| square.to_int());
            let mut expected_value = ALL_SQUARES
                .into_iter()
                .filter(|&square| modified_board.is_passed_pawn(square))
                .collect_vec();
            expected_value.sort_by_key(|square| square.to_int());
            assert_eq!(
                expected_value,
                expected_value,
                "Got all En Passant squares as {expected_value:?} in position {}\n{modified_board}",
                modified_board.get_fen(),
            );
        }
    }
}

#[test]
fn move_is_en_passant() {
    let mut board = Board::default();
    let moves = vec![
        ("e4 c5 e5 f5", Move::from_str("e5f6").unwrap(), true),
        ("e4 g5 e5 g4 h4", Move::from_str("g4h3").unwrap(), true),
        ("e4 f5", Move::from_str("e4f5").unwrap(), false),
    ];
    for (moves_str, valid_or_null_move, expected_return) in moves {
        board.set_fen(STARTING_POSITION_FEN).unwrap();
        board.push_sans(moves_str).unwrap();
        let returned_value = board.is_en_passant(valid_or_null_move);
        assert_eq!(
            returned_value, expected_return,
            "Returned {returned_value} with moves {moves_str} for move {valid_or_null_move}"
        );
    }
}

macro_rules! board_material_check_command {
    ($command: expr; $board: expr; $white_material_score: expr; $black_material_score: expr) => {
        $command;
        assert_eq!(
            $board.get_white_material_score(),
            $white_material_score,
            "Expected {} found {} for White. Failed after running the command:\n{}",
            $white_material_score,
            $board.get_white_material_score(),
            stringify!($command)
        );
        assert_eq!(
            $board.get_black_material_score(),
            $black_material_score,
            "Expected {} found {} for Black. Failed after running the command:\n{}",
            $black_material_score,
            $board.get_black_material_score(),
            stringify!($command)
        );
    };
}

#[rustfmt::skip]
#[test]
fn test_board_material_score_track() {
    let mut board = Board::default();
    let mut white_material_score = INITIAL_MATERIAL_SCORE_ABS / 2;
    let mut black_material_score = INITIAL_MATERIAL_SCORE_ABS / 2;
    board_material_check_command!(board.push_san("e4").unwrap(); board; white_material_score; black_material_score);
    board_material_check_command!(board.push_san("d5").unwrap(); board; white_material_score; black_material_score);
    black_material_score -= Pawn.evaluate();
    board_material_check_command!(board.push_san("exd5").unwrap(); board; white_material_score; black_material_score);
    white_material_score -= Pawn.evaluate();
    board_material_check_command!(board.push_san("Qxd5").unwrap(); board; white_material_score; black_material_score);
    white_material_score += Pawn.evaluate();
    board_material_check_command!(board.pop(); board; white_material_score; black_material_score);
    black_material_score += Pawn.evaluate();
    board_material_check_command!(board.pop(); board; white_material_score; black_material_score);
}

// is_capture
// is_quiet
// is_zeroing
// get_en_passant_square
// has_legal_en_passant
// clean_castling_rights
// reduces_castling_rights
// is_irreversible
// ep_square
// is_castling
// is_double_pawn_push
