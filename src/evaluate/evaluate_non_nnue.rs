use super::*;

const CONTROL_CENTER_BONUS: Score = 50;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Default, Debug)]
pub struct EvaluatorNonNNUE;

impl EvaluatorNonNNUE {
    pub fn new(_: &MinimumBoard) -> Self {
        Self
    }

    pub fn slow_evaluate(minimum_board: &MinimumBoard) -> Score {
        Self::new(minimum_board).evaluate(minimum_board)
    }

    fn evaluate_raw(minimum_board: &MinimumBoard) -> Score {
        let material_score = minimum_board.get_material_score();
        let mut score = material_score;

        // Consider additional heuristics
        let mut mobility_score = 0;
        let mut king_safety_score = 0;
        // let mut pawn_structure_score = 0;
        let mut center_control_score = 0;
        // let mut piece_activity_score = 0;
        // let mut threat_score = 0;

        for (piece, square) in minimum_board.iter() {
            let alpha =
                minimum_board.get_material_score_abs() as f64 / INITIAL_MATERIAL_SCORE_ABS as f64;
            let psqt_score = get_psqt_score(piece, square, alpha);
            score += if piece.get_color() == White {
                psqt_score
            } else {
                -psqt_score
            } as Score;

            // Calculate mobility (number of legal moves)
            let mobility = minimum_board
                .generate_masked_legal_moves(square.to_bitboard(), BB_ALL)
                .count() as Score;
            mobility_score += if piece.get_color() == White {
                mobility
            } else {
                -mobility
            };

            // Calculate king safety
            if piece.get_piece_type() == King {
                let king_safety = Self::evaluate_king_safety(minimum_board, square);
                king_safety_score += if piece.get_color() == White {
                    king_safety
                } else {
                    -king_safety
                };
            }

            // // Calculate pawn structure
            // if piece.get_piece_type() == Pawn {
            //     let pawn_structure = Self::evaluate_pawn_structure(minimum_board, square);
            //     pawn_structure_score += if piece.get_color() == White {
            //         pawn_structure
            //     } else {
            //         -pawn_structure
            //     };
            // }

            // Calculate control of the center
            if BB_CENTER.contains(square) {
                center_control_score += if piece.get_color() == White {
                    CONTROL_CENTER_BONUS
                } else {
                    -CONTROL_CENTER_BONUS
                };
            }

            // // Calculate piece activity
            // let piece_activity = Self::evaluate_piece_activity(minimum_board, piece, square);
            // piece_activity_score += if piece.get_color() == White {
            //     piece_activity
            // } else {
            //     -piece_activity
            // };

            // // Calculate threats
            // let threats = Self::evaluate_threats(minimum_board, piece, square);
            // threat_score += if piece.get_color() == White {
            //     threats
            // } else {
            //     -threats
            // };
        }

        // Combine all heuristics into the final score
        score += mobility_score;
        score += king_safety_score;
        // score += pawn_structure_score;
        score += center_control_score;
        // score += piece_activity_score;
        // score += threat_score;

        score
    }

