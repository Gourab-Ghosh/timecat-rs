use super::*;

#[derive(Clone, Copy, Default, Debug)]
pub struct WeightedMove {
    pub _move: Move,
    pub weight: i32,
}

impl WeightedMove {
    pub fn new(_move: Move, weight: i32) -> Self {
        Self { _move, weight }
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

    pub fn is_killer_move(&self, _move: &Move, ply: Ply) -> bool {
        self.killer_moves[ply].contains(&Some(*_move))
    }

    pub fn add_history_move(&mut self, history_move: Move, board: &Board, depth: Depth) {
        let depth = (depth as MoveWeight).pow(2);
        let src = history_move.get_source();
        let dest = history_move.get_dest();
        self.history_move_scores[board.piece_at(src).unwrap().to_index()]
            [board.color_at(src).unwrap() as usize][dest.to_index()] += depth;
    }

    fn get_least_attackers_move(&self, square: Square, board: &chess::Board) -> Option<Move> {
        let mut captute_moves = chess::MoveGen::new_legal(board);
        captute_moves.set_iterator_mask(get_square_bb(square));
        captute_moves.next()
    }

    fn see(&self, square: Square, board: &mut chess::Board) -> Score {
        let least_attackers_move = match self.get_least_attackers_move(square, board) {
            Some(_move) => _move,
            None => return 0,
        };
        let capture_piece = board.piece_on(square).unwrap_or(Pawn);
        board.clone().make_move(least_attackers_move, board);
        (evaluate_piece(capture_piece) - self.see(square, board)).max(0)
    }

    fn see_capture(&self, square: Square, board: &mut chess::Board) -> Score {
        let least_attackers_move = match self.get_least_attackers_move(square, board) {
            Some(_move) => _move,
            None => return 0,
        };
        let capture_piece = board.piece_on(square).unwrap_or(Pawn);
        board.clone().make_move(least_attackers_move, board);
        evaluate_piece(capture_piece) - self.see(square, board)
    }

    fn mvv_lva(&self, _move: Move, board: &Board) -> MoveWeight {
        MVV_LVA[board
            .piece_at(_move.get_source())
            .unwrap_or(Pawn)
            .to_index()][board.piece_at(_move.get_dest()).unwrap().to_index()]
    }

    #[inline(always)]
    fn score_capture(&self, _move: Move, board: &Board) -> MoveWeight {
        self.see_capture(_move.get_dest(), &mut board.get_sub_board()) as MoveWeight
        // self.mvv_lva(_move, board)
    }

    fn score_threat(&self, _move: Move, sub_board: &chess::Board) -> MoveWeight {
        let dest = _move.get_dest();
        let attacker_piece_score = evaluate_piece(sub_board.piece_on(_move.get_source()).unwrap());
        let attacked_piece_score = evaluate_piece(sub_board.piece_on(dest).unwrap_or(Pawn));
        let mut threat_score = attacker_piece_score - attacked_piece_score;
        if sub_board.pinned() == &get_square_bb(dest) {
            threat_score *= 2;
        }
        (threat_score + 16 * PAWN_VALUE) as MoveWeight
    }

    fn score_move(
        &mut self,
        _move: Move,
        board: &Board,
        ply: Ply,
        best_move: Option<Move>,
        pv_move: Option<Move>,
    ) -> MoveWeight {
        if let Some(m) = best_move {
            if m == _move {
                return 1294000000;
            }
        }
        if self.score_pv {
            if let Some(m) = pv_move {
                if m == _move {
                    self.score_pv = false;
                    return 1293000000;
                }
            }
        }
        let source = _move.get_source();
        let dest = _move.get_dest();
        let mut sub_board = board.get_sub_board();
        sub_board.clone().make_move(_move, &mut sub_board);
        let checkers = *sub_board.checkers();
        let moving_piece = board.piece_at(source).unwrap();
        if checkers != BB_EMPTY {
            return 1292000000 + 100 * checkers.popcnt() as MoveWeight + moving_piece as MoveWeight;
        }
        if board.is_capture(_move) {
            let capture_value = self.score_capture(_move, board);
            return 1291000000 + capture_value;
        }
        for (i, option_move) in self.killer_moves[ply].iter().enumerate() {
            if let Some(killer_move) = option_move {
                if killer_move == &_move {
                    return 1290000000 - i as MoveWeight;
                }
            }
        }
        // let threat_score = match sub_board.null_move() {
        //     Some(board) => {
        //         let mut moves = chess::MoveGen::new_legal(&board);
        //         moves.set_iterator_mask(*board.color_combined(!board.side_to_move()));
        //         moves.into_iter().map(|m| self.threat_value(m, &board)).sum::<MoveWeight>()
        //     },
        //     None => 0,
        // };

        // 100 * history_moves_score + threat_score
        self.history_move_scores[moving_piece.to_index()][board.color_at(source).unwrap() as usize]
            [dest.to_index()]
    }

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
        WeightedMoveListSorter::from_iter(moves_vec.into_iter().map(|m| {
            WeightedMove::new(
                m,
                self.score_move(m, board, ply, best_move, optional_pv_move),
            )
        }))
    }

    pub fn get_weighted_capture_moves<T: IntoIterator<Item = Move>>(
        &self,
        moves_gen: T,
        board: &Board,
    ) -> WeightedMoveListSorter {
        WeightedMoveListSorter::from_iter(
            moves_gen
                .into_iter()
                .map(|m| WeightedMove::new(m, self.score_capture(m, board) as MoveWeight)),
        )
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
