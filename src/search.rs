use super::*;
use EntryFlag::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct PVTable {
    length: SerdeWrapper<[usize; MAX_PLY]>,
    table: SerdeWrapper<[SerdeWrapper<[Option<Move>; MAX_PLY]>; MAX_PLY]>,
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            length: [0; MAX_PLY].into(),
            table: [[None; MAX_PLY].into(); MAX_PLY].into(),
        }
    }

    pub fn get_pv(&self, ply: Ply) -> Vec<&Move> {
        get_item_unchecked!(
            self.table,
            ply,
            0..get_item_unchecked!(@internal self.length, ply).to_owned()
        )
        .iter()
        .take_while(|opt_move| opt_move.is_some())
        .map(|opt_move| opt_move.as_ref().unwrap())
        .collect_vec()
    }

    pub fn update_table(&mut self, ply: Ply, move_: Move) {
        *get_item_unchecked_mut!(self.table, ply, ply) = Some(move_);
        for next_ply in (ply + 1)..*get_item_unchecked!(self.length, ply + 1) {
            *get_item_unchecked_mut!(self.table, ply, next_ply) =
                *get_item_unchecked!(self.table, ply + 1, next_ply);
        }
        self.set_length(ply, *get_item_unchecked!(self.length, ply + 1));
    }

    #[inline]
    pub fn set_length(&mut self, ply: Ply, length: usize) {
        *get_item_unchecked_mut!(self.length, ply) = length;
    }
}

impl Default for PVTable {
    fn default() -> Self {
        Self::new()
    }
}

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Searcher<P: PositionEvaluation> {
    id: usize,
    initial_minimum_board: MinimumBoard,
    board: Board,
    evaluator: P,
    transposition_table: SerdeWrapper<Arc<TranspositionTable>>,
    pv_table: PVTable,
    best_moves: Vec<Move>,
    move_sorter: MoveSorter,
    num_nodes_searched: SerdeWrapper<Arc<AtomicUsize>>,
    selective_depth: SerdeWrapper<Arc<AtomicUsize>>,
    ply: Ply,
    score: Score,
    depth_completed: Depth,
    is_outside_aspiration_window: bool,
    clock: Instant,
    stop_command: SerdeWrapper<Arc<AtomicBool>>,
    properties: EngineProperties,
}

impl<P: PositionEvaluation> Searcher<P> {
    pub fn new(
        id: usize,
        board: Board,
        evaluator: P,
        transposition_table: Arc<TranspositionTable>,
        num_nodes_searched: Arc<AtomicUsize>,
        selective_depth: Arc<AtomicUsize>,
        stop_command: Arc<AtomicBool>,
        properties: EngineProperties,
    ) -> Self {
        Self {
            id,
            initial_minimum_board: board.get_minimum_board().to_owned(),
            board,
            evaluator,
            transposition_table: transposition_table.into(),
            pv_table: PVTable::new(),
            best_moves: Vec::new(),
            move_sorter: MoveSorter::new(),
            num_nodes_searched: num_nodes_searched.into(),
            selective_depth: selective_depth.into(),
            ply: 0,
            score: 0,
            depth_completed: 0,
            is_outside_aspiration_window: false,
            clock: Instant::now(),
            stop_command: stop_command.into(),
            properties,
        }
    }

    #[inline]
    pub fn is_main_threaded(&self) -> bool {
        self.get_id() == 0
    }

    #[inline]
    pub fn get_id(&self) -> usize {
        self.id
    }

    #[inline]
    pub fn get_initial_minimum_board(&self) -> &MinimumBoard {
        &self.initial_minimum_board
    }

    #[inline]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    #[inline]
    pub fn get_transposition_table(&self) -> &TranspositionTable {
        &self.transposition_table
    }

    #[inline]
    pub fn get_pv_table(&self) -> &PVTable {
        &self.pv_table
    }

    #[inline]
    pub fn get_best_moves(&self) -> &[Move] {
        &self.best_moves
    }

    #[inline]
    pub fn get_move_sorter(&self) -> &MoveSorter {
        &self.move_sorter
    }

    #[inline]
    pub fn get_ply(&self) -> Ply {
        self.ply
    }

