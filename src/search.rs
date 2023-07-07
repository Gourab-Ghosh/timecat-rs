use super::*;
use EntryFlag::*;

#[derive(Clone)]
pub struct SearchInfo {
    pub board: Board,
    pub depth: Depth,
    pub seldepth: Ply,
    pub score: Score,
    pub nodes: usize,
    pub hash_full: f64,
    pub overwrites: usize,
    pub collisions: usize,
    pub clock: Instant,
    pub pv: Vec<Option<Move>>,
}

impl SearchInfo {
    pub fn new(searcher: &Searcher, current_depth: Depth) -> Self {
        Self {
            board: searcher.initial_board.clone(),
            depth: current_depth,
            seldepth: searcher.get_selective_depth(),
            score: searcher.get_score(),
            nodes: searcher.get_num_nodes_searched(),
            hash_full: searcher.get_hash_full(),
            overwrites: searcher.get_num_overwrites(),
            collisions: searcher.get_num_collisions(),
            clock: searcher.timer.get_clock(),
            pv: searcher.get_pv().to_vec(),
        }
    }

    pub fn get_pv_as_uci(pv: &[Option<Move>]) -> String {
        let mut pv_string = String::new();
        for move_ in pv {
            pv_string += &(move_.uci() + " ");
        }
        return pv_string.trim().to_string();
    }

    pub fn get_pv_as_algebraic(board: &Board, pv: &[Option<Move>], long: bool) -> String {
        let mut board = board.clone();
        let mut pv_string = String::new();
        for &move_ in pv {
            let is_legal_move = if let Some(move_) = move_ {
                board.is_legal(move_)
            } else {
                false
            };
            pv_string += &(if is_legal_move {
                board.algebraic_and_push(move_, long).unwrap()
            } else {
                colorize(move_.uci(), ERROR_MESSAGE_STYLE)
            } + " ");
        }
        return pv_string.trim().to_string();
    }

    pub fn get_pv_as_san(board: &Board, pv: &[Option<Move>]) -> String {
        Self::get_pv_as_algebraic(board, pv, false)
    }

    pub fn get_pv_as_lan(board: &Board, pv: &[Option<Move>]) -> String {
        Self::get_pv_as_algebraic(board, pv, true)
    }

    pub fn get_pv_string(board: &Board, pv: &[Option<Move>]) -> String {
        if is_in_uci_mode() {
            Self::get_pv_as_uci(pv)
        } else {
            Self::get_pv_as_algebraic(board, pv, use_long_algebraic_notation())
        }
    }

    pub fn get_time_elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    pub fn print_info(&self) {
        let mut score = self.score;
        if !is_in_uci_mode() {
            score = self.board.score_flipped(score);
        }
        let hashfull_string = if is_in_uci_mode() {
            (self.hash_full.round() as u8).to_string()
        } else {
            format!("{:.2}", self.hash_full)
        };
        let style = SUCCESS_MESSAGE_STYLE;
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            colorize("info depth", style),
            self.depth,
            colorize("seldepth", style),
            self.seldepth,
            colorize("score", style),
            score.stringify(),
            colorize("nodes", style),
            self.nodes,
            colorize("nps", style),
            (self.nodes as u128 * 10_u128.pow(9)) / self.get_time_elapsed().as_nanos(),
            colorize("hashfull", style),
            hashfull_string,
            colorize("overwrites", style),
            self.overwrites,
            colorize("collisions", style),
            self.collisions,
            colorize("time", style),
            self.get_time_elapsed().stringify(),
            colorize("pv", style),
            Self::get_pv_string(&self.board, &self.pv),
        );
    }

    pub fn print_warning_message(&self, mut alpha: Score, mut beta: Score) {
        let mut score = self.score;
        if !is_in_uci_mode() {
            alpha = self.board.score_flipped(alpha);
            beta = self.board.score_flipped(beta);
            score = self.board.score_flipped(score);
        }
        let warning_message = format!(
            "info string Resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {}",
            self.depth,
            alpha.stringify(),
            beta.stringify(),
            score.stringify(),
            self.get_time_elapsed().stringify(),
        );
        println!("{}", colorize(warning_message, WARNING_MESSAGE_STYLE));
    }
}

