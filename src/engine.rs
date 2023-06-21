use super::*;
use EntryFlag::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GoCommand {
    Infinite,
    MoveTime(Duration),
    Depth(Depth),
    // Nodes(usize),
    // Mate(usize),
    // Ponder,
    // SearchMoves(Vec<Move>),
    Timed {
        wtime: Duration,
        btime: Duration,
        winc: Duration,
        binc: Duration,
        moves_to_go: Option<u32>,
    },
}

impl GoCommand {
    pub fn is_infinite(&self) -> bool {
        self == &Self::Infinite
    }

    pub fn is_move_time(&self) -> bool {
        matches!(self, Self::MoveTime(_))
    }

    pub fn is_depth(&self) -> bool {
        matches!(self, Self::Depth(_))
    }

    pub fn is_timed(&self) -> bool {
        matches!(self, Self::Timed { .. })
    }
}

pub struct Engine {
    pub board: Board,
    evaluator: Arc<Mutex<Evaluator>>,
    num_nodes_searched: usize,
    selective_depth: Ply,
    ply: Ply,
    pv_length: [usize; MAX_PLY],
    pv_table: [[Option<Move>; MAX_PLY]; MAX_PLY],
    move_sorter: MoveSorter,
    transposition_table: TranspositionTable,
    timer: Timer,
}

impl Engine {
    pub fn new(board: Board, evaluator: Arc<Mutex<Evaluator>>) -> Self {
        for square in *board.occupied() {
            let piece = board.piece_at(square).unwrap();
            let color = board.color_at(square).unwrap();
            evaluator
                .lock()
                .unwrap()
                .activate_nnue(piece, color, square);
        }
        Self {
            board,
            evaluator,
            num_nodes_searched: 0,
            selective_depth: 0,
            ply: 0,
            pv_length: [0; MAX_PLY],
            pv_table: [[None; MAX_PLY]; MAX_PLY],
            move_sorter: MoveSorter::default(),
            transposition_table: TranspositionTable::default(),
            timer: Timer::default(),
        }
    }

    fn push_nnue(&mut self, move_: Move) {
        self.evaluator.lock().unwrap().backup();
        let self_color = self.board.turn();
        let source = move_.get_source();
        let dest = move_.get_dest();
        let self_piece = self.board.piece_at(source).unwrap();
        self.evaluator
            .lock()
            .unwrap()
            .deactivate_nnue(self_piece, self_color, source);
        if self.board.is_capture(move_) {
            let remove_piece_square = if self.board.is_en_passant(move_) {
                dest.backward(self_color).unwrap()
            } else {
                dest
            };
            let piece = self.board.piece_at(remove_piece_square).unwrap();
            self.evaluator
                .lock()
                .unwrap()
                .deactivate_nnue(piece, !self_color, remove_piece_square);
        } else if self.board.is_castling(move_) {
            let (rook_source, rook_dest) = if move_.get_dest().get_file().to_index()
                > move_.get_source().get_file().to_index()
            {
                match self_color {
                    White => (Square::H1, Square::F1),
                    Black => (Square::H8, Square::F8),
                }
            } else {
                match self_color {
                    White => (Square::A1, Square::D1),
                    Black => (Square::A8, Square::D8),
                }
            };
            self.evaluator
                .lock()
                .unwrap()
                .deactivate_nnue(Rook, self_color, rook_source);
            self.evaluator
                .lock()
                .unwrap()
                .activate_nnue(Rook, self_color, rook_dest);
        }
        self.evaluator.lock().unwrap().activate_nnue(
            move_.get_promotion().unwrap_or(self_piece),
            self_color,
            dest,
        );
    }

    pub fn push(&mut self, optional_move: impl Into<Option<Move>>) {
        // if let Some(move_) = optional_move {
        //     self.push_nnue(move_);
        // }
        self.board.push(optional_move);
        self.ply += 1;
    }

    fn pop_nnue(&mut self) {
        self.evaluator.lock().unwrap().restore();
    }

