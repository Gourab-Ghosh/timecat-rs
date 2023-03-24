use super::*;

#[derive(Clone, Copy, Default, Debug)]
pub struct WeightedMove {
    pub _move: Move,
    pub weight: i32,
}

impl WeightedMove {
    pub fn new(_move: Move, weight: i32) -> Self {
        Self {
            _move: _move,
            weight,
        }
    }
}

#[derive(Debug)]
pub struct WeightedMoveListSorter {
    weighted_moves: [WeightedMove; MAX_MOVES_PER_POSITION],
    len: usize,
    idx: usize,
    sorted: bool,
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
            let best_move = get_item_unchecked!(self.weighted_moves, self.idx);
            self.idx += 1;
            return Some(best_move);
        }
        let mut max_weight = MoveWeight::MIN;
        let mut max_idx = self.idx;
        for idx in self.idx..self.len {
            let weighted_move = self.weighted_moves[idx];
            if weighted_move.weight > max_weight {
                max_idx = idx;
                max_weight = weighted_move.weight;
            }
        }
        // unsafe { self.weighted_moves.swap_unchecked(self.idx, max_idx) };
        self.weighted_moves.swap(self.idx, max_idx);
        let best_move = self.weighted_moves[self.idx];
        self.idx += 1;
        Some(best_move)
    }
}

