use super::*;

#[cfg(feature = "inbuilt_nnue")]
static HALFKP_MODEL_READER: LazyStatic<HalfKPModelReader> = LazyStatic::new(|| {
    let mut reader = std::io::Cursor::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/nnue_dir/nn.nnue"
    )));
    HalfKPModelReader::read(&mut reader)
        .map_err(|_| TimecatError::BadNNUEFile)
        .unwrap()
});

#[derive(Clone, Debug)]
pub struct Evaluator {
    model: HalfKPModel,
    score_cache: Arc<CacheTable<Score>>,
}

impl Evaluator {
    pub fn print_info(&self) {
        print_cache_table_info(
            "Evaluation Cache Table",
            self.score_cache.len(),
            self.score_cache.get_size(),
        );
    }

    #[cfg(feature = "inbuilt_nnue")]
    pub fn new(sub_board: &SubBoard) -> Self {
        Self {
            model: HALFKP_MODEL_READER.to_model(sub_board),
            score_cache: Arc::new(CacheTable::new(EVALUATOR_SIZE)),
        }
    }

    pub fn from_nnue_bytes(nnue_bytes: &[u8], sub_board: &SubBoard) -> Result<Self> {
        let mut reader = std::io::Cursor::new(nnue_bytes);
        let model = HalfKPModelReader::read(&mut reader)
            .map_err(|_| TimecatError::BadNNUEFile)?
            .to_model(sub_board);

        Ok(Self {
            model,
            score_cache: Arc::new(CacheTable::new(EVALUATOR_SIZE)),
        })
    }

    pub fn from_nnue_path(path: &str, sub_board: &SubBoard) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let model = HalfKPModelReader::read(&mut reader)
            .map_err(|_| TimecatError::BadNNUEFile)?
            .to_model(sub_board);