pub struct PVTable {
    length: [usize; MAX_PLY],
    table: [[Option<Move>; MAX_PLY]; MAX_PLY],
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            length: [0; MAX_PLY],
            table: [[None; MAX_PLY]; MAX_PLY],
        }
    }

    pub fn get_pv(&self, ply: Ply) -> &[Option<Move>] {
        &self.table[ply][0..self.length[ply]]
    }

    pub fn update_table(&mut self, ply: Ply, move_: Move) {
        self.table[ply][ply] = Some(move_);
        for next_ply in (ply + 1)..self.length[ply + 1] {
            self.table[ply][next_ply] = self.table[ply + 1][next_ply];
        }
        self.length[ply] = self.length[ply + 1];
    }

    #[inline(always)]
    pub fn set_length(&mut self, ply: Ply, length: usize) {
        self.length[ply] = length;
    }

    #[inline(always)]
    pub fn reset_variables(&mut self) {
        for ply in 0..MAX_PLY {
            self.length[ply] = 0;
        }
    }
}

impl Default for PVTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Searcher {
    id: usize,
    initial_board: Board,
    board: Board,
    pv_table: PVTable,
    move_sorter: MoveSorter,
    timer: Timer,
    num_nodes_searched: Arc<AtomicUsize>,
    selective_depth: Arc<AtomicUsize>,
    ply: Ply,
    score: Score,
}

