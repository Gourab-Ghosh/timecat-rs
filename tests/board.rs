use timecat::*;

#[test]
fn board_has_insufficient_material() {
    let mut board = Board::default();
    let fens = [
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
    ];
    for (fen, expected_return) in fens {
        board.set_fen(fen).unwrap();
        let returned_value = board.is_insufficient_material();
        assert_eq!(
            returned_value, expected_return,
            "Returned {returned_value} in position {fen}"
        );
    }
}

#[test]
fn board_has_only_same_colored_bishop() {
    let mut board = Board::default();
    let fens = [
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
    ];
    for (fen, expected_return) in fens {
        board.set_fen(fen).unwrap();
        let returned_value = board.has_only_same_colored_bishop();
        assert_eq!(
            returned_value, expected_return,
            "Returned {returned_value} in position {fen}"
        );
    }
}

macro_rules! test_repetition {
    ($func:ident, $array:expr) => {
        paste! {
            #[test]
            fn [<move_$func>]() {
                for (fen, moves, move_, returned_value) in $array {
                    let mut board = Board::from_fen(fen).unwrap();
                    board.push_sans(moves).unwrap();
                    assert_eq!(
                        board.$func(board.parse_san(move_).unwrap().unwrap()),
                        returned_value,
                        "Returned {returned_value} in position {fen} with moves {moves} and move {move_}"
                    );
                }
            }
        }
    }
}

#[rustfmt::skip]
test_repetition!(
    gives_repetition,
    [
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7 Ke1", "Ke8", true),
        ("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36", "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3", "Ke5", true,),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7", "Ke1", false),
    ]
);

#[rustfmt::skip]
test_repetition!(
    gives_threefold_repetition,
    [
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7 Ke1", "Ke8", true),
        ("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36", "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3 Ke5 Kd2 Kd5 Ke3 Ke5 Kd2 Kf6", "Ke3", true,),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7", "Ke1", false),
    ]
);

#[rustfmt::skip]
test_repetition!(
    gives_claimable_threefold_repetition,
    [
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7 Ke1 Ke8 Ke2 Ke7", "Ke1", true),
        ("8/8/8/p1ppkPp1/P1p3P1/2P1K3/2P5/8 b - - 0 36", "d4+ cxd4+ cxd4+ Kd2 Kf6 c3 d3 Ke3 Ke5 Kd2 Kd5 Ke3 Ke5 Kd2", "Kf6", true,),
        (STARTING_POSITION_FEN, "e4 e5 Ke2 Ke7 Ke1 Ke8 Ke2", "Ke7", false),
    ]
);
