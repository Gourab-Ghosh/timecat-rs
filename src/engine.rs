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

pub struct SearchInfo {
    board: Board,
    depth: Depth,
    seldepth: Depth,
    score: Score,
    nodes: usize,
    hash_full: f64,
    collisions: usize,
    clock: Instant,
    pv: Vec<Option<Move>>
}

impl SearchInfo {
    pub fn new(engine: &Engine, current_depth: Depth, score: Score, clock: Instant) -> Self {
        Self {
            board: engine.board.clone(),
            depth: current_depth,
            seldepth: engine.get_selective_depth(),
            score,
            nodes: engine.get_num_nodes_searched(),
            hash_full: engine.get_hash_full(),
            collisions: engine.get_num_collisions(),
            clock,
            pv: engine.get_pv().to_vec(),
        }
    }

    pub fn set_board(&mut self, board: &Board) {
        self.board = board.clone();
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
            let is_legal_move = if let Some(move_) = move_ { board.is_legal(move_) } else { false };
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
        let style = SUCCESS_MESSAGE_STYLE;
        println!(
            "{} {} {} {} {} {} {} {} {} {} {} {:.2} {} {} {} {:.3} {} {}",
            colorize("info depth", style),
            self.depth,
            colorize("seldepth", style),
            self.seldepth,
            colorize("score", style),
            score.stringify_score(),
            colorize("nodes", style),
            self.nodes,
            colorize("nps", style),
            (self.nodes as u128 * 10_u128.pow(9)) / self.get_time_elapsed().as_nanos(),
            colorize("hashfull", style),
            self.hash_full,
            colorize("collisions", style),
            self.collisions,
            colorize("time", style),
            self.get_time_elapsed().as_secs_f64(),
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
            "Resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {:.3} s",
            self.depth,
            alpha.stringify_score(),
            beta.stringify_score(),
            score.stringify_score(),
            self.get_time_elapsed().as_secs_f64(),
        );
        println!("{}", colorize(warning_message, WARNING_MESSAGE_STYLE));
    }
}

struct PVTable {
    length: [usize; MAX_PLY],
    table: [[Option<Move>; MAX_PLY]; MAX_PLY],
}

impl PVTable {
    fn new() -> Self {
        Self {
            length: [0; MAX_PLY],
            table: [[None; MAX_PLY]; MAX_PLY],
        }
    }

    pub fn get_pv(&self, ply: Ply) -> &[Option<Move>] {
        &self.table[ply][0..self.length[ply]]
    }

    pub fn update_table(&mut self, move_: Move, ply: Ply) {
        self.table[ply][ply] = Some(move_);
        for next_ply in (ply + 1)..self.length[ply + 1] {
            self.table[ply][next_ply] = self.table[ply + 1][next_ply];
        }
        self.length[ply] = self.length[ply + 1];
    }

    #[inline(always)]
    fn set_length(&mut self, ply: Ply, length: usize) {
        self.length[ply] = length;
    }

    #[inline(always)]
    fn clear_table(&mut self, ply: Ply) {
        self.length[ply] = 0;
    }

    #[inline(always)]
    fn reset_variables(&mut self) {
        for i in 0..MAX_PLY {
            self.clear_table(i);
        }
    }
}