impl Searcher {
    pub fn new(
        id: usize,
        board: Board,
        num_nodes_searched: Arc<AtomicUsize>,
        selective_depth: Arc<AtomicUsize>,
        stopper: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            initial_board: board.clone(),
            board,
            pv_table: PVTable::new(),
            move_sorter: MoveSorter::new(),
            timer: if id == 0 {
                Timer::new(stopper)
            } else {
                Timer::new_dummy()
            },
            num_nodes_searched,
            selective_depth,
            ply: 0,
            score: 0,
        }
    }

    #[inline(always)]
    fn is_main_threaded(&self) -> bool {
        self.id == 0
    }

    fn push(&mut self, optional_move: impl Into<Option<Move>>) {
        self.board.push(optional_move);
        self.ply += 1;
    }

    fn pop(&mut self) -> Option<Move> {
        self.ply -= 1;
        self.board.pop()
    }

    fn print_root_node_info(
        &self,
        board: &Board,
        curr_move: Move,
        depth: Depth,
        score: Score,
        time_elapsed: Duration,
    ) {
        println!(
            "{} {} {} {} {} {} {} {} {} {} {}",
            colorize("info", INFO_MESSAGE_STYLE),
            colorize("curr move", INFO_MESSAGE_STYLE),
            curr_move.stringify_move(board).unwrap(),
            colorize("depth", INFO_MESSAGE_STYLE),
            depth,
            colorize("score", INFO_MESSAGE_STYLE),
            board.score_flipped(score).stringify(),
            colorize("nodes", INFO_MESSAGE_STYLE),
            self.get_num_nodes_searched(),
            colorize("time", INFO_MESSAGE_STYLE),
            time_elapsed.stringify(),
        );
    }

    fn is_draw_move(&self, move_: Move) -> bool {
        self.board.gives_threefold_repetition(move_)
            || self.board.gives_claimable_threefold_repetition(move_)
    }

    fn get_sorted_root_node_moves(&mut self) -> Vec<(Move, MoveWeight)> {
        let mut moves_vec_sorted = self
            .move_sorter
            .get_weighted_moves_sorted(
                self.board.generate_legal_moves(),
                &self.board,
                0,
                TRANSPOSITION_TABLE.read_best_move(self.board.hash()),
                self.get_best_move(),
                Evaluator::is_easily_winning_position(&self.board, self.board.get_material_score()),
            )
            .map(|WeightedMove { move_, .. }| {
                (
                    move_,
                    MoveSorter::score_root_moves(&self.board, move_, self.get_best_move()),
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
        if FOLLOW_PV {
            self.move_sorter.follow_pv();
        }
        if self.is_main_threaded() {
            self.selective_depth.store(0, MEMORY_ORDERING);
        }
        let enable_timer = depth > 1 && self.is_main_threaded();
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
        let mut score = -CHECKMATE_SCORE;
        let mut max_score = score;
        let mut flag = HashAlpha;
        let is_endgame = self.board.is_endgame();
        let moves = self.get_sorted_root_node_moves();
        for (move_index, &(move_, _)) in moves.iter().enumerate() {
            if !is_endgame && self.is_draw_move(move_) && max_score > -DRAW_SCORE {
                continue;
            }
            let clock = Instant::now();
            self.push(move_);
            if move_index == 0
                || -self
                    .alpha_beta(depth - 1, -alpha - 1, -alpha, enable_timer)
                    .unwrap_or(alpha)
                    > alpha
            {
                score = -self
                    .alpha_beta(depth - 1, -beta, -alpha, enable_timer)
                    .unwrap_or(-score);
                max_score = max_score.max(score);
            }
            self.pop();
            if self.timer.check_stop(enable_timer) {
                if max_score == -CHECKMATE_SCORE {
                    return None;
                }
                break;
            }
            if print_move_info && self.is_main_threaded() {
                let time_elapsed = clock.elapsed();
                if time_elapsed > PRINT_MOVE_INFO_DURATION_THRESHOLD {
                    self.print_root_node_info(&self.board, move_, depth, score, time_elapsed)
                }
            }
            if score > alpha {
                flag = HashExact;
                alpha = score;
                self.pv_table.update_table(self.ply, move_);
                if score >= beta {
                    TRANSPOSITION_TABLE.write(key, depth, self.ply, beta, HashBeta, move_);
                    return Some(beta);
                }
            }
        }
        if !self.timer.check_stop(enable_timer) {
            TRANSPOSITION_TABLE.write(key, depth, self.ply, alpha, flag, self.get_best_move());
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
        self.pv_table.set_length(self.ply, self.ply);
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
        let best_move = if is_pv_node && self.is_main_threaded() {
            TRANSPOSITION_TABLE.read_best_move(key)
        } else {
            match TRANSPOSITION_TABLE.read(key, depth, self.ply, alpha, beta) {
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
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDERING);
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
            self.get_nth_pv_move(self.ply),
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
            if score > alpha {
                flag = HashExact;
                self.pv_table.update_table(self.ply, move_);
                alpha = score;
                if not_capture_move {
                    self.move_sorter.add_history_move(move_, &self.board, depth);
                }
                if score >= beta {
                    TRANSPOSITION_TABLE.write(key, depth, self.ply, beta, HashBeta, move_);
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
        if !self.timer.check_stop(enable_timer) {
            TRANSPOSITION_TABLE.write(
                key,
                depth,
                self.ply,
                alpha,
                flag,
                self.get_nth_pv_move(self.ply),
            );
        }
        Some(alpha)
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        if self.ply == MAX_PLY - 1 {
            return self.board.evaluate_flipped();
        }
        self.pv_table.set_length(self.ply, self.ply);
        if self.board.is_other_draw() {
            return 0;
        }
        if self.is_main_threaded() {
            self.selective_depth.fetch_max(self.ply, MEMORY_ORDERING);
        }
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDERING);
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
            TRANSPOSITION_TABLE.read_best_move(key),
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
                self.pv_table.update_table(self.ply, move_);
                alpha = score;
            }
        }
        alpha
    }

    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDERING)
    }

    pub fn get_selective_depth(&self) -> Ply {
        self.selective_depth.load(MEMORY_ORDERING)
    }

    pub fn get_hash_full(&self) -> f64 {
        TRANSPOSITION_TABLE.get_hash_full()
    }

    pub fn get_num_overwrites(&self) -> usize {
        TRANSPOSITION_TABLE.get_num_overwrites()
    }

    pub fn get_num_collisions(&self) -> usize {
        TRANSPOSITION_TABLE.get_num_collisions()
    }

    pub fn get_pv(&self) -> Vec<Option<Move>> {
        self.pv_table.get_pv(0).to_vec()
    }

    fn extract_pv_from_t_table(&self, board: &mut Board) -> Vec<Option<Move>> {
        let mut pv = Vec::new();
        let best_move = TRANSPOSITION_TABLE.read_best_move(board.hash());
        if let Some(best_move) = best_move {
            pv.push(Some(best_move));
            board.push(best_move);
            pv.append(&mut self.extract_pv_from_t_table(board).to_vec());
        }
        pv
    }

    pub fn get_pv_from_t_table(&self) -> Vec<Option<Move>> {
        self.extract_pv_from_t_table(&mut self.initial_board.clone())
    }

    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        self.get_pv().get(n).copied().flatten()
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.get_nth_pv_move(0)
    }

    pub fn get_ponder_move(&self) -> Option<Move> {
        self.get_nth_pv_move(1)
    }

    pub fn get_score(&self) -> Score {
        self.score
    }

    pub fn get_search_info(&self, depth: Depth) -> SearchInfo {
        SearchInfo::new(self, depth)
    }

    fn parse_timed_command(&self, command: GoCommand) -> GoCommand {
        if let GoCommand::Timed {
            wtime,
            btime,
            winc,
            binc,
            moves_to_go,
        } = command
        {
            if self.board.generate_legal_moves().len() == 1 {
                return GoCommand::Depth(1);
            }
            let (self_time, self_inc, opponent_time, _opponent_inc) = match self.board.turn() {
                White => (wtime, winc, btime, binc),
                Black => (btime, binc, wtime, winc),
            };
            let divider = moves_to_go.unwrap_or(30);
            let new_inc = self_inc
                .checked_sub(Duration::from_secs(3))
                .unwrap_or_default();
            let opponent_time_advantage = opponent_time.checked_sub(self_time).unwrap_or_default();
            let search_time = self_time
                .checked_sub(opponent_time_advantage)
                .unwrap_or_default()
                / divider
                + new_inc;
            return GoCommand::MoveTime(
                search_time
                    .checked_sub(get_move_overhead())
                    .unwrap_or_default(),
            );
        }
        panic!("Expected Timed Command!");
    }

    pub fn go(&mut self, mut command: GoCommand, print_info: bool) {
        if command.is_timed() {
            command = self.parse_timed_command(command);
        }
        if let GoCommand::MoveTime(duration) = command {
            self.timer.set_max_time(duration);
        }
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut current_depth = 1;
        while current_depth < Depth::MAX {
            self.score = self
                .search(current_depth, alpha, beta, print_info)
                .unwrap_or(self.score);
            let search_info = self.get_search_info(current_depth);
            if print_info && self.is_main_threaded() {
                search_info.print_info();
            }
            if self.timer.check_stop(true) {
                break;
            }
            if self.score <= alpha || self.score >= beta {
                if print_info && self.is_main_threaded() {
                    search_info.print_warning_message(alpha, beta);
                }
                alpha = -INFINITY;
                beta = INFINITY;
                continue;
            }
            let cutoff = if is_checkmate(self.score) {
                10
            } else {
                ASPIRATION_WINDOW_CUTOFF
            };
            alpha = self.score - cutoff;
            beta = self.score + cutoff;
            if command == GoCommand::Depth(current_depth) {
                break;
            }
            current_depth += 1;
        }
    }
}