    #[inline]
    pub fn get_stop_command(&self) -> Arc<AtomicBool> {
        self.stop_command.clone().into_inner()
    }

    #[inline]
    pub fn get_num_nodes_searched(&self) -> usize {
        self.num_nodes_searched.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_selective_depth(&self) -> Ply {
        self.selective_depth.load(MEMORY_ORDERING)
    }

    #[inline]
    pub fn get_clock(&self) -> Instant {
        self.clock
    }

    #[inline]
    pub fn get_time_elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    #[inline]
    pub fn get_pv(&self) -> Vec<&Move> {
        self.pv_table.get_pv(0)
    }

    #[inline]
    pub fn get_pv_from_t_table(&self) -> Vec<Move> {
        extract_pv_from_t_table(&self.initial_minimum_board, &self.transposition_table)
            .into_iter()
            .map_into()
            .collect_vec()
    }

    #[inline]
    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        Some(**self.pv_table.get_pv(0).get(n)?)
    }

    #[inline]
    pub fn get_best_move(&self) -> Option<Move> {
        self.get_nth_pv_move(0)
    }

    #[inline]
    pub fn get_ponder_move(&self) -> Option<Move> {
        self.get_nth_pv_move(1)
    }

    #[inline]
    pub fn get_score(&self) -> Score {
        self.score
    }

    #[inline]
    pub fn get_depth_completed(&self) -> Depth {
        self.depth_completed
    }

    #[inline]
    pub fn is_outside_aspiration_window(&self) -> bool {
        self.is_outside_aspiration_window
    }

    #[inline]
    pub fn get_search_info(&self) -> SearchInfo {
        self.into()
    }

    #[inline]
    pub fn stop_search_at_every_node(
        &self,
        controller: Option<&mut impl SearchControl<Self>>,
    ) -> bool {
        if self.stop_command.load(MEMORY_ORDERING) {
            return true;
        }
        if let Some(controller) = controller {
            controller.stop_search_at_every_node(self)
        } else {
            false
        }
    }

    fn pop(&mut self) -> ValidOrNullMove {
        self.ply -= 1;
        self.board.pop()
    }

    fn evaluate_flipped(&mut self) -> Score {
        self.evaluator.evaluate_flipped(&self.board)
    }

    #[inline]
    pub fn print_root_node_info(
        board: &Board,
        curr_move: Move,
        depth: Depth,
        score: Score,
        num_nodes_searched: usize,
        time_elapsed: Duration,
    ) {
        println_wasm!(
            "{} {} {} {} {} {} {} {} {} {} {}",
            "info".colorize(INFO_MESSAGE_STYLE),
            "curr move".colorize(INFO_MESSAGE_STYLE),
            curr_move.stringify_move(board).unwrap(),
            "depth".colorize(INFO_MESSAGE_STYLE),
            depth,
            "score".colorize(INFO_MESSAGE_STYLE),
            board.score_flipped(score).stringify(),
            "nodes".colorize(INFO_MESSAGE_STYLE),
            num_nodes_searched,
            "time".colorize(INFO_MESSAGE_STYLE),
            time_elapsed.stringify(),
        );
    }

    fn is_draw_move(&self, valid_or_null_move: ValidOrNullMove) -> bool {
        self.board.gives_threefold_repetition(valid_or_null_move)
            || self
                .board
                .gives_claimable_threefold_repetition(valid_or_null_move)
    }

    fn update_best_moves(&mut self) {
        if let Some(best_move) = self.get_best_move() {
            self.best_moves
                .retain(|&valid_or_null_move| valid_or_null_move != best_move);
            self.best_moves.insert(0, best_move);
        }
    }

    fn get_sorted_root_node_moves(&mut self) -> Vec<(Move, MoveWeight)> {
        let mut moves_vec_sorted = self
            .move_sorter
            .get_weighted_moves_sorted(
                &self.board,
                &self.transposition_table,
                0,
                self.transposition_table
                    .read_best_move(self.board.get_hash()),
                self.get_best_move(),
            )
            .map(|WeightedMove { move_, .. }| {
                let pv_move = self.get_best_move();
                (
                    move_,
                    MoveSorter::score_root_moves(
                        &mut self.board,
                        &mut self.evaluator,
                        move_,
                        pv_move,
                        &self.best_moves,
                    ),
                )
            })
            .collect_vec();
        moves_vec_sorted.sort_by_key(|&t| Reverse(t.1));
        moves_vec_sorted
    }