    // Enhanced King Safety
    fn evaluate_king_safety(minimum_board: &MinimumBoard, king_square: Square) -> Score {
        let king_color = minimum_board.get_piece_at(king_square).unwrap().get_color();
        let mut safety_score = 0;

        // Evaluate pawn shield (simplified, assuming a standard chessboard)
        let Some(forward_king_square) = king_square.forward(king_color) else {
            return 0;
        };
        let pawn_shield_squares = [
            forward_king_square.left(),
            Some(forward_king_square),
            forward_king_square.right(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();

        for &shield_square in &pawn_shield_squares {
            if let Some(pawn) = minimum_board.get_piece_at(shield_square) {
                if pawn.get_piece_type() == Pawn && pawn.get_color() == king_color {
                    safety_score += 10; // Example value for a pawn in the shield
                }
            }
        }

        // Penalize open files or no pawn shield
        if pawn_shield_squares
            .iter()
            .all(|&sq| minimum_board.get_piece_at(sq).is_none())
        {
            safety_score -= 50; // Example penalty for an exposed king
        }

        // // Penalize for enemy pieces attacking nearby squares
        // let enemy_color = !king_color;
        // let nearby_squares = king_square.neighbors();
        // for &square in &nearby_squares {
        //     if minimum_board.is_square_attacked(square, enemy_color) {
        //         safety_score -= 20; // Example penalty for each attack on nearby squares
        //     }
        // }

        safety_score
    }

    // Enhanced Pawn Structure
    fn evaluate_pawn_structure(minimum_board: &MinimumBoard, pawn_square: Square) -> Score {
        let mut structure_score = 0;
        let pawn = minimum_board.get_piece_at(pawn_square).unwrap();
        let pawn_color = pawn.get_color();

        // Evaluate isolated pawns (no friendly pawns on adjacent files)
        let file = pawn_square.get_file();
        let adjacent_files = [file.left(), file.right()];

        let isolated = adjacent_files.iter().flatten().all(|&adj_file| {
            ALL_RANKS.iter().all(|&rank| {
                let sq = Square::from_rank_and_file(rank, adj_file);
                minimum_board.get_piece_at(sq).map_or(true, |p| {
                    p.get_piece_type() != Pawn || p.get_color() != pawn_color
                })
            })
        });

        if isolated {
            structure_score -= 20; // Example penalty for isolated pawns
        }

        // Evaluate doubled pawns (multiple pawns of the same color on the same file)
        let file_pawns = ALL_RANKS
            .iter()
            .filter(|&&rank| {
                let sq = Square::from_rank_and_file(rank, file);
                minimum_board.get_piece_at(sq).map_or(false, |p| {
                    p.get_piece_type() == Pawn && p.get_color() == pawn_color
                })
            })
            .count();

        if file_pawns > 1 {
            structure_score -= 10 * (file_pawns as Score - 1) as Score; // Example penalty for each doubled pawn
        }

        // Evaluate passed pawns (no opposing pawns blocking or attacking the path to promotion)
        if minimum_board.is_passed_pawn(pawn_square) {
            structure_score += 30; // Example bonus for passed pawns
        }

        // // Evaluate backward pawns (no friendly pawns protecting from behind)
        // let backward = adjacent_files.iter().flatten().all(|&adj_file| {
        //     (0..pawn_square.get_rank()).all(|rank| {
        //         let sq = Square::from_rank_and_file(rank, adj_file);
        //         minimum_board.get_piece_at(sq).map_or(true, |p| {
        //             p.get_piece_type() != Pawn || p.get_color() != pawn_color
        //         })
        //     })
        // });

        // if backward {
        //     structure_score -= 15; // Example penalty for backward pawns
        // }

        structure_score
    }

    // Enhanced Piece Activity
    fn evaluate_piece_activity(minimum_board: &MinimumBoard, _piece: Piece, square: Square) -> Score {
        let mut activity_score = 0;
        // let piece_color = piece.get_color();

        // // Simplified example: give a bonus for pieces controlling key squares
        // let key_squares = [D4, E4, D5, E5]; // Central squares
        // for &key_square in &key_squares {
        //     if minimum_board.is_square_attacked(key_square, piece_color) {
        //         activity_score += 5; // Example bonus for controlling a key square
        //     }
        // }

        // Penalize pieces trapped or blocked by own pawns
        let piece_mobility = minimum_board
            .generate_masked_legal_moves(square.to_bitboard(), BB_ALL)
            .count() as Score;
        if piece_mobility < 2 {
            activity_score -= 10; // Example penalty for low mobility
        }

        activity_score
    }

    // Evaluate threats
    fn evaluate_threats(minimum_board: &MinimumBoard, piece: Piece, square: Square) -> Score {
        let mut threat_score = 0;
        let piece_color = piece.get_color();

        // Simplified example: give a bonus for attacking opponent's pieces
        for move_ in minimum_board
            .generate_masked_legal_moves(square.to_bitboard(), minimum_board.occupied_co(!piece_color))
        {
            threat_score += match minimum_board.get_piece_type_at(move_.get_dest()).unwrap() {
                King => 0,
                piece_type => piece_type.evaluate(),
            }
        }

        threat_score
    }
}

impl PositionEvaluation for EvaluatorNonNNUE {
    fn evaluate(&mut self, minimum_board: &MinimumBoard) -> Score {
        let material_score = minimum_board.get_material_score();
        let mut score = material_score;
        for (piece, square) in minimum_board.iter() {
            let alpha =
                minimum_board.get_material_score_abs() as f64 / INITIAL_MATERIAL_SCORE_ABS as f64;
            let psqt_score = get_psqt_score(piece, square, alpha);
            score += if piece.get_color() == White {
                psqt_score
            } else {
                -psqt_score
            } as Score;
        }
        score
        // Self::evaluate_raw(minimum_board)
    }
}
