use super::*;
use EntryFlag::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GoCommand {
    Infinite,
    Time(Duration),
    Depth(Depth),
}

pub struct Engine {
    pub board: Board,
    num_nodes_searched: usize,
    ply: Ply,
    pv_length: [usize; MAX_PLY],
    pv_table: [[Option<Move>; MAX_PLY]; MAX_PLY],
    move_sorter: MoveSorter,
    transposition_table: TranspositionTable,
    timer: Timer,
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
            timer: Timer::new(),
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

    pub fn set_fen(&mut self, fen: &str) {
        self.board.set_fen(fen);
        self.reset_variables()
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
        self.timer.reset_variables();
        self.transposition_table.clear();
    }

    fn update_pv_table(&mut self, move_: Move) {
        self.pv_table[self.ply][self.ply] = Some(move_);
        for next_ply in (self.ply + 1)..self.pv_length[self.ply + 1] {
            self.pv_table[self.ply][next_ply] = self.pv_table[self.ply + 1][next_ply];
        }
        self.pv_length[self.ply] = self.pv_length[self.ply + 1];
    }

    fn print_root_node_info(
        &self,
        curr_move: Move,
        depth: Depth,
        score: Score,
        time_elapsed: Duration,
    ) {
        println!(
            "{} {} {} {} {} {} {} {} {} {:.3} s",
            colorize("curr move", INFO_STYLE),
            self.board.san(Some(curr_move)).unwrap(),
            colorize("depth", INFO_STYLE),
            depth,
            colorize("score", INFO_STYLE),
            score_to_string(self.board.score_flipped(score)),
            colorize("nodes", INFO_STYLE),
            self.num_nodes_searched,
            colorize("time", INFO_STYLE),
            time_elapsed.as_secs_f32(),
        );
    }

    fn is_draw_move(&mut self, move_: Move) -> bool {
        self.board.gives_threefold_repetition(move_)
            || self.board.gives_claimable_threefold_repetition(move_)
    }

    fn get_sorted_root_node_moves(&mut self) -> Vec<(Move, MoveWeight)> {
        let mut moves_vec_sorted = self
            .move_sorter
            .get_weighted_sort_moves(
                self.board.generate_legal_moves(),
                &self.board,
                self.ply,
                self.transposition_table.read_best_move(self.board.hash()),
                self.pv_table[0][self.ply],
            )
            .map(|wm| {
                (
                    wm.move_,
                    MoveSorter::score_root_moves(&mut self.board, wm.move_),
                )
            })
            .collect_vec();
        moves_vec_sorted.sort_by_key(|&t| -t.1);
        moves_vec_sorted
    }

    fn search(
        &mut self,
        depth: Depth,
        mut alpha: Score,
        beta: Score,
        print_move_info: bool,
    ) -> Score {
        if self.timer.stop_search() || self.timer.is_time_up() {
            return 0;
        }
        if self.board.is_game_over() {
            return if self.board.is_checkmate() {
                -CHECKMATE_SCORE
            } else {
                0
            };
        }
        let key = self.board.hash();
        let initial_alpha = alpha;
        let initial_beta = beta;
        let mut score = -CHECKMATE_SCORE;
        let mut max_score = score;
        let mut flag = HashAlpha;
        let is_endgame = self.board.is_endgame();
        for (move_index, &(move_, _)) in self.get_sorted_root_node_moves().iter().enumerate() {
            // let san = self.board.san(move_).unwrap();
            if !is_endgame && self.is_draw_move(move_) && max_score > -DRAW_SCORE {
                continue;
            }
            let clock = Instant::now();
            self.push(Some(move_));
            if move_index == 0 || -self.alpha_beta(depth - 1, -alpha - 1, -alpha) > alpha {
                score = -self.alpha_beta(depth - 1, -beta, -alpha);
                max_score = max_score.max(score);
                // println!("{} {}", san, score_to_string(score));
            }
            self.pop();
            if self.timer.stop_search() {
                break;
            }
            if print_move_info {
                let time_elapsed = clock.elapsed();
                if time_elapsed > PRINT_MOVE_INFO_DURATION_THRESHOLD {
                    self.print_root_node_info(move_, depth, score, time_elapsed)
                }
            }
            if score > alpha {
                flag = HashExact;
                alpha = score;
                if initial_alpha < score && score < initial_beta {
                    self.update_pv_table(move_);
                }
                if score >= beta {
                    self.transposition_table.write(
                        key,
                        depth,
                        self.ply,
                        beta,
                        HashBeta,
                        Some(move_),
                    );
                    return beta;
                }
            }
        }
        if !self.timer.stop_search() {
            self.transposition_table.write(
                key,
                depth,
                self.ply,
                alpha,
                flag,
                self.pv_table[self.ply][self.ply],
            );
        }
        max_score
    }

