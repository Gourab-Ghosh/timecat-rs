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

    pub fn push(&mut self, option_move: Option<Move>) {
        self.board.push(option_move);
        self.ply += 1;
    }

    pub fn pop(&mut self) -> Option<Move> {
        self.ply -= 1;
        self.board.pop()
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
    }

    fn update_pv_table(&mut self, _move: Move) {
        self.pv_table[self.ply][self.ply] = Some(_move);
        for next_ply in (self.ply + 1)..self.pv_length[self.ply + 1] {
            self.pv_table[self.ply][next_ply] = self.pv_table[self.ply + 1][next_ply];
        }
        self.pv_length[self.ply] = self.pv_length[self.ply + 1];
    }

    pub fn can_apply_lmr(&self, _move: Move) -> bool {
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
        (LMR_BASE_REDUCTION + (depth as f32).ln() * (move_index as f32).ln() / LMR_MOVE_DIVIDER)
            .round() as Depth
    }

    fn search(
        &mut self,
        depth: Depth,
        mut alpha: Score,
        beta: Score,
        print_move_info: bool,
    ) -> Score {
        self.pv_length[self.ply] = self.ply;
        if self.board.is_game_over() {
            return if self.board.is_checkmate() {
                -CHECKMATE_SCORE
            } else {
                0
            };
        }
        let key = self.board.hash();
        let mut score = -CHECKMATE_SCORE;
        let mut flag = HashAlpha;
        let mut moves_vec_sorted = Vec::from_iter(self.move_sorter.get_weighted_sort_moves(
            self.board.generate_legal_moves(),
            &self.board,
            self.ply,
            self.transposition_table.read_best_move(key),
            self.pv_table[0][self.ply],
        ));
        moves_vec_sorted
            .sort_by_cached_key(|&wm| self.board.gives_claimable_threefold_repetition(wm._move));
        moves_vec_sorted.sort_by_cached_key(|&wm| self.board.gives_repetition(wm._move));
        for (move_index, weighted_move) in moves_vec_sorted.iter().enumerate() {
            let _move = weighted_move._move;
            let clock = Instant::now();
            let repetition_draw_possible = self.board.gives_repetition(_move)
                || self.board.gives_claimable_threefold_repetition(_move);
            self.push(Some(_move));
            if !self.board.is_endgame() && repetition_draw_possible && score > -DRAW_SCORE {
                self.pop();
                continue;
            }
            let check_extension_depth = match_interpolate!(
                0,
                0.7 * depth as f32,
                INITIAL_MATERIAL_SCORE_ABS,
                evaluate_piece(Bishop),
                self.board.get_material_score_abs()
            )
            .round() as Depth;
            if move_index == 0 || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, check_extension_depth, true) > alpha {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, check_extension_depth, true);
            }
            self.pop();
            if print_move_info {
                let time_elapsed = clock.elapsed();
                if time_elapsed.as_secs_f32() > PRINT_MOVE_INFO_TIME_THRESHOLD {
                    println!(
                        "{} {} {} {} {} {} {} {} {} {:.3} s",
                        colorize("curr move", INFO_STYLE),
                        self.board.san(_move).unwrap(),
                        colorize("depth", INFO_STYLE),
                        depth,
                        colorize("score", INFO_STYLE),
                        score_to_string(if self.board.turn() == White {
                            score
                        } else {
                            -score
                        }),
                        colorize("nodes", INFO_STYLE),
                        self.num_nodes_searched,
                        colorize("time", INFO_STYLE),
                        time_elapsed.as_secs_f32(),
                    );
                }
            }
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
        );
        alpha
    }

    fn alpha_beta(
        &mut self,
        mut depth: Depth,
        mut alpha: Score,
        mut beta: Score,
        check_extension_depth: Depth,
        apply_null_move: bool,
    ) -> Score {
        self.pv_length[self.ply] = self.ply;
        let num_pieces = self.board.get_num_pieces();
        let is_endgame = num_pieces <= ENDGAME_PIECE_THRESHOLD;
        let not_in_check = !self.board.is_check();
        let is_pvs_node = alpha != beta - 1;
        let mate_score = CHECKMATE_SCORE - self.ply as Score;
        let moves_gen = self.board.generate_legal_moves();
        if moves_gen.len() == 0 {
            return if not_in_check { 0 } else { -mate_score };
        }
        let num_repetitions = self.board.get_num_repetitions();
        if num_repetitions > 1 || self.board.is_other_draw() {
            return 0;
        }
        // if !not_in_check && !is_pvs_node && depth < 3 && !is_endgame && num_pieces > 4 {
        if !not_in_check && depth < check_extension_depth {
            depth += 1
        }
        depth = depth.max(0);
        let key = self.board.hash();
        let best_move = if is_pvs_node {
            self.transposition_table.read_best_move(key)
        } else {
            match self.transposition_table.read(key, depth, alpha, beta) {
                (Some(score), _) => return score,
                (None, best_move) => best_move,
            }
        };
        // mate distance pruning
        alpha = alpha.max(-mate_score);
        beta = beta.min(mate_score - 1);
        if alpha >= beta {
            return alpha;
        }
        if depth == 0 {
            return self.quiescence(alpha, beta);
        }
        self.num_nodes_searched += 1;
        let mut futility_pruning = false;
        if not_in_check && !is_pvs_node {
            // static evaluation pruning
            let static_evaluation = self.board.evaluate_flipped();
            if depth < 3 && (beta - 1).abs() > -mate_score + PAWN_VALUE {
                let evaluation_margin = PAWN_VALUE * depth as i16;
                let evaluation_diff = static_evaluation - evaluation_margin;
                if evaluation_diff >= beta {
                    return evaluation_diff;
                }
            }
            // null move pruning
            if apply_null_move && static_evaluation > beta {
                let r = NULL_MOVE_MIN_REDUCTION
                    + (depth - NULL_MOVE_MIN_DEPTH) / NULL_MOVE_DEPTH_DIVIDER;
                if depth > r {
                    self.push(None);
                    let score = -self.alpha_beta(
                        depth - 1 - r,
                        -beta,
                        -beta + 1,
                        check_extension_depth,
                        false,
                    );
                    self.pop();
                    if score >= beta {
                        return beta;
                    }
                }
                // razoring
                if !is_pvs_node && self.board.has_non_pawn_material() && depth < 4 {
                    let mut evaluation = static_evaluation + PAWN_VALUE;
                    if evaluation < beta {
                        if depth == 1 {
                            let new_evaluation = self.quiescence(alpha, beta);
                            return new_evaluation.max(evaluation);
                        }
                        evaluation += PAWN_VALUE;
                        if evaluation < beta && depth < 4 {
                            let new_evaluation = self.quiescence(alpha, beta);
                            if new_evaluation < beta {
                                return new_evaluation.max(evaluation);
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
                not_capture_move && not_in_check && !is_pvs_node && self.can_apply_lmr(_move);
            self.push(Some(_move));
            let mut score: Score;
            if move_index == 0 {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, check_extension_depth, true);
            } else {
                if move_index >= FULL_DEPTH_SEARCH_LMR
                    && depth >= REDUCTION_LIMIT_LMR
                    && safe_to_apply_lmr
                    && !DISABLE_LMR
                {
                    let lmr_reduction = self.get_lmr_reduction(depth, move_index);
                    score = -self.alpha_beta(
                        depth - 1 - lmr_reduction,
                        -alpha - 1,
                        -alpha,
                        check_extension_depth,
                        true,
                    );
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    score = -self.alpha_beta(
                        depth - 1,
                        -alpha - 1,
                        -alpha,
                        check_extension_depth,
                        true,
                    );
                    if score > alpha && score < beta {
                        score =
                            -self.alpha_beta(depth - 1, -beta, -alpha, check_extension_depth, true);
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
                    );
                    if not_capture_move {
                        self.move_sorter.update_killer_moves(_move, self.ply);
                    }
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
        );
        alpha
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        self.pv_length[self.ply] = self.ply;
        if self.board.is_game_over() {
            if self.board.status() == BoardStatus::Checkmate {
                return -CHECKMATE_SCORE + self.ply as Score;
            }
            return 0;
        }
        self.num_nodes_searched += 1;
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
            if weighted_move.weight.is_negative() {
                break;
            }
            self.push(Some(weighted_move._move));
            let score = -self.quiescence(-beta, -alpha);
            self.pop();
            if score > alpha {
                self.update_pv_table(weighted_move._move);
                alpha = score;
                if score >= beta {
                    return beta;
                }
            }
        }
        alpha
    }

    fn get_pv(&self, ply: Ply) -> Vec<Move> {
        let mut pv = Vec::new();
        for i in 0..self.pv_length[ply as usize] {
            pv.push(self.pv_table[ply as usize][i].unwrap_or_default());
        }
        pv
    }

    fn get_pv_as_uci(&self, ply: Ply) -> String {
        let mut pv_string = String::new();
        for _move in self.get_pv(ply) {
            pv_string.push_str(&_move.to_string());
            pv_string.push(' ');
        }
        return pv_string.trim().to_string();
    }

    fn get_pv_as_algebraic(&self, ply: Ply, long: bool) -> String {
        let mut board = self.board.clone();
        let mut pv_string = String::new();
        for _move in self.get_pv(ply) {
            pv_string += &(if board.is_legal(_move) {
                board.san_and_push(_move).unwrap()
            } else {
                colorize(_move, ERROR_MESSAGE_STYLE)
            } + " ");
        }
        return pv_string.trim().to_string();
    }

    fn get_pv_as_san(&self, ply: Ply) -> String {
        self.get_pv_as_algebraic(ply, false)
    }

    fn get_pv_as_lan(&self, ply: Ply) -> String {
        self.get_pv_as_algebraic(ply, true)
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

    pub fn go(&mut self, depth: Depth, print_info: bool) -> (Move, Score) {
        self.reset_variables();
        // if !self.board.is_endgame() {
        //     self.transposition_table.clear();
        // }
        self.transposition_table.clear();
        let mut current_depth = 1;
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut score;
        let clock = Instant::now();
        loop {
            if FOLLOW_PV {
                self.move_sorter.follow_pv();
            }
            score = self.search(current_depth, alpha, beta, print_info);
            let time_passed = clock.elapsed();
            if score <= alpha || score >= beta {
                if print_info {
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
            if print_info {
                self.print_search_info(current_depth, score, time_passed);
            }
            if current_depth == depth {
                break;
            }
            current_depth += 1;
        }
        let best_move = self.get_best_move().unwrap();
        (best_move, score)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::new())
    }
}
