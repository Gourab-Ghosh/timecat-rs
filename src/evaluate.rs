use super::*;
use nnue::StockfishNetwork;

pub struct Evaluator {
    stockfish_network: StockfishNetwork,
    cache: CacheTable<Score>,
}

impl Evaluator {
    pub fn new(print: bool) -> Self {
        if print {
            println_info(
                "Evaluator Cache size",
                format!(
                    "{} MB",
                    EVALUATOR_SIZE.to_cache_table_memory_size::<Score>()
                ),
            );
        }
        Self {
            stockfish_network: StockfishNetwork::new(),
            cache: CacheTable::new(EVALUATOR_SIZE.to_cache_table_size::<Score>(), 0),
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

    #[allow(unused_variables)]
    pub fn backup(&mut self) {}

    #[allow(unused_variables)]
    pub fn restore(&mut self) {}

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
        king_distance_score + losing_king_corner_score
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
        (70 * PAWN_VALUE + king_forcing_score) * signum + 2 * material_score
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
                            .sorted()
                            .collect_tuple()
                            .unwrap();
                        if [(Knight, Knight), (Knight, Bishop)].contains(&non_king_pieces) {
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
        // let material_score = material_score as f64 / MAX_MATERIAL_SCORE as f64 * PAWN_VALUE as f64;
        // let mut score = self.stockfish_network.eval(&board.get_sub_board());
        // let winning_side = if score.is_positive() { White } else { Black };
        // let king_forcing_score = score.signum() as f64
        //     * self.force_opponent_king_to_corner(board, winning_side) as f64
        //     / 13.2;
        // let mut endgame_evaluation = 10.0 * king_forcing_score + 5.0 * material_score;
        // endgame_evaluation *= match_interpolate!(
        //     0,
        //     1,
        //     INITIAL_MATERIAL_SCORE_ABS,
        //     0,
        //     board.get_material_score_abs()
        // );
        // score += endgame_evaluation.round() as Score;
        // score
        // self.stockfish_network.eval(&board.get_sub_board()) + material_score / (MAX_MATERIAL_SCORE / 20)
        self.stockfish_network.eval(&board.get_sub_board())
    }

    fn hashed_evaluate(&mut self, board: &Board) -> Score {
        let hash = board.hash();
        if let Some(score) = self.cache.get(hash) {
            return score;
        }
        let score = self.evaluate_raw(board);
        self.cache.add(hash, score);
        score
    }

    pub fn evaluate(&mut self, board: &Board) -> Score {
        self.hashed_evaluate(board)
    }

    pub fn evaluate_flipped(&mut self, board: &Board) -> Score {
        board.score_flipped(self.evaluate(board))
    }

    pub fn reset_variables(&mut self) {
        self.cache.reset_variables();
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new(true)
    }
}
