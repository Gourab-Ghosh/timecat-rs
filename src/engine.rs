use super::*;
use EntryFlag::*;

pub struct Engine {
    pub board: Board,
    num_nodes_searched: usize,
    ply: Ply,
    pv_length: [usize; MAX_PLY],
    pv_table: [[Option<Move>; MAX_PLY]; MAX_PLY],
    move_sorter: MoveSorter,
    transposition_table: TranspositionTable,
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            num_nodes_searched: 0,
            ply: 0,
            pv_length: [0; MAX_PLY],
            pv_table: [[None; MAX_PLY]; MAX_PLY],
            move_sorter: MoveSorter::default(),
            transposition_table: TranspositionTable::default(),
        }
    }

    pub fn push(&mut self, _move: Move) {
        self.board.push(_move);
        self.ply += 1;
    }

    pub fn pop(&mut self) -> Move {
        self.ply -= 1;
        self.board.pop()
    }

    pub fn push_null_move(&mut self) {
        self.board.push_null_move();
        self.ply += 1;
    }

    pub fn pop_null_move(&mut self) {
        self.ply -= 1;
        self.board.pop_null_move()
    }

    fn reset_variables(&mut self) {
        self.ply = 0;
        self.num_nodes_searched = 0;
        for i in 0..MAX_PLY {
            self.pv_length[i] = 0;
            for j in 0..MAX_PLY {
                self.pv_table[i][j] = None;
            }
        }
        self.move_sorter.reset_variables();
        self.transposition_table.clear();
    }

    fn update_pv_table(&mut self, _move: Move) {
        self.pv_table[self.ply][self.ply] = Some(_move);
        for next_ply in (self.ply + 1)..self.pv_length[self.ply + 1] {
            self.pv_table[self.ply][next_ply] = self.pv_table[self.ply + 1][next_ply];
        }
        self.pv_length[self.ply] = self.pv_length[self.ply + 1];
    }

    pub fn can_apply_lmr(&self, _move: &Move) -> bool {
        // return false;
        if _move.get_promotion().is_some() {
            return false;
        }
        if self.move_sorter.is_killer_move(_move, self.ply) {
            return false;
        }
        true
    }

    fn get_lmr_reduction(&self, depth: Depth, move_index: usize) -> Depth {
        // if depth < 3 || move_index < 3 {
        //     return 0;
        // }
        // let reduction = (depth / 2).min(3);
        // reduction
        (LMR_BASE_REDUCTION + (depth as f32).ln() * (move_index as f32).ln() / LMR_MOVE_DIVIDER)
            .round() as Depth
        // (LMR_BASE_REDUCTION + ((depth as usize * move_index) as f32).sqrt() / LMR_MOVE_DIVIDER).ceil() as Depth
    }

    fn get_modified_score(&self, score: Score, draw_score: Score) -> Score {
        let mut modified_score = -if score == 0 && !self.board.is_threefold_repetition() {
            draw_score
        } else {
            score
        };
        modified_score -= draw_score * (self.board.get_num_repetitions() as Score - 1) as Score / 2;
        modified_score
    }

    fn search(&mut self, depth: Depth, mut alpha: Score, beta: Score) -> Score {
        self.pv_length[self.ply] = self.ply;
        if self.board.is_game_over() {
            return if self.board.is_checkmate() {
                -CHECKMATE_SCORE
            } else {
                0
            };
        }
        let key = self.board.get_hash();
        let (mut score, mut write_tt) = (-CHECKMATE_SCORE, false);
        let mut flag = HashAlpha;
        let draw_score = if self.board.is_endgame() {
            0
        } else {
            DRAW_SCORE
        };
        for (move_index, weighted_move) in self
            .move_sorter
            .get_weighted_sort_moves(
                self.board.generate_legal_moves(),
                &self.board,
                self.ply,
                self.transposition_table.read_best_move(key),
                self.pv_table[0][self.ply],
            )
            .enumerate()
        {
            let _move = weighted_move._move;
            self.push(_move);
            if move_index == 0 || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, true).0 > alpha {
                (score, write_tt) = self.alpha_beta(depth - 1, -beta, -alpha, true);
                score = self.get_modified_score(score, draw_score);
            }
            self.pop();
            if score > alpha {
                flag = HashExact;
                alpha = score;
                self.update_pv_table(_move);
                if score >= beta {
                    self.transposition_table.write(
                        key,
                        depth,
                        self.ply,
                        beta,
                        HashBeta,
                        Some(_move),
                        write_tt,
                    );
                    return beta;
                }
            }
        }
        self.transposition_table.write(
            key,
            depth,
            self.ply,
            alpha,
            flag,
            self.pv_table[self.ply][self.ply],
            write_tt,
        );
        alpha
    }

    fn alpha_beta(
        &mut self,
        mut depth: Depth,
        mut alpha: Score,
        mut beta: Score,
        apply_null_move: bool,
    ) -> (Score, bool) {
        self.num_nodes_searched += 1;
        self.pv_length[self.ply] = self.ply;
        let num_pieces = self.board.get_num_pieces();
        let is_endgame = num_pieces <= ENDGAME_PIECE_THRESHOLD;
        let not_in_check = !self.board.is_check();
        let is_pvs_node = alpha != beta - 1;
        if !is_pvs_node && (self.board.is_threefold_repetition() || self.board.is_fifty_moves()) {
            return (0, false);
        }
        if self.board.is_insufficient_material() {
            return (0, true);
        }
        let mate_score = CHECKMATE_SCORE - self.ply as Score;
        let moves_gen = self.board.generate_legal_moves();
        if moves_gen.len() == 0 {
            return (if not_in_check { 0 } else { -mate_score }, true);
        }
        // if !not_in_check && depth < 4 {
        //     depth += 1
        // }
        depth = depth.max(0);
        let key = self.board.get_hash();
        let best_move = if is_pvs_node {
            self.transposition_table.read_best_move(key)
        } else {
            match self.transposition_table.read(key, depth, alpha, beta) {
                (Some(score), _) => return (score, true),
                (None, best_move) => best_move,
            }
        };
        if depth == 0 {
            return (self.quiescence(alpha, beta), true);
        }
        // mate distance pruning
        alpha = alpha.max(-mate_score);
        beta = beta.min(mate_score - 1);
        if alpha >= beta {
            return (alpha, true);
        }
        let mut futility_pruning = false;
        if not_in_check && !is_pvs_node {
            // static evaluation pruning
            let static_evaluation = self.board.evaluate_flipped();
            if depth < 3 && (beta - 1).abs() > -mate_score + PAWN_VALUE {
                let evaluation_margin = PAWN_VALUE * depth as i16;
                let evaluation_diff = static_evaluation - evaluation_margin;
                if evaluation_diff >= beta {
                    return (evaluation_diff, true);
                }
            }
            // // null move reduction
            // if apply_null_move && depth > 2 {
            //     let r = NULL_MOVE_MIN_REDUCTION
            //         + (depth - NULL_MOVE_MIN_DEPTH) / NULL_MOVE_DEPTH_DIVIDER;
            //     if depth > r {
            //         self.push_null_move();
            //         let (score, write_tt) = self.alpha_beta(depth - 1 - r, -beta, -beta + 1, false);
            //         let score = -score;
            //         self.pop_null_move();
            //         if score >= beta {
            //             return (beta, write_tt);
            //         }
            //     }
            //     // razoring
            //     if depth < 4 {
            //         let mut evaluation = static_evaluation + PAWN_VALUE;
            //         if evaluation < beta {
            //             if depth == 1 {
            //                 let new_evaluation = self.quiescence(alpha, beta);
            //                 return (new_evaluation.max(evaluation), true);
            //             }
            //             evaluation += PAWN_VALUE;
            //             if evaluation < beta && depth < 4 {
            //                 let new_evaluation = self.quiescence(alpha, beta);
            //                 if new_evaluation < beta {
            //                     return (new_evaluation.max(evaluation), true);
            //                 }
            //             }
            //         }
            //     }
            // }
            if apply_null_move && static_evaluation > beta {
                let r = NULL_MOVE_MIN_REDUCTION
                    + (depth - NULL_MOVE_MIN_DEPTH) / NULL_MOVE_DEPTH_DIVIDER;
                if depth > r {
                    self.push_null_move();
                    let (score, write_tt) = self.alpha_beta(depth - 1 - r, -beta, -beta + 1, false);
                    let score = -score;
                    self.pop_null_move();
                    if score >= beta {
                        return (beta, write_tt);
                    }
                }
                // razoring
                if !is_pvs_node && self.board.has_non_pawn_material() && depth < 4 {
                    let mut evaluation = static_evaluation + PAWN_VALUE;
                    if evaluation < beta {
                        if depth == 1 {
                            let new_evaluation = self.quiescence(alpha, beta);
                            return (new_evaluation.max(evaluation), true);
                        }
                        evaluation += PAWN_VALUE;
                        if evaluation < beta && depth < 4 {
                            let new_evaluation = self.quiescence(alpha, beta);
                            if new_evaluation < beta {
                                return (new_evaluation.max(evaluation), true);
                            }
                        }
                    }
                }
            }
            // futility pruning
            if depth < 4 && alpha < mate_score && !is_endgame {
                let futility_margin = match depth {
                    0 => 0,
                    1 => PAWN_VALUE,
                    2 => evaluate_piece(Knight),
                    3 => evaluate_piece(Rook),
                    _ => unreachable!(),
                };
                futility_pruning = static_evaluation + futility_margin <= alpha;
            }
        }
        let mut flag = HashAlpha;
        let weighted_moves = self.move_sorter.get_weighted_sort_moves(
            moves_gen,
            &self.board,
            self.ply,
            best_move,
            self.pv_table[0][self.ply],
        );
        let mut write_tt = true;
        for (move_index, weighted_move) in weighted_moves.enumerate() {
            let _move = weighted_move._move;
            let not_capture_move = !self.board.is_capture(_move);
            if move_index != 0
                && futility_pruning
                && not_capture_move
                && _move.get_promotion().is_none()
                && not_in_check
            {
                continue;
            }
            let safe_to_apply_lmr =
                self.can_apply_lmr(&_move) && !is_pvs_node && not_capture_move && not_in_check;
            self.push(_move);
            let mut score: Score;
            if move_index == 0 {
                (score, write_tt) = self.alpha_beta(depth - 1, -beta, -alpha, true);
                score = -score;
            } else {
                if move_index >= FULL_DEPTH_SEARCH_LMR
                    && depth >= REDUCTION_LIMIT_LMR
                    && safe_to_apply_lmr
                    && !DISABLE_LMR
                {
                    let lmr_reduction = self.get_lmr_reduction(depth, move_index);
                    (score, write_tt) =
                        self.alpha_beta(depth - 1 - lmr_reduction, -alpha - 1, -alpha, true);
                    score = -score;
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    (score, write_tt) = self.alpha_beta(depth - 1, -alpha - 1, -alpha, true);
                    score = -score;
                    if score > alpha && score < beta {
                        (score, write_tt) = self.alpha_beta(depth - 1, -beta, -alpha, true);
                        score = -score;
                    }
                }
            }
            self.pop();
            if score > alpha {
                flag = HashExact;
                self.update_pv_table(_move);
                alpha = score;
                if not_capture_move {
                    self.move_sorter.add_history_move(_move, &self.board, depth);
                }
                if score >= beta {
                    self.transposition_table.write(
                        key,
                        depth,
                        self.ply,
                        beta,
                        HashBeta,
                        Some(_move),
                        write_tt,
                    );
                    if not_capture_move {
                        self.move_sorter.update_killer_moves(_move, self.ply);
                    }
                    return (beta, write_tt);
                }
            }
        }
        self.transposition_table.write(
            key,
            depth,
            self.ply,
            alpha,
            flag,
            self.pv_table[self.ply][self.ply],
            write_tt,
        );
        (alpha, write_tt)
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        self.num_nodes_searched += 1;
        if self.board.is_draw() {
            return 0;
        }
        let evaluation = self.board.evaluate_flipped();
        if evaluation >= beta {
            return beta;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        for weighted_move in self
            .move_sorter
            .get_weighted_capture_moves(self.board.generate_legal_captures(), &self.board)
        {
            if weighted_move.weight < 0 {
                break;
            }
            self.push(weighted_move._move);
            let score = -self.quiescence(-beta, -alpha);
            self.pop();
            if score > alpha {
                alpha = score;
                if score >= beta {
                    return beta;
                }
            }
        }
        alpha
    }

    fn get_pv(&self, depth: u8) -> Vec<Move> {
        let mut pv = Vec::new();
        for i in 0..self.pv_length[depth as usize] {
            pv.push(self.pv_table[depth as usize][i].unwrap_or_default());
        }
        pv
    }

    fn get_pv_as_uci(&self, depth: u8) -> String {
        let mut pv_string = String::new();
        for _move in self.get_pv(depth) {
            pv_string.push_str(&_move.to_string());
            pv_string.push(' ');
        }
        return pv_string.trim().to_string();
    }

    fn get_pv_as_algebraic(&self, depth: u8, long: bool) -> String {
        let mut board = self.board.clone();
        let mut pv_string = String::new();
        for _move in self.get_pv(depth) {
            pv_string += &(if board.is_legal(_move) {
                board.san_and_push(_move).unwrap()
            } else {
                colorize(_move, ERROR_MESSAGE_STYLE)
            } + " ");
        }
        return pv_string.trim().to_string();
    }

    fn get_pv_as_san(&self, depth: u8) -> String {
        self.get_pv_as_algebraic(depth, false)
    }

    fn get_pv_as_lan(&self, depth: u8) -> String {
        self.get_pv_as_algebraic(depth, true)
    }

    pub fn get_pv_string(&self) -> String {
        self.get_pv_as_san(0)
    }

    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.pv_table[0][0]
    }

    pub fn print_warning_message(&self, current_depth: Depth) {
        let warning_message = format!(
            "Resetting alpha to -INFINITY and beta to INFINITY at depth {}",
            current_depth
        );
        println!("{}", colorize(warning_message, WARNING_MESSAGE_STYLE));
    }

    pub fn print_search_info(&self, current_depth: Depth, score: Score, time_passed: Duration) {
        let style = SUCCESS_MESSAGE_STYLE;
        println!(
            "{} {} {} {} {} {} {} {} {} {:.3} {} {}",
            colorize("info depth", style),
            current_depth,
            colorize("score", style),
            score_to_string(score),
            colorize("nodes", style),
            self.num_nodes_searched,
            colorize("nps", style),
            (self.num_nodes_searched as u128 * 10_u128.pow(9)) / time_passed.as_nanos(),
            colorize("time", style),
            time_passed.as_secs_f32(),
            colorize("pv", style),
            self.get_pv_string(),
        );
    }

    pub fn go(&mut self, depth: Depth, print: bool) -> (Move, Score) {
        self.reset_variables();
        let mut current_depth = 1;
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut score;
        let clock = Instant::now();
        loop {
            if FOLLOW_PV {
                self.move_sorter.follow_pv();
            }
            score = self.search(current_depth, alpha, beta);
            let time_passed = clock.elapsed();
            if score <= alpha || score >= beta {
                if print {
                    self.print_warning_message(current_depth);
                }
                alpha = -INFINITY;
                beta = INFINITY;
                continue;
            }
            alpha = score - ASPIRATION_WINDOW_CUTOFF;
            beta = score + ASPIRATION_WINDOW_CUTOFF;
            if self.board.turn() == Black {
                score = -score;
            }
            if print {
                self.print_search_info(current_depth, score, time_passed);
            }
            if current_depth == depth || is_checkmate(score) {
                break;
            }
            current_depth += 1;
        }
        (self.get_best_move().unwrap(), score)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::new())
    }
}