    #[allow(clippy::let_and_return)]
    pub fn pop(&mut self) -> Option<Move> {
        let optional_move = self.board.pop();
        self.ply -= 1;
        // if let Some(move_) = optional_move {
        //     self.pop_nnue();
        // }
        optional_move
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
        self.transposition_table.reset_variables();
        // self.evaluator.lock().unwrap().reset_variables();
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), chess::Error> {
        for square in *self.board.occupied() {
            let piece = self.board.piece_at(square).unwrap();
            let color = self.board.color_at(square).unwrap();
            self.evaluator
                .lock()
                .unwrap()
                .deactivate_nnue(piece, color, square);
        }
        let result = self.board.set_fen(fen);
        for square in *self.board.occupied() {
            let piece = self.board.piece_at(square).unwrap();
            let color = self.board.color_at(square).unwrap();
            self.evaluator
                .lock()
                .unwrap()
                .activate_nnue(piece, color, square);
        }
        self.reset_variables();
        result
    }

    pub fn from_fen(fen: &str) -> Result<Self, chess::Error> {
        Ok(Engine::new(
            Board::from_fen(fen)?,
            Arc::new(Mutex::new(Evaluator::default())),
        ))
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
            "{} {} {} {} {} {} {} {} {} {} {:.3} s",
            colorize("info", INFO_STYLE),
            colorize("curr move", INFO_STYLE),
            self.board.stringify_move(curr_move).unwrap(),
            colorize("depth", INFO_STYLE),
            depth,
            colorize("score", INFO_STYLE),
            score_to_string(self.board.score_flipped(score)),
            colorize("nodes", INFO_STYLE),
            self.num_nodes_searched,
            colorize("time", INFO_STYLE),
            time_elapsed.as_secs_f64(),
        );
    }

    fn is_draw_move(&mut self, move_: Move) -> bool {
        self.board.gives_threefold_repetition(move_)
            || self.board.gives_claimable_threefold_repetition(move_)
    }

    fn get_sorted_root_node_moves(&mut self) -> Vec<(Move, MoveWeight)> {
        let mut moves_vec_sorted = self
            .move_sorter
            .get_weighted_moves_sorted(
                self.board.generate_legal_moves(),
                &self.board,
                self.ply,
                self.transposition_table.read_best_move(self.board.hash()),
                self.pv_table[0][self.ply],
                Evaluator::is_easily_winning_position(&self.board, self.board.get_material_score()),
            )
            .map(|WeightedMove { move_, .. }| {
                (
                    move_,
                    MoveSorter::score_root_moves(
                        &mut self.board,
                        &mut self.evaluator.lock().unwrap(),
                        move_,
                        self.pv_table[0][0],
                    ),
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
    ) -> Option<Score> {
        self.selective_depth = 0;
        let enable_timer = depth > 1;
        if self.timer.check_stop(enable_timer) {
            return None;
        }
        if self.board.is_game_over() {
            return if self.board.is_checkmate() {
                Some(-CHECKMATE_SCORE)
            } else {
                Some(0)
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
            if !is_endgame && self.is_draw_move(move_) && max_score > -DRAW_SCORE {
                continue;
            }
            let clock = Instant::now();
            self.push(move_);
            if move_index == 0
                || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, enable_timer)? > alpha
            {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, enable_timer)?;
                max_score = max_score.max(score);
            }
            self.pop();
            if self.timer.check_stop(enable_timer) {
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
                if (initial_alpha < score && score < initial_beta) || is_checkmate(score) {
                    self.update_pv_table(move_);
                }
                // self.update_pv_table(move_);
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, self.ply, beta, HashBeta, move_);
                    return Some(beta);
                }
            }
        }
        if !self.timer.check_stop(enable_timer) {
            self.transposition_table.write(
                key,
                depth,
                self.ply,
                alpha,
                flag,
                self.pv_table[self.ply][self.ply],
            );
        }
        Some(max_score)
    }

    fn get_lmr_reduction(depth: Depth, move_index: usize, is_pv_node: bool) -> Depth {
        let mut reduction =
            LMR_BASE_REDUCTION + (depth as f64).ln() * (move_index as f64).ln() / LMR_MOVE_DIVIDER;
        // let mut reduction = (depth as f64 - 1.0).max(0.0).sqrt() + (move_index as f64 - 1.0).max(0.0).sqrt();
        if is_pv_node {
            // reduction /= 3.0;
            reduction *= 2.0 / 3.0;
        }
        reduction.round() as Depth
    }

    fn alpha_beta(
        &mut self,
        mut depth: Depth,
        mut alpha: Score,
        mut beta: Score,
        mut enable_timer: bool,
    ) -> Option<Score> {
        self.pv_length[self.ply] = self.ply;
        if self.board.is_other_draw() {
            return Some(0);
        }
        let mate_score = CHECKMATE_SCORE - self.ply as Score;
        // // mate distance pruning
        // alpha = alpha.max(-mate_score);
        // beta = beta.min(mate_score - 1);
        // if alpha >= beta {
        //     return Some(alpha);
        // }
        // mate distance pruning
        if mate_score < beta {
            beta = mate_score;
            if alpha >= mate_score {
                return Some(mate_score);
            }
        }
        let checkers = self.board.get_checkers();
        let min_depth = if self.move_sorter.is_following_pv() {
            1
        } else {
            0
        };
        depth = (depth + checkers.popcnt() as Depth).max(min_depth);
        let is_pv_node = alpha != beta - 1;
        let key = self.board.hash();
        let best_move = if is_pv_node {
            self.transposition_table.read_best_move(key)
        } else {
            match self
                .transposition_table
                .read(key, depth, self.ply, alpha, beta)
            {
                (Some(score), _) => return Some(score),
                (None, best_move) => best_move,
            }
        };
        if self.ply == MAX_PLY - 1 {
            return Some(self.board.evaluate_flipped());
        }
        enable_timer &= depth > 3;
        if self.timer.check_stop(enable_timer) {
            return None;
        }
        if depth == 0 {
            return Some(self.quiescence(alpha, beta));
        }
        // let is_endgame = self.board.is_endgame();
        self.num_nodes_searched += 1;
        let not_in_check = checkers == BB_EMPTY;
        if not_in_check && !DISABLE_ALL_PRUNINGS {
            // static evaluation
            let static_evaluation = self.board.evaluate_flipped();
            if depth < 3 && !is_pv_node && !is_checkmate(beta) {
                let eval_margin = ((6 * PAWN_VALUE) / 5) * depth as Score;
                let new_score = static_evaluation - eval_margin;
                if new_score >= beta {
                    return Some(new_score);
                }
            }
            // null move pruning
            if depth >= NULL_MOVE_MIN_DEPTH
                && static_evaluation >= beta
                && self.board.has_non_pawn_material()
            {
                let r = NULL_MOVE_MIN_REDUCTION
                    + (depth.abs_diff(NULL_MOVE_MIN_DEPTH) as f64 / NULL_MOVE_DEPTH_DIVIDER as f64)
                        .round() as Depth;
                self.push(None);
                let score = -self.alpha_beta(depth - 1 - r, -beta, -beta + 1, enable_timer)?;
                self.pop();
                if self.timer.check_stop(enable_timer) {
                    return None;
                }
                if score >= beta {
                    return Some(beta);
                }
            }
            // razoring
            let d = 3;
            if !is_pv_node && depth <= d && !self.board.is_endgame() {
                let mut score = static_evaluation + (5 * PAWN_VALUE) / 4;
                if score < beta {
                    if depth == 1 {
                        let new_score = self.quiescence(alpha, beta);
                        return Some(new_score.max(score));
                    }
                    score += (7 * PAWN_VALUE) / 4;
                    if score < beta && depth < d {
                        let new_score = self.quiescence(alpha, beta);
                        if new_score < beta {
                            return Some(new_score.max(score));
                        }
                    }
                }
            }
        }
        let mut flag = HashAlpha;
        let moves_gen = self.board.generate_legal_moves();
        let weighted_moves = self.move_sorter.get_weighted_moves_sorted(
            moves_gen,
            &self.board,
            self.ply,
            best_move,
            self.pv_table[0][self.ply],
            Evaluator::is_easily_winning_position(&self.board, self.board.get_material_score()),
        );
        let mut move_index = 0;
        for WeightedMove { move_, .. } in weighted_moves {
            let not_capture_move = !self.board.is_capture(move_);
            let mut safe_to_apply_lmr = move_index >= FULL_DEPTH_SEARCH_LMR
                && depth >= REDUCTION_LIMIT_LMR
                && !DISABLE_LMR
                && not_capture_move
                && not_in_check
                && move_.get_promotion().is_none()
                && !self.move_sorter.is_killer_move(move_, self.ply)
                && !self.board.is_passed_pawn(move_.get_source());
            self.push(move_);
            safe_to_apply_lmr &= !self.board.is_check();
            let mut score: Score;
            if move_index == 0 {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, enable_timer)?;
            } else {
                if safe_to_apply_lmr {
                    let lmr_reduction = Self::get_lmr_reduction(depth, move_index, is_pv_node);
                    score = if depth > lmr_reduction {
                        -self.alpha_beta(
                            depth - 1 - lmr_reduction,
                            -alpha - 1,
                            -alpha,
                            enable_timer,
                        )?
                    } else {
                        alpha + 1
                    }
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    score = -self.alpha_beta(depth - 1, -alpha - 1, -alpha, enable_timer)?;
                    if score > alpha && score < beta {
                        score = -self.alpha_beta(depth - 1, -beta, -alpha, enable_timer)?;
                    }
                }
            }
            self.pop();
            if self.timer.check_stop(enable_timer) {
                return None;
            }
            if score > alpha {
                flag = HashExact;
                self.update_pv_table(move_);
                alpha = score;
                if not_capture_move {
                    self.move_sorter.add_history_move(move_, &self.board, depth);
                }
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, self.ply, beta, HashBeta, move_);
                    if not_capture_move {
                        self.move_sorter.update_killer_moves(move_, self.ply);
                    }
                    return Some(beta);
                }
            }
            move_index += 1;
        }
        if move_index == 0 {
            return Some(if not_in_check { 0 } else { -mate_score });
        }
        if !self.timer.check_stop(false) {
            self.transposition_table.write(
                key,
                depth,
                self.ply,
                alpha,
                flag,
                self.pv_table[self.ply][self.ply],
            );
        }
        Some(alpha)
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        if self.ply == MAX_PLY - 1 {
            return self.board.evaluate_flipped();
        }
        self.pv_length[self.ply] = self.ply;
        if self.board.is_other_draw() {
            return 0;
        }
        self.selective_depth = self.ply.max(self.selective_depth);
        self.num_nodes_searched += 1;
        let evaluation = self.board.evaluate_flipped();
        if evaluation >= beta {
            return beta;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        let key = self.board.hash();
        for WeightedMove { move_, weight } in self.move_sorter.get_weighted_capture_moves_sorted(
            self.board.generate_legal_captures(),
            self.transposition_table.read_best_move(key),
            &self.board,
        ) {
            if weight.is_negative() {
                break;
            }
            self.push(Some(move_));
            let score = -self.quiescence(-beta, -alpha);
            self.pop();
            if score >= beta {
                return beta;
            }
            // delta pruning
            let mut delta = evaluate_piece(Queen);
            if let Some(piece) = move_.get_promotion() {
                delta += evaluate_piece(piece) - PAWN_VALUE;
            }
            if score + delta < alpha {
                return alpha;
            }
            if score > alpha {
                self.update_pv_table(move_);
                alpha = score;
            }
        }
        alpha
    }

    fn get_pv(&self, ply: Ply) -> Vec<Move> {
        self.pv_table[ply][0..self.pv_length[ply]]
            .iter()
            .map(|optional_move| optional_move.unwrap_or_default())
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
                board.algebraic_and_push(move_, long).unwrap()
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
        if is_in_uci_mode() {
            self.get_pv_as_uci(0)
        } else {
            self.get_pv_as_san(0)
        }
    }

    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.pv_table[0][0]
    }

    pub fn get_ponder_move(&self) -> Option<Move> {
        self.pv_table[0][1]
    }

    pub fn print_warning_message(
        &self,
        current_depth: Depth,
        mut alpha: Score,
        mut beta: Score,
        mut score: Score,
    ) {
        if !is_in_uci_mode() {
            alpha = self.board.score_flipped(alpha);
            beta = self.board.score_flipped(beta);
            score = self.board.score_flipped(score);
        }
        let warning_message = format!(
            "Resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {:.3} s",
            current_depth,
            score_to_string(alpha),
            score_to_string(beta),
            score_to_string(score),
            self.timer.elapsed().as_secs_f64(),
        );
        println!("{}", colorize(warning_message, WARNING_MESSAGE_STYLE));
    }

    pub fn print_search_info(
        &self,
        current_depth: Depth,
        mut score: Score,
        time_elapsed: Duration,
    ) {
        if !is_in_uci_mode() {
            score = self.board.score_flipped(score);
        }
        let style = SUCCESS_MESSAGE_STYLE;
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {:.2} {} {} {} {:.3} {} {}",
            colorize("info depth", style),
            current_depth,
            colorize("seldepth", style),
            self.selective_depth,
            colorize("score", style),
            score_to_string(score),
            colorize("nodes", style),
            self.num_nodes_searched,
            colorize("nps", style),
            (self.num_nodes_searched as u128 * 10_u128.pow(9)) / time_elapsed.as_nanos(),
            colorize("hashfull", style),
            self.transposition_table.get_hash_full(),
            colorize("collisions", style),
            self.transposition_table.get_num_collisions(),
            colorize("time", style),
            time_elapsed.as_secs_f64(),
            colorize("pv", style),
            self.get_pv_string(),
        );
    }

    pub fn go(&mut self, mut command: GoCommand, print_info: bool) -> (Option<Move>, Score) {
        self.reset_variables();
        self.timer.start_communication_check();
        if command.is_timed() {
            command = self.parse_timed_command(command);
        }
        if let GoCommand::MoveTime(duration) = command {
            self.timer.set_max_time(duration);
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
            let curr_board_ply = self.board.get_ply();
            score = self
                .search(current_depth, alpha, beta, print_info)
                .unwrap_or(prev_score);
            for _ in 0..curr_board_ply.abs_diff(self.board.get_ply()) {
                if command.is_depth() {
                    panic!("Something went wrong with the search!");
                }
                self.pop();
            }
            let time_elapsed = self.timer.elapsed();
            if print_info {
                self.print_search_info(current_depth, score, time_elapsed);
            }
            if self.timer.check_stop(true) {
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
            let cutoff = if is_checkmate(score) {
                10
            } else {
                ASPIRATION_WINDOW_CUTOFF
            };
            alpha = score - cutoff;
            beta = score + cutoff;
            if command == GoCommand::Depth(current_depth) {
                break;
            }
            current_depth += 1;
        }
        self.timer.stop_communication_check();
        (self.get_best_move(), self.board.score_flipped(score))
    }

    pub fn parse_timed_command(&self, command: GoCommand) -> GoCommand {
        if let GoCommand::Timed {
            wtime,
            btime,
            winc,
            binc,
            moves_to_go,
        } = command
        {
            let (time, inc) = match self.board.turn() {
                White => (wtime, winc),
                Black => (btime, binc),
            };
            let divider = moves_to_go.unwrap_or(30);
            let new_inc = inc
                .checked_sub(Duration::from_millis(1000))
                .unwrap_or(Duration::from_millis(0));
            let search_time = (time / divider + new_inc)
                .checked_sub(Duration::from_millis(2000))
                .unwrap_or(Duration::from_millis(0));
            // let multiplier = match_interpolate!(0.5, 1, 32, 2, self.board.get_num_pieces());
            // let search_time = Duration::from_secs_f64(search_time.as_secs_f64() * multiplier);
            GoCommand::MoveTime(search_time)
        } else {
            panic!("Expected Timed Command!")
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        let evaluator = Arc::new(Mutex::new(Evaluator::default()));
        Self::new(Board::new(evaluator.clone()), evaluator)
    }
}