        Ok(Self {
            model,
            score_cache: Arc::new(CacheTable::new(EVALUATOR_SIZE)),
        })
    }

    #[inline]
    pub fn get_model(&self) -> &HalfKPModel {
        &self.model
    }

    #[inline]
    pub fn get_model_mut(&mut self) -> &mut HalfKPModel {
        &mut self.model
    }

    fn force_opponent_king_to_corner(
        sub_board: &SubBoard,
        winning_side: Color,
        is_bishop_knight_endgame: bool,
    ) -> Score {
        let winning_side_king_square = sub_board.get_king_square(winning_side);
        let losing_side_king_square = sub_board.get_king_square(!winning_side);
        let mut probable_least_distant_corner = None;
        if is_bishop_knight_endgame {
            let is_light_squared_bishop =
                !(sub_board.get_piece_mask(Bishop) & BB_LIGHT_SQUARES).is_empty();
            let least_distant_corners = if is_light_squared_bishop {
                [Square::A8, Square::H1]
            } else {
                [Square::A1, Square::H8]
            };
            probable_least_distant_corner = Some(
                *least_distant_corners
                    .iter()
                    .min_by_key(|&&corner_square| corner_square.distance(losing_side_king_square))
                    .unwrap(),
            );
        } else {
            for (bb, &corner_square) in BOARD_QUARTER_MASKS
                .iter()
                .zip([Square::A8, Square::H8, Square::A1, Square::H1].iter())
            {
                if bb.contains(losing_side_king_square) {
                    probable_least_distant_corner = Some(corner_square);
                    break;
                }
            }
        }
        let least_distant_corner = unsafe { probable_least_distant_corner.unwrap_unchecked() };
        let king_distance_score =
            7 - winning_side_king_square.distance(losing_side_king_square) as Score;
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
        let losing_king_opponent_pieces_score = sub_board
            .occupied_co(winning_side)
            .map(|square| 7 - square.distance(losing_side_king_square))
            .sum::<u8>() as Score;
        10 * (10 * king_distance_score + 2 * losing_king_corner_score)
            + losing_king_opponent_pieces_score
    }

    fn force_passed_pawn_push(sub_board: &SubBoard) -> Score {
        todo!("force_passed_pawn_push {}", sub_board)
    }

    fn king_corner_forcing_evaluation(sub_board: &SubBoard, material_score: Score) -> Score {
        let is_bishop_knight_endgame = sub_board.get_num_pieces() == 4
            && material_score.abs() == Knight.evaluate() + Bishop.evaluate();
        let winning_side = if material_score.is_positive() {
            White
        } else {
            Black
        };
        let signum = material_score.signum();
        let king_forcing_score =
            Self::force_opponent_king_to_corner(sub_board, winning_side, is_bishop_knight_endgame);
        (50 * PAWN_VALUE + king_forcing_score / 2) * signum + 2 * material_score
    }

    pub fn is_easily_winning_position(sub_board: &SubBoard, material_score: Score) -> bool {
        if material_score.abs() > PAWN_VALUE + Bishop.evaluate() {
            let white_occupied = sub_board.occupied_co(White);
            let black_occupied = sub_board.occupied_co(Black);
            let num_white_pieces = white_occupied.popcnt();
            let num_black_pieces = black_occupied.popcnt();
            let num_pieces = num_white_pieces + num_black_pieces;
            if num_pieces < 5 {
                if num_white_pieces == 2 && num_black_pieces == 2 {
                    let non_king_white_piece = sub_board
                        .piece_type_at(
                            (white_occupied & !sub_board.get_piece_mask(King))
                                .next()
                                .unwrap(),
                        )
                        .unwrap();
                    let non_king_black_piece = sub_board
                        .piece_type_at(
                            (black_occupied & !sub_board.get_piece_mask(King))
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
                        let non_king_pieces: (PieceType, PieceType) = (bb
                            & !sub_board.get_piece_mask(King))
                        .map(|s| sub_board.piece_type_at(s).unwrap())
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

    fn evaluate_raw(sub_board: &SubBoard, mut nnue_eval_func: impl FnMut() -> Score) -> Score {
        let knights_mask = sub_board.get_piece_mask(Knight);
        if sub_board.get_non_king_pieces_mask() == knights_mask && knights_mask.popcnt() < 3 {
            return 0;
        }
        let material_score = sub_board.get_material_score();
        if Self::is_easily_winning_position(sub_board, material_score) {
            return Self::king_corner_forcing_evaluation(sub_board, material_score);
        }
        let mut nnue_eval = nnue_eval_func();
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
                    sub_board.get_masked_material_score_abs(sub_board.occupied_co(losing_side))
                )
                .round() as Score
                * PAWN_VALUE;
        }
        nnue_eval
    }

    fn hashed_evaluate(&mut self, sub_board: &SubBoard) -> Score {
        let hash = sub_board.get_hash();
        if let Some(score) = self.score_cache.get(hash) {
            return score;
        }
        let score = Self::evaluate_raw(sub_board, || {
            self.model.update_model_and_evaluate(sub_board)
        });
        self.score_cache.add(hash, score);
        score
    }

    #[inline]
    pub(crate) fn evaluate(&mut self, sub_board: &SubBoard) -> Score {
        self.hashed_evaluate(sub_board)
    }

    #[cfg(feature = "inbuilt_nnue")]
    #[inline]
    pub fn slow_evaluate_only_nnue(sub_board: &SubBoard) -> Score {
        HALFKP_MODEL_READER
            .to_model(sub_board)
            .evaluate_current_state(sub_board.turn())
    }

    #[cfg(feature = "inbuilt_nnue")]
    #[inline]
    pub fn slow_evaluate(sub_board: &SubBoard) -> Score {
        Self::evaluate_raw(sub_board, || Self::slow_evaluate_only_nnue(sub_board))
    }

    #[inline]
    pub fn reset_variables(&self) {
        self.score_cache.reset_variables();
    }

    #[inline]
    pub fn clear(&self) {
        self.score_cache.clear();
    }

    #[inline]
    pub fn set_size(&self, size: CacheTableSize) {
        self.score_cache.set_size(size);
    }
}

#[cfg(feature = "inbuilt_nnue")]
impl Default for Evaluator {
    fn default() -> Self {
        Self::new(&SubBoard::default())
    }
}
