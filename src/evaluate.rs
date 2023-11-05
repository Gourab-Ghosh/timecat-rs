use super::*;
use nnue::StockfishNetwork;

pub struct Evaluator {
    stockfish_network: StockfishNetwork,
    score_cache: CacheTable<Score>,
}

impl Evaluator {
    pub fn print_info(&self) {
        println!("{}", format!("Evaluation Cache Table initialization complete with {} entries taking {} MB space.", self.score_cache.len(), self.score_cache.get_size()).colorize(INFO_MESSAGE_STYLE));
    }

    pub fn new() -> Self {
        Self {
            stockfish_network: StockfishNetwork::new(),
            score_cache: CacheTable::new(EVALUATOR_SIZE, 0),
        }
    }

    #[allow(unused_variables)]
    pub fn activate_nnue(&mut self, piece: Piece, color: Color, square: Square) {
        // self.stockfish_network.activate();
    }

    #[allow(unused_variables)]
    pub fn deactivate_nnue(&mut self, piece: Piece, color: Color, square: Square) {
        // self.stockfish_network.activate();
    }

    fn force_opponent_king_to_corner(
        &self,
        board: &Board,
        winning_side: Color,
        is_bishop_knight_endgame: bool,
    ) -> Score {
        let winning_side_king_square = board.get_king_square(winning_side);
        let losing_side_king_square = board.get_king_square(!winning_side);
        let mut least_distant_corner = Square::default();
        if is_bishop_knight_endgame {
            let is_light_squared_bishop =
                board.get_piece_mask(Bishop) & BB_LIGHT_SQUARES != BB_EMPTY;
            let least_distant_corners = if is_light_squared_bishop {
                [Square::A8, Square::H1]
            } else {
                [Square::A1, Square::H8]
            };
            least_distant_corner = *least_distant_corners
                .iter()
                .min_by_key(|&&corner_square| {
                    square_distance(corner_square, losing_side_king_square)
                })
                .unwrap();
        } else {
            for (bb, &corner_square) in BOARD_QUARTER_MASKS
                .iter()
                .zip([Square::A8, Square::H8, Square::A1, Square::H1].iter())
            {
                if bb & get_square_bb(losing_side_king_square) != BB_EMPTY {
                    least_distant_corner = corner_square;
                    break;
                }
            }
        }
        let king_distance_score =
            7 - square_distance(winning_side_king_square, losing_side_king_square) as Score;
        let losing_king_rank_distance_score = 15
            - losing_side_king_square
                .get_rank()
                .to_index()
                .abs_diff(least_distant_corner.get_rank().to_index()) as Score;
        let losing_king_file_distance_score = 15
            - losing_side_king_square
                .get_file()
                .to_index()
                .abs_diff(least_distant_corner.get_file().to_index()) as Score;
        let losing_king_corner_score =
            losing_king_rank_distance_score.pow(2) + losing_king_file_distance_score.pow(2);
        let losing_king_opponent_pieces_score = board
            .occupied_co(winning_side)
            .map(|square| 7 - square_distance(square, losing_side_king_square))
            .sum::<u8>() as Score;
        10 * (10 * king_distance_score + 2 * losing_king_corner_score)
            + losing_king_opponent_pieces_score
    }

    fn force_passed_pawn_push(&self, board: &Board) -> Score {
        todo!("force_passed_pawn_push {}", board)
    }

    fn king_corner_forcing_evaluation(&self, board: &Board, material_score: Score) -> Score {
        let is_bishop_knight_endgame = board.get_num_pieces() == 4
            && material_score.abs() == evaluate_piece(Knight) + evaluate_piece(Bishop);
        let winning_side = if material_score.is_positive() {
            White
        } else {
            Black
        };
        let signum = material_score.signum();
        let king_forcing_score =
            self.force_opponent_king_to_corner(board, winning_side, is_bishop_knight_endgame);
        (50 * PAWN_VALUE + king_forcing_score / 2) * signum + 2 * material_score
    }

    pub fn is_easily_winning_position(board: &Board, material_score: Score) -> bool {
        if material_score.abs() > PAWN_VALUE + evaluate_piece(Bishop) {
            let white_occupied = board.occupied_co(White);
            let black_occupied = board.occupied_co(Black);
            let num_white_pieces = white_occupied.popcnt();
            let num_black_pieces = black_occupied.popcnt();
            let num_pieces = num_white_pieces + num_black_pieces;
            if num_pieces < 5 {
                if num_white_pieces == 2 && num_black_pieces == 2 {
                    let non_king_white_piece = board
                        .piece_at(
                            (white_occupied & !board.get_piece_mask(King))
                                .next()
                                .unwrap(),
                        )
                        .unwrap();
                    let non_king_black_piece = board
                        .piece_at(
                            (black_occupied & !board.get_piece_mask(King))
                                .next()
                                .unwrap(),
                        )
                        .unwrap();
                    let mut non_king_pieces = [non_king_white_piece, non_king_black_piece];
                    non_king_pieces.sort();
                    if non_king_pieces == [Pawn, Rook] {
                        return false;
                    }
                }
                for (&bb, &num_pieces) in [white_occupied, black_occupied]
                    .iter()
                    .zip([num_white_pieces, num_black_pieces].iter())
                {
                    if num_pieces == 3 {
                        let non_king_pieces: (Piece, Piece) = (bb & !board.get_piece_mask(King))
                            .map(|s| board.piece_at(s).unwrap())
                            .collect_tuple()
                            .unwrap();
                        if non_king_pieces == (Knight, Knight) {
                            return false;
                        }
                    }
                }
                return true;
            }
            if num_white_pieces == 1 || num_black_pieces == 1 {
                return true;
            }
        }
        false
    }

    pub fn evaluate_raw(&self, board: &Board) -> Score {
        let material_score = board.get_material_score();
        if Self::is_easily_winning_position(board, material_score) {
            return self.king_corner_forcing_evaluation(board, material_score);
        }
        let mut nnue_eval = self.stockfish_network.eval(&board.get_sub_board());
        if nnue_eval.abs() > WINNING_SCORE_THRESHOLD {
            let multiplier = match_interpolate!(
                0,
                1,
                WINNING_SCORE_THRESHOLD,
                35 * PAWN_VALUE,
                nnue_eval.abs()
            );
            nnue_eval += (multiplier * (material_score as f64)).round() as Score;
            let losing_side = if nnue_eval.is_positive() {
                Black
            } else {
                White
            };
            nnue_eval += nnue_eval.signum()
                * match_interpolate!(
                    0,
                    5.0 * multiplier,
                    MAX_MATERIAL_SCORE,
                    0,
                    board.get_masked_material_score_abs(board.occupied_co(losing_side))
                )
                .round() as Score
                * PAWN_VALUE;
        }
        nnue_eval
    }

    fn hashed_evaluate(&self, board: &Board) -> Score {
        let hash = board.hash();
        if let Some(score) = self.score_cache.get(hash) {
            return score;
        }
        let score = self.evaluate_raw(board);
        self.score_cache.add(hash, score);
        score
    }

    pub fn evaluate(&self, board: &Board) -> Score {
        self.hashed_evaluate(board)
    }

    pub fn reset_variables(&self) {
        self.score_cache.reset_variables();
    }

    pub fn clear(&self) {
        self.score_cache.clear();
    }

    pub fn set_size(&self, size: CacheTableSize) {
        self.score_cache.set_size(size);
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
