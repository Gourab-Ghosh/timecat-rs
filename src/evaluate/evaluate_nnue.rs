use super::*;

#[cfg(feature = "inbuilt_nnue")]
static HALFKP_MODEL_READER: LazyLock<HalfKPModelReader> = LazyLock::new(|| {
    let mut reader = std::io::Cursor::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/nnue_dir/nn.nnue"
    )));
    HalfKPModelReader::read(&mut reader)
        .map_err(|_| TimecatError::BadNNUEFile)
        .unwrap()
});

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct EvaluatorNNUE {
    model: HalfKPModel,
    score_cache: SerdeWrapper<Arc<CacheTable<Score>>>,
}

impl EvaluatorNNUE {
    pub fn from_model(model: HalfKPModel) -> Self {
        Self {
            model,
            score_cache: CacheTable::new(EVALUATOR_SIZE).into(),
        }
    }

    #[cfg(feature = "inbuilt_nnue")]
    pub fn new(position: &BoardPosition) -> Self {
        Self::from_model(HALFKP_MODEL_READER.to_model(position))
    }

    pub fn from_nnue_bytes(nnue_bytes: &[u8], position: &BoardPosition) -> Result<Self> {
        let mut reader = std::io::Cursor::new(nnue_bytes);
        let model = HalfKPModelReader::read(&mut reader)
            .map_err(|_| TimecatError::BadNNUEFile)?
            .to_model(position);
        Ok(Self::from_model(model))
    }

    pub fn from_nnue_path(path: &str, position: &BoardPosition) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let model = HalfKPModelReader::read(&mut reader)
            .map_err(|_| TimecatError::BadNNUEFile)?
            .to_model(position);
        Ok(Self::from_model(model))
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
        position: &BoardPosition,
        winning_side: Color,
        is_bishop_knight_endgame: bool,
    ) -> Score {
        let winning_side_king_square = position.get_king_square(winning_side);
        let losing_side_king_square = position.get_king_square(!winning_side);
        let mut probable_least_distant_corner = None;
        if is_bishop_knight_endgame {
            let is_light_squared_bishop =
                !(position.get_piece_mask(Bishop) & BB_LIGHT_SQUARES).is_empty();
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
        let losing_king_opponent_pieces_score = position
            .occupied_color(winning_side)
            .map(|square| 7 - square.distance(losing_side_king_square))
            .sum::<u8>() as Score;
        10 * (10 * king_distance_score + 2 * losing_king_corner_score)
            + losing_king_opponent_pieces_score
    }

    fn force_passed_pawn_push(position: &BoardPosition) -> Score {
        todo!("force_passed_pawn_push {}", position)
    }

    fn king_corner_forcing_evaluation(position: &BoardPosition, material_score: Score) -> Score {
        let is_bishop_knight_endgame = position.get_num_pieces() == 4
            && material_score.abs() == const { Knight.evaluate() + Bishop.evaluate() };
        let winning_side = if material_score.is_positive() {
            White
        } else {
            Black
        };
        let signum = material_score.signum();
        let king_forcing_score =
            Self::force_opponent_king_to_corner(position, winning_side, is_bishop_knight_endgame);
        (50 * PAWN_VALUE + king_forcing_score / 2) * signum + 2 * material_score
    }

    pub fn is_easily_winning_position(position: &BoardPosition, material_score: Score) -> bool {
        if material_score.abs() > const { PAWN_VALUE + Bishop.evaluate() } {
            let white_occupied = position.occupied_color(White);
            let black_occupied = position.occupied_color(Black);
            let num_white_pieces = white_occupied.popcnt();
            let num_black_pieces = black_occupied.popcnt();
            let num_pieces = num_white_pieces + num_black_pieces;
            if num_pieces < 5 {
                if num_white_pieces == 2 && num_black_pieces == 2 {
                    let non_king_white_piece = position
                        .get_piece_type_at(
                            (white_occupied & !position.get_piece_mask(King))
                                .next()
                                .unwrap(),
                        )
                        .unwrap();
                    let non_king_black_piece = position
                        .get_piece_type_at(
                            (black_occupied & !position.get_piece_mask(King))
                                .next()
                                .unwrap(),
                        )
                        .unwrap();
                    let mut non_king_pieces = [non_king_white_piece, non_king_black_piece];
                    non_king_pieces.sort_unstable();
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
                            & !position.get_piece_mask(King))
                        .map(|s| position.get_piece_type_at(s).unwrap())
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

    fn evaluate_raw(position: &BoardPosition, mut nnue_eval_func: impl FnMut() -> Score) -> Score {
        let knights_mask = position.get_piece_mask(Knight);
        if position.get_non_king_pieces_mask() == knights_mask && knights_mask.popcnt() < 3 {
            return 0;
        }
        let material_score = position.get_material_score();
        if Self::is_easily_winning_position(position, material_score) {
            return Self::king_corner_forcing_evaluation(position, material_score);
        }
        let mut nnue_eval = nnue_eval_func();
        if nnue_eval.abs() > WINNING_SCORE_THRESHOLD {
            let multiplier = match_interpolate_float!(
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
                * match_interpolate_float!(
                    0,
                    5.0 * multiplier,
                    MAX_MATERIAL_SCORE,
                    0,
                    position.get_masked_material_score_abs(position.occupied_color(losing_side))
                )
                .round() as Score
                * PAWN_VALUE;
        }
        nnue_eval
    }

    fn hashed_evaluate(&mut self, position: &BoardPosition) -> Score {
        let hash = position.get_hash();
        if let Some(score) = self.score_cache.get(hash) {
            return score;
        }
        let score = Self::evaluate_raw(position, || self.model.update_model_and_evaluate(position));
        self.score_cache.add(hash, score);
        score
    }

    #[cfg(feature = "inbuilt_nnue")]
    #[inline]
    pub fn slow_evaluate_nnue_raw(position: &BoardPosition) -> Score {
        HALFKP_MODEL_READER
            .to_model(position)
            .evaluate_current_state(position.turn())
    }

    #[cfg(feature = "inbuilt_nnue")]
    #[inline]
    pub fn slow_evaluate(position: &BoardPosition) -> Score {
        Self::evaluate_raw(position, || Self::slow_evaluate_nnue_raw(position))
    }

    #[inline]
    pub fn set_size(&self, size: CacheTableSize) {
        self.score_cache.set_size(size);
    }
}

impl PositionEvaluation for EvaluatorNNUE {
    #[inline]
    fn evaluate(&mut self, position: &BoardPosition) -> Score {
        self.hashed_evaluate(position)
    }

    #[inline]
    fn reset_variables(&mut self) {
        self.score_cache.reset_variables();
    }

    #[inline]
    fn clear(&mut self) {
        self.score_cache.clear();
    }

    #[inline]
    fn print_info(&self) {
        print_cache_table_info(
            "Evaluation Cache Table",
            self.score_cache.len(),
            self.score_cache.get_size(),
        );
    }
}

#[cfg(feature = "inbuilt_nnue")]
impl Default for EvaluatorNNUE {
    fn default() -> Self {
        Self::new(&BoardPosition::default())
    }
}