    fn get_lmr_reduction(depth: Depth, move_index: usize, is_pv_node: bool) -> Depth {
        let mut reduction =
            LMR_BASE_REDUCTION + (depth as f32).ln() * (move_index as f32).ln() / LMR_MOVE_DIVIDER;
        // let mut reduction = (depth as f32 - 1.0).max(0.0).sqrt() + (move_index as f32 - 1.0).max(0.0).sqrt();
        if is_pv_node {
            reduction /= 3.0;
            // reduction *= 2.0;
        }
        reduction.round() as Depth
    }

    fn alpha_beta(&mut self, mut depth: Depth, mut alpha: Score, mut beta: Score) -> Score {
        if self.ply == MAX_PLY || self.timer.stop_search() || self.timer.is_time_up() {
            return 0;
        }
        self.pv_length[self.ply] = self.ply;
        // let num_pieces = self.board.get_num_pieces();
        // let is_endgame = num_pieces <= ENDGAME_PIECE_THRESHOLD;
        let not_in_check = !self.board.is_check();
        let is_pv_node = alpha != beta - 1;
        let mate_score = CHECKMATE_SCORE - self.ply as Score;
        let moves_gen = self.board.generate_legal_moves();
        if moves_gen.len() == 0 {
            return if not_in_check { 0 } else { -mate_score };
        }
        if self.board.is_other_draw() {
            return 0;
        }
        if !not_in_check {
            depth += 1
        }
        depth = depth.max(0);
        let key = self.board.hash();
        let best_move = if is_pv_node {
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
        if not_in_check && !DISABLE_ALL_PRUNINGS {
            // static evaluation
            let static_evaluation = self.board.evaluate_flipped();

            if depth < 3 && !is_pv_node && beta.abs_diff(1) as Score > -INFINITY + PAWN_VALUE {
                let eval_margin = ((6 * PAWN_VALUE) / 5) * depth as Score;
                let new_score = static_evaluation - eval_margin;
                if new_score >= beta {
                    return new_score;
                }
            }

            // if depth <= 8 && !is_pv_node && !is_checkmate(beta) {
            //     let eval_margin = ((6 * PAWN_VALUE) / 5) * depth as Score;
            //     if static_evaluation - eval_margin >= beta {
            //         return static_evaluation - eval_margin;
            //     }
            // }

            // null move pruning
            if depth >= NULL_MOVE_MIN_DEPTH
                && self.board.has_non_pawn_material()
                && static_evaluation >= beta
            {
                let r = NULL_MOVE_MIN_REDUCTION
                    + (depth.abs_diff(NULL_MOVE_MIN_DEPTH) as f32 / NULL_MOVE_DEPTH_DIVIDER as f32)
                        .round() as Depth;
                if depth > r {
                    self.push(None);
                    let score = -self.alpha_beta(depth - 1 - r, -beta, -beta + 1);
                    self.pop();
                    if self.timer.stop_search() {
                        return 0;
                    }
                    if score >= beta {
                        return beta;
                    }
                }
            }
            // // razoring
            // if !is_pv_node && depth <= 3 && is_endgame {
            //     let mut score = static_evaluation + (5 * PAWN_VALUE) / 4;
            //     if score < beta {
            //         if depth == 1 {
            //             let new_score = self.quiescence(alpha, beta);
            //             return new_score.max(score);
            //         }
            //         score += (7 * PAWN_VALUE) / 4;
            //         if score < beta && depth <= 2 {
            //             let new_score = self.quiescence(alpha, beta);
            //             if new_score < beta {
            //                 return new_score.max(score);
            //             }
            //         }
            //     }
            // }
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
            let move_ = weighted_move.move_;
            let not_capture_move = !self.board.is_capture(move_);
            let mut safe_to_apply_lmr = not_capture_move
                && not_in_check
                && depth >= 3
                && move_.get_promotion().is_none()
                && !self.move_sorter.is_killer_move(move_, self.ply)
                && !self.board.is_passed_pawn(move_.get_source());
            self.push(Some(move_));
            safe_to_apply_lmr &= !self.board.is_check();
            let mut score: Score;
            if move_index == 0 {
                score = -self.alpha_beta(depth - 1, -beta, -alpha);
            } else {
                if move_index >= FULL_DEPTH_SEARCH_LMR
                    && depth >= REDUCTION_LIMIT_LMR
                    && safe_to_apply_lmr
                    && !DISABLE_LMR
                {
                    let lmr_reduction = Self::get_lmr_reduction(depth, move_index, is_pv_node);
                    if depth > lmr_reduction {
                        score = -self.alpha_beta(depth - 1 - lmr_reduction, -alpha - 1, -alpha);
                    } else {
                        score = alpha + 1;
                    }
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    score = -self.alpha_beta(depth - 1, -alpha - 1, -alpha);
                    if score > alpha && score < beta {
                        score = -self.alpha_beta(depth - 1, -beta, -alpha);
                    }
                }
            }
            self.pop();
            if self.timer.stop_search() {
                return 0;
            }
            if score > alpha {
                flag = HashExact;
                self.update_pv_table(move_);
                alpha = score;
                if not_capture_move {
                    self.move_sorter.add_history_move(move_, &self.board, depth);
                }
                if score >= beta {
                    self.transposition_table.write(
                        key,
                        depth,
                        self.ply,
                        beta,
                        HashBeta,
                        Some(move_),
                    );
                    if not_capture_move {
                        self.move_sorter.update_killer_moves(move_, self.ply);
                    }
                    return beta;
                }
            }
        }
        if !self.timer.stop_search() {
            self.transposition_table.write(
                key,
                depth,
                self.ply,
                alpha,
                flag,
                self.pv_table[self.ply][self.ply],
            );
        }
        alpha
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        if self.ply == MAX_PLY || self.timer.stop_search() || self.timer.is_time_up() {
            return 0;
        }
        self.pv_length[self.ply] = self.ply;
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
            self.push(Some(weighted_move.move_));
            let score = -self.quiescence(-beta, -alpha);
            self.pop();
            if self.timer.stop_search() {
                return 0;
            }
            if score > alpha {
                self.update_pv_table(weighted_move.move_);
                alpha = score;
                if score >= beta {
                    return beta;
                }
            }
        }
        alpha
    }

