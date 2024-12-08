use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct WeightedMoveListSorter {
    weighted_moves: Vec<WeightedMove>,
    len: usize,
    idx: usize,
    sorted: bool,
}

impl WeightedMoveListSorter {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get_weighted_moves(&self) -> &[WeightedMove] {
        &self.weighted_moves
    }
}

impl Iterator for WeightedMoveListSorter {
    type Item = WeightedMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.len {
            self.idx = 0;
            self.sorted = true;
            return None;
        }
        if self.sorted {
            let best_move = *get_item_unchecked!(self.weighted_moves, self.idx);
            self.idx += 1;
            return Some(best_move);
        }
        let mut max_weight = MoveWeight::MIN;
        let mut max_idx = self.idx;
        for idx in self.idx..self.len {
            let weighted_move = get_item_unchecked!(self.weighted_moves, idx);
            if weighted_move.weight > max_weight {
                max_idx = idx;
                max_weight = weighted_move.weight;
            }
        }
        // unsafe { self.weighted_moves.swap_unchecked(self.idx, max_idx) };
        self.weighted_moves.swap(self.idx, max_idx);
        let best_move = *get_item_unchecked!(self.weighted_moves, self.idx);
        self.idx += 1;
        Some(best_move)
    }
}

impl FromIterator<WeightedMove> for WeightedMoveListSorter {
    fn from_iter<T: IntoIterator<Item = WeightedMove>>(iter: T) -> Self {
        let mut weighted_moves = Vec::new();
        let mut len = 0;
        let mut sorted = true;
        let mut last_weight = MoveWeight::MAX;
        for weighted_move in iter {
            weighted_moves.push(weighted_move);
            if sorted {
                sorted = last_weight > weighted_move.weight;
                last_weight = weighted_move.weight;
            }
            len += 1;
        }
        Self {
            weighted_moves,
            len,
            idx: 0,
            sorted,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MoveSorter {
    killer_moves: SerdeWrapper<[SerdeWrapper<[Option<Move>; NUM_KILLER_MOVES]>; MAX_PLY]>,
    history_move_scores: SerdeWrapper<[SerdeWrapper<[MoveWeight; 64]>; 12]>,
    follow_pv: bool,
    score_pv: bool,
}

impl MoveSorter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_variables(&mut self) {
        self.killer_moves
            .fill(SerdeWrapper::new([None; NUM_KILLER_MOVES]));
        self.history_move_scores = SerdeWrapper::new([SerdeWrapper::new([0; 64]); 12]);
        self.follow_pv = false;
        self.score_pv = false;
    }

    pub fn update_killer_moves(&mut self, killer_move: Move, ply: Ply) {
        let arr = get_item_unchecked_mut!(self.killer_moves, ply);
        arr.rotate_right(1);
        *get_item_unchecked_mut!(arr, 0) = Some(killer_move);
    }

    pub fn is_killer_move(&self, move_: Move, ply: Ply) -> bool {
        // TODO: Scope for improvement ig?
        get_item_unchecked!(self.killer_moves, ply).contains(&Some(move_))
    }

    pub fn add_history_move(&mut self, history_move: Move, position: &BoardPosition, depth: Depth) {
        let depth = (depth as MoveWeight).pow(2);
        let src = history_move.get_source();
        let dest = history_move.get_dest();
        let piece = position.get_piece_at(src).unwrap();
        *get_item_unchecked_mut!(self.history_move_scores, piece.to_index(), dest.to_index()) +=
            depth;
    }

    #[inline]
    pub fn get_history_score(&self, history_move: Move, position: &BoardPosition) -> MoveWeight {
        let src = history_move.get_source();
        let dest = history_move.get_dest();
        let piece = position.get_piece_at(src).unwrap();
        *get_item_unchecked!(self.history_move_scores, piece.to_index(), dest.to_index())
    }

    fn get_least_attackers_move(square: Square, position: &BoardPosition) -> Option<Move> {
        position
            .generate_masked_legal_moves(position.self_occupied(), square.to_bitboard())
            .next() // No need to find least attacker as the moves are already sorted
    }

    // fn get_least_attackers_move(square: Square, position: &BoardPosition) -> Option<Move> {
    //     if position.is_check() {
    //         position
    //             .generate_masked_legal_moves(
    //                 position.self_occupied(),
    //                 square.to_bitboard(),
    //             )
    //             .next() // No need to find least attacker as the moves are already sorted
    //     } else {
    //         let attackers_mask = position.get_attackers_mask(square, position.turn());
    //         for piece_type in ALL_PIECE_TYPES {
    //             let least_attackers = position.get_piece_mask(piece_type) & attackers_mask;
    //             if !least_attackers.is_empty() {
    //                 return Some(
    //                     Move::new_unchecked(
    //                         least_attackers.to_square(),
    //                         square,
    //                         if piece_type == Pawn
    //                             && square.get_rank() == position.turn().to_their_backrank()
    //                         {
    //                             Some(Queen)
    //                         } else {
    //                             None
    //                         },
    //                     ),
    //                 );
    //             }
    //         }
    //         None
    //     }
    // }

    fn see(square: Square, position: &BoardPosition) -> Score {
        let least_attackers_move = match Self::get_least_attackers_move(square, position) {
            Some(valid_or_null_move) => valid_or_null_move,
            None => return 0,
        };
        let capture_piece = position.get_piece_type_at(square).unwrap_or(Pawn);
        (capture_piece.evaluate()
            - Self::see(square, &position.make_move_new(least_attackers_move)))
        .max(0)
    }

    fn see_capture(square: Square, position: &BoardPosition) -> Score {
        let least_attackers_move = match Self::get_least_attackers_move(square, position) {
            Some(valid_or_null_move) => valid_or_null_move,
            None => return 0,
        };
        let capture_piece = position.get_piece_type_at(square).unwrap_or(Pawn);
        capture_piece.evaluate() - Self::see(square, &position.make_move_new(least_attackers_move))
    }

    fn mvv_lva(move_: Move, position: &BoardPosition) -> MoveWeight {
        *get_item_unchecked!(
            MVV_LVA,
            position
                .get_piece_type_at(move_.get_source())
                .unwrap()
                .to_index(),
            position
                .get_piece_type_at(move_.get_dest())
                .unwrap_or(Pawn)
                .to_index()
        )
    }

    #[inline]
    fn score_capture(move_: Move, best_move: Option<Move>, position: &BoardPosition) -> MoveWeight {
        if Some(move_) == best_move {
            return 10000;
        }
        Self::see_capture(move_.get_dest(), position) as MoveWeight
    }

    fn score_easily_winning_position_moves(
        position: &BoardPosition,
        source: Square,
        dest: Square,
    ) -> Option<MoveWeight> {
        let moving_piece = position.get_piece_type_at(source).unwrap();
        if moving_piece != Pawn {
            let losing_color = !position.get_winning_side().unwrap_or(White);
            let losing_king_square = position.get_king_square(losing_color);
            if losing_king_square == source {
                return Some(-100 * source.distance(Square::E4) as MoveWeight);
            }
            let source_distance = source.distance(losing_king_square);
            let dest_distance = dest.distance(losing_king_square);
            if dest_distance < source_distance {
                return Some(
                    50 * match moving_piece {
                        King => 5,
                        Knight => 4,
                        Queen => 3,
                        Rook => 2,
                        Bishop => 1,
                        _ => unreachable!(),
                    } - dest_distance as MoveWeight,
                );
            }
        }
        None
    }

    fn score_move(
        &mut self,
        move_: Move,
        position: &BoardPosition,
        ply: Ply,
        best_move: Option<Move>,
        pv_move: Option<Move>,
    ) -> MoveWeight {
        // pv move
        if self.score_pv && pv_move == Some(move_) {
            self.score_pv = false;
            return 900000;
        }
        // best move
        if best_move == Some(move_) {
            return 800000;
        }
        if position.is_capture(move_) {
            return 600000 + Self::score_capture(move_, None, position);
        }
        for (idx, &stored_move) in get_item_unchecked!(self.killer_moves, ply)
            .iter()
            .enumerate()
        {
            if stored_move == Some(move_) {
                return 500000 - idx as MoveWeight;
            }
        }
        // history
        let history_score = self.get_history_score(move_, position);
        if history_score != 0 {
            return 400000 + history_score;
        }
        let move_made_position = position.make_move_new(move_);
        // check
        let checkers = move_made_position.get_checkers();
        let moving_piece = position.get_piece_type_at(move_.get_source()).unwrap();
        if !checkers.is_empty() {
            return -700000 + 10 * checkers.popcnt() as MoveWeight - moving_piece as MoveWeight;
        }
        MAX_MOVES_PER_POSITION as MoveWeight
            - move_made_position.generate_legal_moves().len() as MoveWeight
    }

    pub fn get_weighted_moves_sorted(
        &mut self,
        position: &BoardPosition,
        moves: impl IntoIterator<Item = Move>,
        transposition_table: &TranspositionTable,
        ply: Ply,
        mut best_move: Option<Move>,
        pv_move: Option<Move>,
    ) -> WeightedMoveListSorter {
        if best_move.is_none() {
            best_move = transposition_table.read_best_move(position.get_hash());
        }
        let moves_vec = moves.into_iter().collect_vec();
        if self.follow_pv {
            self.follow_pv = false;
            if let Some(move_) = pv_move {
                if moves_vec.contains(&move_) {
                    self.follow_pv = true;
                    self.score_pv = true;
                }
            }
        }
        if moves_vec.len() < 2 {
            return WeightedMoveListSorter::from_iter(
                moves_vec.iter().map(|&move_| WeightedMove::new(move_, 0)),
            );
        }
        WeightedMoveListSorter::from_iter(moves_vec.into_iter().enumerate().map(|(idx, m)| {
            WeightedMove::new(
                m,
                (self.score_move(m, position, ply, best_move, pv_move) << 10) - idx as MoveWeight,
            )
        }))
    }

    pub fn get_weighted_capture_moves_sorted(
        &self,
        position: &BoardPosition,
        transposition_table: &TranspositionTable,
    ) -> WeightedMoveListSorter {
        let best_move = transposition_table.read_best_move(position.get_hash());
        WeightedMoveListSorter::from_iter(position.generate_legal_captures().enumerate().map(
            |(idx, m)| {
                WeightedMove::new(
                    m,
                    1000 * Self::score_capture(m, best_move, position)
                        + MAX_MOVES_PER_POSITION as MoveWeight
                        - idx as MoveWeight,
                )
            },
        ))
    }

    pub fn score_root_moves<P: PositionEvaluation>(
        board: &Board,
        evaluator: &mut P,
        move_: Move,
        pv_move: Option<Move>,
        best_moves: &[Move],
    ) -> MoveWeight {
        if !board.is_endgame() {
            if let Some(index) = best_moves
                .iter()
                .take(NUM_BEST_ROOT_MOVES_TO_SEARCH_FIRST)
                .position(|&best_move| best_move == move_)
            {
                return 200_000 - index as MoveWeight;
            }
        }
        if pv_move.is_some() {
            return 100_000;
        }
        if board.gives_repetition(move_) {
            return -50;
        }
        let is_endgame = board.is_endgame();
        if !is_endgame && board.gives_claimable_threefold_repetition(move_) {
            return -40;
        }
        let mut score = 0;
        let mut evaluation = evaluator.evaluate_flipped(board) as MoveWeight;
        if evaluation == 0 {
            evaluation = 1;
        }
        if is_endgame {
            if move_.get_promotion().is_some() {
                score += 30_000;
            }
            if board.is_capture(move_) {
                score += 2000 * evaluation.signum() + Self::score_capture(move_, None, board);
            }
            let source = move_.get_source();
            if board.is_passed_pawn(source) {
                let promotion_distance = board
                    .turn()
                    .to_their_backrank()
                    .to_index()
                    .abs_diff(source.get_rank().to_index());
                score += 20 - promotion_distance as MoveWeight;
            }
        }
        score
    }

    pub fn follow_pv(&mut self) {
        self.follow_pv = true;
    }

    pub fn is_following_pv(&self) -> bool {
        self.follow_pv
    }
}

impl Default for MoveSorter {
    fn default() -> Self {
        Self {
            killer_moves: SerdeWrapper::new([SerdeWrapper::new([None; NUM_KILLER_MOVES]); MAX_PLY]),
            history_move_scores: SerdeWrapper::new([SerdeWrapper::new([0; 64]); 12]),
            follow_pv: false,
            score_pv: false,
        }
    }
}
