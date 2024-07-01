use timecat::*;

fn check_evaluation(board: &mut Board, depth: u8) -> std::result::Result<(), Vec<ValidOrNullMove>> {
    if depth == 0 {
        return Ok(());
    }
    for valid_or_null_move in board.generate_legal_moves() {
        board.push_unchecked(valid_or_null_move);
        let sub_board = board.get_sub_board().to_owned();
        if board
            .get_evaluator_mut()
            .get_model_mut()
            .update_model_and_evaluate(&sub_board)
            != Evaluator::slow_evaluate_only_nnue(&sub_board)
        {
            return Err(board.get_all_moves());
        }
        check_evaluation(board, depth - 1)?;
        board.pop();
    }
    Ok(())
}

macro_rules! test_model_updated_correctly {
    ($func_name: ident, $fen: expr, $depth: expr) => {
        #[test]
        fn $func_name() {
            let mut board = Board::from_fen($fen).unwrap();
            if let Err(variation) = check_evaluation(&mut board, $depth) {
                panic!(
                    "Incorrect evaluation at position {} with starting fen {} and moves {}",
                    board.get_fen(),
                    $fen,
                    Board::variation_san(&Board::from_fen($fen).unwrap(), variation)
                );
            }
        }
    };
}

test_model_updated_correctly!(model_accumulator_update_test_1, STARTING_POSITION_FEN, 4);
test_model_updated_correctly!(
    model_accumulator_update_test_2,
    "2kr3r/pp1bbppp/2np1n2/2P1p1q1/2B1P3/2N2N2/PBPP1PPP/R2QR1K1 w - - 8 10",
    3
);
test_model_updated_correctly!(
    model_accumulator_update_test_3,
    "2kr3r/pp2nppp/3pB3/2P1p1b1/4R3/8/PBPP1PPP/R2Q2K1 b - - 0 14",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_4,
    "2k3r1/pp4Bp/4R3/2Pr2b1/8/8/P1P2PPP/6K1 b - - 0 21",
    3
);
test_model_updated_correctly!(
    model_accumulator_update_test_5,
    "8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_6,
    "8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_7,
    "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_8,
    "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_9,
    "5k2/8/8/8/8/8/8/4K2R w K - 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_10,
    "r3k3/8/8/8/8/8/8/3K4 b q - 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_11,
    "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_12,
    "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_13,
    "8/8/8/8/8/k7/p1K5/8 b - - 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_14,
    "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1",
    4
);
test_model_updated_correctly!(
    model_accumulator_update_test_kiwipete,
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    3
);