    fn get_pv(&self, ply: Ply) -> Vec<Move> {
        self.pv_table[ply][0..self.pv_length[ply]]
            .iter()
            .map(|option_move| option_move.unwrap_or_default())
            .collect_vec()
    }

    fn get_pv_as_uci(&self, ply: Ply) -> String {
        let mut pv_string = String::new();
        for move_ in self.get_pv(ply) {
            pv_string.push_str(&move_.to_string());
            pv_string.push(' ');
        }
        return pv_string.trim().to_string();
    }

    fn get_pv_as_algebraic(&self, ply: Ply, long: bool) -> String {
        let mut board = self.board.clone();
        let mut pv_string = String::new();
        for move_ in self.get_pv(ply) {
            pv_string += &(if board.is_legal(move_) {
                board.algebraic_and_push(Some(move_), long).unwrap()
            } else {
                colorize(move_, ERROR_MESSAGE_STYLE)
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

    pub fn print_warning_message(
        &self,
        current_depth: Depth,
        alpha: Score,
        beta: Score,
        score: Score,
    ) {
        let warning_message = format!(
            "Resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {:.3} s",
            current_depth,
            score_to_string(alpha),
            score_to_string(beta),
            score_to_string(score),
            self.timer.elapsed().as_secs_f32(),
        );
        println!("{}", colorize(warning_message, WARNING_MESSAGE_STYLE));
    }

    pub fn print_search_info(&self, current_depth: Depth, score: Score, time_elapsed: Duration) {
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
            (self.num_nodes_searched as u128 * 10_u128.pow(9)) / time_elapsed.as_nanos(),
            colorize("time", style),
            time_elapsed.as_secs_f32(),
            colorize("pv", style),
            self.get_pv_string(),
        );
    }

    pub fn go(&mut self, command: GoCommand, print_info: bool) -> (Option<Move>, Score) {
        self.reset_variables();
        if let GoCommand::Time(duration) = command {
            self.timer.set_max_time(Some(duration));
        }
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut score = 0;
        let mut current_depth = 1;
        while current_depth < Depth::MAX {
            if FOLLOW_PV {
                self.move_sorter.follow_pv();
            }
            let prev_score = score;
            score = self.search(current_depth, alpha, beta, print_info);
            let time_elapsed = self.timer.elapsed();
            if print_info {
                self.print_search_info(
                    current_depth,
                    self.board.score_flipped(score),
                    time_elapsed,
                );
            }
            if self.timer.stop_search() {
                if score == 0 {
                    score = prev_score;
                }
                break;
            }
            if score <= alpha || score >= beta {
                if print_info {
                    self.print_warning_message(current_depth, alpha, beta, score);
                }
                alpha = -INFINITY;
                beta = INFINITY;
                continue;
            }
            alpha = score - ASPIRATION_WINDOW_CUTOFF;
            beta = score + ASPIRATION_WINDOW_CUTOFF;
            if command == GoCommand::Depth(current_depth) {
                break;
            }
            current_depth += 1;
        }
        (self.get_best_move(), self.board.score_flipped(score))
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::new())
    }
}