    fn search(
        &mut self,
        depth: Depth,
        mut alpha: Score,
        beta: Score,
        mut controller: Option<&mut impl SearchControl<Self>>,
        print_move_info: bool,
    ) -> Option<Score> {
        if FOLLOW_PV {
            self.move_sorter.follow_pv();
        }
        if self.is_main_threaded() {
            self.selective_depth.store(0, MEMORY_ORDERING);
        }
        if self.board.is_game_over() {
            return if self.board.is_checkmate() {
                Some(-self.evaluator.evaluate_checkmate(0))
            } else {
                Some(self.evaluator.evaluate_draw())
            };
        }
        if !(depth > 1 && self.is_main_threaded()) {
            controller = None;
        }
        if self.stop_search_at_every_node(controller.as_deref_mut()) {
            return None;
        }
        let key = self.board.get_hash();
        let mut score = -INFINITY;
        let mut flag = HashAlpha;
        let is_endgame = self.board.is_endgame();
        let moves = self.get_sorted_root_node_moves();
        for (move_index, &(move_, _)) in moves.iter().enumerate() {
            if !is_endgame && self.is_draw_move(move_.into()) && score > -DRAW_SCORE {
                continue;
            }
            let clock = Instant::now();
            self.push_unchecked(move_);
            if move_index == 0
                || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, controller.as_deref_mut())?
                    > alpha
            {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, controller.as_deref_mut())?;
            }
            self.pop();
            if print_move_info && self.is_main_threaded() {
                let time_elapsed = clock.elapsed();
                if time_elapsed > PRINT_MOVE_INFO_DURATION_THRESHOLD {
                    Self::print_root_node_info(
                        &self.board,
                        move_,
                        depth,
                        score,
                        self.get_num_nodes_searched(),
                        time_elapsed,
                    )
                }
            }
            if score > alpha {
                flag = HashExact;
                alpha = score;
                self.pv_table.update_table(self.ply, move_);
                if score >= beta {
                    self.transposition_table.write(
                        key,
                        depth,
                        self.ply,
                        beta,
                        HashBeta,
                        Some(move_),
                    );
                    return Some(beta);
                }
            }
        }
        if !self.stop_search_at_every_node(controller) {
            self.transposition_table
                .write(key, depth, self.ply, alpha, flag, self.get_best_move());
        }
        self.update_best_moves();
        Some(alpha)
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
        mut controller: Option<&mut impl SearchControl<Self>>,
    ) -> Option<Score> {
        self.pv_table.set_length(self.ply, self.ply);
        let mate_score = self.evaluator.evaluate_checkmate(self.ply);
        let draw_score = self.evaluator.evaluate_draw();
        if self.board.is_other_draw() {
            return Some(draw_score);
        }
        if self.properties.use_mate_distance_pruning() {
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
        }
        let checkers = self.board.get_checkers();
        if depth > 10 {
            depth += checkers.popcnt() as Depth;
        }
        let min_depth = self.move_sorter.is_following_pv() as Depth;
        depth = depth.max(min_depth);
        let is_pv_node = alpha != beta - 1;
        let key = self.board.get_hash();
        let best_move = if is_pv_node && self.is_main_threaded() {
            self.transposition_table.read_best_move(key)
        } else {
            let (optional_data, best_move) = self.transposition_table.read(key, depth, self.ply);
            if let Some((score, flag)) = optional_data {
                // match flag {
                //     HashExact => return Some(score),
                //     HashAlpha => alpha = alpha.max(score),
                //     HashBeta => beta = beta.min(score),
                // }
                // if alpha >= beta {
                //     return Some(alpha);
                // }
                match flag {
                    HashExact => return Some(score),
                    HashAlpha => {
                        if score <= alpha {
                            return Some(score);
                        }
                    }
                    HashBeta => {
                        if score >= beta {
                            return Some(score);
                        }
                    }
                }
            }
            best_move
        };
        if self.ply == MAX_PLY - 1 {
            return Some(self.evaluate_flipped());
        }
        // enable_controller &= depth > 3;
        if self.stop_search_at_every_node(controller.as_deref_mut()) {
            return None;
        }
        if depth == 0 {
            return Some(self.quiescence(alpha, beta));
        }
        if self.is_main_threaded() && is_pv_node {
            self.selective_depth.fetch_max(self.ply, MEMORY_ORDERING);
        }
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDERING);
        let not_in_check = checkers.is_empty();
        let mut futility_pruning = false;
        if not_in_check && !DISABLE_ALL_PRUNINGS {
            // static evaluation
            let static_evaluation = self.evaluate_flipped();
            if depth < 3 && !is_pv_node && !is_checkmate(beta) {
                let eval_margin = ((6 * PAWN_VALUE) / 5) * depth as Score;
                let new_score = static_evaluation - eval_margin;
                if new_score >= beta {
                    return Some(new_score);
                }
            }
            // razoring
            const RAZORING_DEPTH: Depth = 3;
            if !is_pv_node && depth <= RAZORING_DEPTH && !is_checkmate(beta) {
                let mut score = static_evaluation + const { (5 * PAWN_VALUE) / 4 };
                if score < beta {
                    if depth == 1 {
                        let new_score = self.quiescence(alpha, beta);
                        return Some(new_score.max(score));
                    }
                    score += const { (7 * PAWN_VALUE) / 4 };
                    if score < beta && depth < RAZORING_DEPTH {
                        let new_score = self.quiescence(alpha, beta);
                        if new_score < beta {
                            return Some(new_score.max(score));
                        }
                    }
                }
            }
            // null move pruning
            if depth >= NULL_MOVE_MIN_DEPTH
                && static_evaluation >= beta
                && self.board.has_non_pawn_material()
            {
                // let r = NULL_MOVE_MIN_REDUCTION
                //     + (depth.max(NULL_MOVE_MIN_DEPTH) as f64 / NULL_MOVE_DEPTH_DIVIDER as f64)
                //         .round() as Depth;
                // let reduced_depth = depth - r - 1;
                let r = 1920 + (depth as i32) * 2368;
                let reduced_depth = (((depth as u32) * 4096 - (r as u32)) / 4096) as Depth;
                self.push_unchecked(ValidOrNullMove::NullMove);
                let score =
                    -self.alpha_beta(reduced_depth, -beta, -beta + 1, controller.as_deref_mut())?;
                self.pop();
                if score >= beta {
                    return Some(beta);
                }
            }
            // futility pruning condition
            if depth < 4 && alpha < mate_score {
                let futility_margin = match depth {
                    0 => 0,
                    1 => PAWN_VALUE,
                    2 => const { Knight.evaluate() },
                    3 => const { Rook.evaluate() },
                    _ => unreachable!(),
                };
                futility_pruning = static_evaluation + futility_margin <= alpha;
            }
        }
        let mut flag = HashAlpha;
        let weighted_moves = self.move_sorter.get_weighted_moves_sorted(
            &self.board,
            &self.transposition_table,
            self.ply,
            best_move,
            self.get_nth_pv_move(self.ply),
        );
        if weighted_moves.is_empty() {
            return if not_in_check {
                Some(draw_score)
            } else {
                Some(-mate_score)
            };
        }
        for (move_index, WeightedMove { move_, .. }) in weighted_moves.enumerate() {
            let not_capture_move = !self.board.is_capture(move_);
            let not_an_interesting_position = not_capture_move
                && not_in_check
                && move_.get_promotion().is_none()
                && !self.move_sorter.is_killer_move(move_, self.ply);
            if move_index != 0 && futility_pruning && not_an_interesting_position {
                continue;
            }
            let mut safe_to_apply_lmr = move_index >= FULL_DEPTH_SEARCH_LMR
                && depth >= REDUCTION_LIMIT_LMR
                && self.properties.use_lmr()
                && not_an_interesting_position;
            self.push_unchecked(move_);
            safe_to_apply_lmr &= !self.board.is_check();
            let mut score: Score;
            if move_index == 0 {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, controller.as_deref_mut())?;
            } else {
                if safe_to_apply_lmr {
                    let lmr_reduction = Self::get_lmr_reduction(depth, move_index, is_pv_node);
                    score = if depth > lmr_reduction {
                        -self.alpha_beta(
                            depth - 1 - lmr_reduction,
                            -alpha - 1,
                            -alpha,
                            controller.as_deref_mut(),
                        )?
                    } else {
                        alpha + 1
                    }
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    score = -self.alpha_beta(
                        depth - 1,
                        -alpha - 1,
                        -alpha,
                        controller.as_deref_mut(),
                    )?;
                    if score > alpha && score < beta {
                        score = -self.alpha_beta(
                            depth - 1,
                            -beta,
                            -alpha,
                            controller.as_deref_mut(),
                        )?;
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
                    return Some(beta);
                }
            }
        }
        if !self.stop_search_at_every_node(controller) {
            self.transposition_table.write(
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
            return self.evaluate_flipped();
        }
        self.pv_table.set_length(self.ply, self.ply);
        if self.board.is_other_draw() {
            return self.evaluator.evaluate_draw();
        }
        let is_pv_node = alpha != beta - 1;
        if self.is_main_threaded() && is_pv_node {
            self.selective_depth.fetch_max(self.ply, MEMORY_ORDERING);
        }
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDERING);
        let evaluation = self.evaluate_flipped();
        if evaluation >= beta {
            return beta;
        }
        alpha = alpha.max(evaluation);
        for WeightedMove { move_, weight } in self
            .move_sorter
            .get_weighted_capture_moves_sorted(&self.board, &self.transposition_table)
        {
            if weight.is_negative() {
                break;
            }
            self.push_unchecked(move_);
            let score = -self.quiescence(-beta, -alpha);
            self.pop();
            if score >= beta {
                return beta;
            }
            if score > alpha {
                self.pv_table.update_table(self.ply, move_);
                alpha = score;
            }
            // delta pruning
            let mut delta = const { Queen.evaluate() };
            if let Some(piece) = move_.get_promotion() {
                delta += piece.evaluate() - PAWN_VALUE;
            }
            if score + delta < alpha {
                return alpha;
            }
        }
        alpha
    }

    pub fn go(
        &mut self,
        mut command: GoCommand,
        mut controller: impl SearchControl<Self>,
        verbose: bool,
    ) {
        if self.board.generate_legal_moves().len() == 1 {
            command = GoCommand::Depth(1);
        }
        controller.on_receiving_go_command(command, self);
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        self.depth_completed = 0;
        while self.depth_completed < Depth::MAX
            && !self.stop_command.load(MEMORY_ORDERING)
            && !controller.stop_search_at_root_node(self)
        {
            let last_score = self.score;
            self.score = self
                .search(
                    self.depth_completed + 1,
                    alpha,
                    beta,
                    Some(&mut controller),
                    verbose,
                )
                .unwrap_or(self.score);
            let search_info = self.get_search_info();
            if verbose && self.is_main_threaded() {
                search_info.print_info();
            }
            self.is_outside_aspiration_window = self.score <= alpha || self.score >= beta;
            controller.on_each_search_completion(self);
            if self.is_outside_aspiration_window {
                if verbose && self.is_main_threaded() {
                    search_info.print_warning_message(alpha, beta);
                }
                alpha = -INFINITY;
                beta = INFINITY;
                self.score = last_score;
                continue;
            }
            let cutoff = if is_checkmate(self.score) {
                5
            } else {
                ASPIRATION_WINDOW_CUTOFF
            };
            alpha = self.score - cutoff;
            beta = self.score + cutoff;
            self.depth_completed += 1;
        }
    }
}

impl<P: PositionEvaluation> SearcherMethodOverload<Move> for Searcher<P> {
    fn push_unchecked(&mut self, move_: Move) {
        self.board.push_unchecked(move_);
        self.ply += 1;
    }
}

impl<P: PositionEvaluation> SearcherMethodOverload<ValidOrNullMove> for Searcher<P> {
    fn push_unchecked(&mut self, valid_or_null_move: ValidOrNullMove) {
        self.board.push_unchecked(valid_or_null_move);
        self.ply += 1;
    }
}