impl FromIterator<WeightedMove> for WeightedMoveListSorter {
    fn from_iter<T: IntoIterator<Item = WeightedMove>>(iter: T) -> Self {
        let mut weighted_moves = [WeightedMove::default(); MAX_MOVES_PER_POSITION];
        let mut len = 0;
        let mut sorted = true;
        let mut last_weight = MoveWeight::MAX;
        for weighted_move in iter {
            weighted_moves[len] = weighted_move;
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

#[derive(Debug)]
pub struct MoveSorter {
    killer_moves: [[Option<Move>; NUM_KILLER_MOVES]; MAX_PLY],
    history_move_scores: [[[MoveWeight; 64]; 2]; 6],
    follow_pv: bool,
    score_pv: bool,
}

impl MoveSorter {
    pub fn reset_variables(&mut self) {
        for ply in 0..MAX_PLY {
            for idx in 0..NUM_KILLER_MOVES {
                self.killer_moves[ply][idx] = None;
            }
        }
        for piece in 0..6 {
            for color in 0..2 {
                for square in 0..64 {
                    self.history_move_scores[piece][color][square] = 0;
                }
            }
        }
        self.follow_pv = false;
        self.score_pv = false;
    }

    pub fn update_killer_moves(&mut self, killer_move: Move, ply: Ply) {
        let arr = &mut self.killer_moves[ply];
        arr.rotate_right(1);
        arr[0] = Some(killer_move);
    }

    pub fn is_killer_move(&self, _move: Move, ply: Ply) -> bool {
        self.killer_moves[ply].contains(&Some(_move))
    }

    pub fn add_history_move(&mut self, history_move: Move, board: &Board, depth: Depth) {
        let depth = (depth as MoveWeight).pow(2);
        let src = history_move.get_source();
        let dest = history_move.get_dest();
        self.history_move_scores[board.piece_at(src).unwrap().to_index()]
            [board.color_at(src).unwrap() as usize][dest.to_index()] += depth;
    }

    #[inline(always)]
    pub fn get_history_score(&self, history_move: Move, board: &Board) -> MoveWeight {
        let src = history_move.get_source();
        let dest = history_move.get_dest();
        self.history_move_scores[board.piece_at(src).unwrap().to_index()]
            [board.color_at(src).unwrap() as usize][dest.to_index()]
    }

    fn get_least_attackers_move(square: Square, board: &chess::Board) -> Option<Move> {
        let mut captute_moves = chess::MoveGen::new_legal(board);
        captute_moves.set_iterator_mask(get_square_bb(square));
        captute_moves.next()
    }

    fn see(square: Square, board: &mut chess::Board) -> Score {
        let least_attackers_move = match Self::get_least_attackers_move(square, board) {
            Some(_move) => _move,
            None => return 0,
        };
        let capture_piece = board.piece_on(square).unwrap_or(Pawn);
        board.clone().make_move(least_attackers_move, board);
        (evaluate_piece(capture_piece) - Self::see(square, board)).max(0)
    }

    fn see_capture(square: Square, board: &mut chess::Board) -> Score {
        let least_attackers_move = match Self::get_least_attackers_move(square, board) {
            Some(_move) => _move,
            None => return 0,
        };
        let capture_piece = board.piece_on(square).unwrap_or(Pawn);
        board.clone().make_move(least_attackers_move, board);
        evaluate_piece(capture_piece) - Self::see(square, board)
    }

    fn mvv_lva(_move: Move, board: &Board) -> MoveWeight {
        MVV_LVA[board.piece_at(_move.get_source()).unwrap().to_index()]
            [board.piece_at(_move.get_dest()).unwrap_or(Pawn).to_index()]
    }

    #[inline(always)]
    fn score_capture(_move: Move, board: &Board) -> MoveWeight {
        Self::see_capture(_move.get_dest(), &mut board.get_sub_board()) as MoveWeight
        // Self::mvv_lva(_move, board)
    }

    fn score_move(
        &mut self,
        _move: Move,
        board: &Board,
        ply: Ply,
        best_move: Option<Move>,
        pv_move: Option<Move>,
    ) -> MoveWeight {
        // best move
        if best_move == Some(_move) {
            return 1294000;
        }
        // pv move
        if self.score_pv && pv_move == Some(_move) {
            self.score_pv = false;
            return 1293000;
        }
        let source = _move.get_source();
        let dest = _move.get_dest();
        let mut sub_board = board.get_sub_board();
        sub_board.clone().make_move(_move, &mut sub_board);
        let checkers = *sub_board.checkers();
        let moving_piece = board.piece_at(source).unwrap();
        // ckeck
        if checkers != BB_EMPTY {
            return 1292000 + 10 * checkers.popcnt() as MoveWeight - moving_piece as MoveWeight;
        }
        if board.is_capture(_move) {
            let capture_value = Self::score_capture(_move, board);
            return 1291000 + capture_value;
        }
        for (idx, &option_move) in self.killer_moves[ply].iter().enumerate() {
            if option_move == Some(_move) {
                return 1290000 - idx as MoveWeight;
            }
        }
        self.get_history_score(_move, board)
    }

    // const HASH_MOVE_SCORE: MoveWeight = 25000;
    // const PV_MOVE_SCORE: MoveWeight = 24000;
    // const WINNING_CAPTURES_OFFSET: MoveWeight = 10;
    // const KILLER_MOVE_SCORE: MoveWeight = 2;
    // const CASTLING_SCORE: MoveWeight = 1;
    // const HISTORY_MOVE_OFFSET: MoveWeight = -30000;
    // const LOSING_CAPTURES_OFFSET: MoveWeight = -30001;

    // fn score_move(
    //     &mut self,
    //     _move: Move,
    //     board: &Board,
    //     ply: Ply,
    //     best_move: Option<Move>,
    //     pv_move: Option<Move>,
    // ) -> MoveWeight {
    //     if Some(_move) == best_move {
    //         return Self::HASH_MOVE_SCORE;
    //     }

    //     if self.score_pv {
    //         if let Some(m) = pv_move {
    //             if m == _move {
    //                 self.score_pv = false;
    //                 return Self::PV_MOVE_SCORE;
    //             }
    //         }
    //     }

    //     if board.is_quiet(_move) {
    //         if self.killer_moves[ply].contains(&Some(_move)) {
    //             return Self::KILLER_MOVE_SCORE;
    //         }

    //         if board.is_castling(_move) {
    //             return Self::CASTLING_SCORE;
    //         }

    //         return Self::HISTORY_MOVE_OFFSET + self.get_history_score(_move, board);
    //     }

    //     let mut score = 0;
    //     if board.is_capture(_move) {
    //         if board.is_en_passant(_move) {
    //             return Self::WINNING_CAPTURES_OFFSET;
    //         }

    //         score += Self::mvv_lva(_move, board);

    //         if Self::score_capture(_move, board).is_positive() {
    //             score += Self::WINNING_CAPTURES_OFFSET;
    //         } else {
    //             score += Self::LOSING_CAPTURES_OFFSET;
    //         }
    //     }

    //     score += match _move.get_promotion() {
    //         Some(piece) => evaluate_piece(piece),
    //         None => 0,
    //     } as MoveWeight;
    //     score
    // }

    pub fn get_weighted_sort_moves<T: IntoIterator<Item = Move>>(
        &mut self,
        moves_gen: T,
        board: &Board,
        ply: Ply,
        best_move: Option<Move>,
        optional_pv_move: Option<Move>,
    ) -> WeightedMoveListSorter {
        let moves_vec = Vec::from_iter(moves_gen.into_iter());
        if self.follow_pv {
            self.follow_pv = false;
            if let Some(pv_move) = optional_pv_move {
                if moves_vec.contains(&pv_move) {
                    self.follow_pv = true;
                    self.score_pv = true;
                }
            }
        }
        WeightedMoveListSorter::from_iter(moves_vec.into_iter().enumerate().map(|(idx, m)| {
            WeightedMove::new(
                m,
                1000 * self.score_move(m, board, ply, best_move, optional_pv_move)
                    + idx as MoveWeight,
            )
        }))
    }

    pub fn get_weighted_capture_moves<T: IntoIterator<Item = Move>>(
        &self,
        moves_gen: T,
        board: &Board,
    ) -> WeightedMoveListSorter {
        WeightedMoveListSorter::from_iter(moves_gen.into_iter().enumerate().map(|(idx, m)| {
            WeightedMove::new(
                m,
                1000 * Self::score_capture(m, board) + MAX_MOVES_PER_POSITION as MoveWeight
                    + idx as MoveWeight,
            )
        }))
    }

    pub fn follow_pv(&mut self) {
        self.follow_pv = true;
    }
}

impl Default for MoveSorter {
    fn default() -> Self {
        Self {
            killer_moves: [[None; NUM_KILLER_MOVES]; MAX_PLY],
            history_move_scores: [[[0; 64]; 2]; 6],
            follow_pv: false,
            score_pv: false,
        }
    }
}