pub struct Engine {
    pub board: Board,
    pv_table: PVTable,
    transposition_table: TranspositionTable,
    move_sorter: MoveSorter,
    timer: Timer,
    num_nodes_searched: AtomicUsize,
    selective_depth: AtomicUsize,
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            pv_table: PVTable::new(),
            num_nodes_searched: AtomicUsize::new(0),
            selective_depth: AtomicUsize::new(0),
            move_sorter: MoveSorter::default(),
            transposition_table: TranspositionTable::default(),
            timer: Timer::default(),
        }
    }

    fn reset_variables(&mut self) {
        self.num_nodes_searched.store(0, MEMORY_ORDER);
        self.pv_table.reset_variables();
        self.move_sorter.reset_variables();
        self.timer.reset_variables();
        self.transposition_table.reset_variables();
        // self.board.get_evaluator().lock().unwrap().reset_variables();
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), chess::Error> {
        let result = self.board.set_fen(fen);
        self.reset_variables();
        result
    }

    pub fn from_fen(fen: &str) -> Result<Self, chess::Error> {
        Ok(Engine::new(Board::from_fen(fen)?))
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
            "{} {} {} {} {} {} {} {} {} {} {:.3} s",
            colorize("info", INFO_STYLE),
            colorize("curr move", INFO_STYLE),
            curr_move.stringify_move(board).unwrap(),
            colorize("depth", INFO_STYLE),
            depth,
            colorize("score", INFO_STYLE),
            board.score_flipped(score).stringify_score(),
            colorize("nodes", INFO_STYLE),
            self.num_nodes_searched.load(MEMORY_ORDER),
            colorize("time", INFO_STYLE),
            time_elapsed.as_secs_f64(),
        );
    }

    fn is_draw_move(&mut self, board: &mut Board, move_: Move) -> bool {
        board.gives_threefold_repetition(move_) || board.gives_claimable_threefold_repetition(move_)
    }

    fn get_sorted_root_node_moves(&mut self) -> Vec<(Move, MoveWeight)> {
        let mut moves_vec_sorted = self
            .move_sorter
            .get_weighted_moves_sorted(
                self.board.generate_legal_moves(),
                &self.board,
                0,
                self.transposition_table.read_best_move(self.board.hash()),
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
        self.selective_depth.store(0, MEMORY_ORDER);
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
        let moves = self.get_sorted_root_node_moves();
        let mut board = self.board.clone();
        let mut ply = 0;
        for (move_index, &(move_, _)) in moves.iter().enumerate() {
            if !is_endgame && self.is_draw_move(&mut board, move_) && max_score > -DRAW_SCORE {
                continue;
            }
            let clock = Instant::now();
            board.push(move_);
            ply += 1;
            if move_index == 0
                || -self.alpha_beta(&mut board, depth - 1, ply, -alpha - 1, -alpha, enable_timer)?
                    > alpha
            {
                score =
                    -self.alpha_beta(&mut board, depth - 1, ply, -beta, -alpha, enable_timer)?;
                max_score = max_score.max(score);
            }
            board.pop();
            ply -= 1;
            if self.timer.check_stop(enable_timer) {
                break;
            }
            if print_move_info {
                let time_elapsed = clock.elapsed();
                if time_elapsed > PRINT_MOVE_INFO_DURATION_THRESHOLD {
                    self.print_root_node_info(&board, move_, depth, score, time_elapsed)
                }
            }
            if score > alpha {
                flag = HashExact;
                alpha = score;
                if (initial_alpha < score && score < initial_beta) || is_checkmate(score) {
                    self.pv_table.update_table(move_, ply);
                }
                // self.pv_table.update_table(move_, ply);
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, ply, beta, HashBeta, move_);
                    return Some(beta);
                }
            }
        }
        if !self.timer.check_stop(enable_timer) {
            self.transposition_table
                .write(key, depth, ply, alpha, flag, self.get_best_move());
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
        board: &mut Board,
        mut depth: Depth,
        mut ply: Ply,
        mut alpha: Score,
        mut beta: Score,
        mut enable_timer: bool,
    ) -> Option<Score> {
        self.pv_table.set_length(ply, ply);
        if board.is_other_draw() {
            return Some(0);
        }
        let mate_score = CHECKMATE_SCORE - ply as Score;
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
        let checkers = board.get_checkers();
        let min_depth = if self.move_sorter.is_following_pv() {
            1
        } else {
            0
        };
        depth = (depth + checkers.popcnt() as Depth).max(min_depth);
        let is_pv_node = alpha != beta - 1;
        let key = board.hash();
        let best_move = if is_pv_node {
            self.transposition_table.read_best_move(key)
        } else {
            match self.transposition_table.read(key, depth, ply, alpha, beta) {
                (Some(score), _) => return Some(score),
                (None, best_move) => best_move,
            }
        };
        if ply == MAX_PLY - 1 {
            return Some(board.evaluate_flipped());
        }
        enable_timer &= depth > 3;
        if self.timer.check_stop(enable_timer) {
            return None;
        }
        if depth == 0 {
            return Some(self.quiescence(board, ply, alpha, beta));
        }
        // let is_endgame = board.is_endgame();
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDER);
        let not_in_check = checkers == BB_EMPTY;
        if not_in_check && !DISABLE_ALL_PRUNINGS {
            // static evaluation
            let static_evaluation = board.evaluate_flipped();
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
                && board.has_non_pawn_material()
            {
                let r = NULL_MOVE_MIN_REDUCTION
                    + (depth.abs_diff(NULL_MOVE_MIN_DEPTH) as f64 / NULL_MOVE_DEPTH_DIVIDER as f64)
                        .round() as Depth;
                board.push(None);
                ply += 1;
                let score =
                    -self.alpha_beta(board, depth - 1 - r, ply, -beta, -beta + 1, enable_timer)?;
                board.pop();
                ply -= 1;
                if self.timer.check_stop(enable_timer) {
                    return None;
                }
                if score >= beta {
                    return Some(beta);
                }
            }
            // razoring
            let d = 3;
            if !is_pv_node && depth <= d && !board.is_endgame() {
                let mut score = static_evaluation + (5 * PAWN_VALUE) / 4;
                if score < beta {
                    if depth == 1 {
                        let new_score = self.quiescence(board, ply, alpha, beta);
                        return Some(new_score.max(score));
                    }
                    score += (7 * PAWN_VALUE) / 4;
                    if score < beta && depth < d {
                        let new_score = self.quiescence(board, ply, alpha, beta);
                        if new_score < beta {
                            return Some(new_score.max(score));
                        }
                    }
                }
            }
        }
        let mut flag = HashAlpha;
        let moves_gen = board.generate_legal_moves();
        let weighted_moves = self.move_sorter.get_weighted_moves_sorted(
            moves_gen,
            board,
            ply,
            best_move,
            self.get_nth_pv_move(ply),
            Evaluator::is_easily_winning_position(board, board.get_material_score()),
        );
        let mut move_index = 0;
        for WeightedMove { move_, .. } in weighted_moves {
            let not_capture_move = !board.is_capture(move_);
            let mut safe_to_apply_lmr = move_index >= FULL_DEPTH_SEARCH_LMR
                && depth >= REDUCTION_LIMIT_LMR
                && !DISABLE_LMR
                && not_capture_move
                && not_in_check
                && move_.get_promotion().is_none()
                && !self.move_sorter.is_killer_move(move_, ply)
                && !board.is_passed_pawn(move_.get_source());
            board.push(move_);
            ply += 1;
            safe_to_apply_lmr &= !board.is_check();
            let mut score: Score;
            if move_index == 0 {
                score = -self.alpha_beta(board, depth - 1, ply, -beta, -alpha, enable_timer)?;
            } else {
                if safe_to_apply_lmr {
                    let lmr_reduction = Self::get_lmr_reduction(depth, move_index, is_pv_node);
                    score = if depth > lmr_reduction {
                        -self.alpha_beta(
                            board,
                            depth - 1 - lmr_reduction,
                            ply,
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
                    score = -self.alpha_beta(
                        board,
                        depth - 1,
                        ply,
                        -alpha - 1,
                        -alpha,
                        enable_timer,
                    )?;
                    if score > alpha && score < beta {
                        score =
                            -self.alpha_beta(board, depth - 1, ply, -beta, -alpha, enable_timer)?;
                    }
                }
            }
            board.pop();
            ply -= 1;
            if self.timer.check_stop(enable_timer) {
                return None;
            }
            if score > alpha {
                flag = HashExact;
                self.pv_table.update_table(move_, ply);
                alpha = score;
                if not_capture_move {
                    self.move_sorter.add_history_move(move_, board, depth);
                }
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, ply, beta, HashBeta, move_);
                    if not_capture_move {
                        self.move_sorter.update_killer_moves(move_, ply);
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
            self.transposition_table
                .write(key, depth, ply, alpha, flag, self.get_nth_pv_move(ply));
        }
        Some(alpha)
    }

    fn quiescence(
        &mut self,
        board: &mut Board,
        mut ply: Ply,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        if ply == MAX_PLY - 1 {
            return board.evaluate_flipped();
        }
        self.pv_table.set_length(ply, ply);
        if board.is_other_draw() {
            return 0;
        }
        self.selective_depth.fetch_max(ply, MEMORY_ORDER);
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDER);
        let evaluation = board.evaluate_flipped();
        if evaluation >= beta {
            return beta;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        let key = board.hash();
        for WeightedMove { move_, weight } in self.move_sorter.get_weighted_capture_moves_sorted(
            board.generate_legal_captures(),
            self.transposition_table.read_best_move(key),
            board,
        ) {
            if weight.is_negative() {
                break;
            }
            board.push(Some(move_));
            ply += 1;
            let score = -self.quiescence(board, ply, -beta, -alpha);
            board.pop();
            ply -= 1;
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
                self.pv_table.update_table(move_, ply);
                alpha = score;
            }
        }
        alpha
    }

    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDER)
    }

    pub fn get_selective_depth(&self) -> Depth {
        self.selective_depth.load(MEMORY_ORDER) as Depth
    }

    pub fn get_hash_full(&self) -> f64 {
        self.transposition_table.get_hash_full()
    }

    pub fn get_num_collisions(&self) -> usize {
        self.transposition_table.get_num_collisions()
    }

    pub fn get_pv(&self) -> &[Option<Move>] {
        self.pv_table.get_pv(0)
    }

    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        self.pv_table.get_pv(0).get(n).copied().flatten()
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.get_nth_pv_move(0)
    }

    pub fn get_ponder_move(&self) -> Option<Move> {
        self.get_nth_pv_move(1)
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
            score = self
                .search(current_depth, alpha, beta, print_info)
                .unwrap_or(prev_score);
            let search_info = SearchInfo::new(self, current_depth, score, self.timer.get_clock());
            if print_info {
                search_info.print_info();
            }
            if self.timer.check_stop(true) {
                break;
            }
            if score <= alpha || score >= beta {
                if print_info {
                    search_info.print_warning_message(alpha, beta);
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
        Self::new(Board::default())
    }
}
