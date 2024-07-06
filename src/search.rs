use super::*;
use EntryFlag::*;

// #[cfg(feature = "serde")]
// fn serialize_time_instant<S>(instant: &Instant, serializer: S) -> std::result::Result<S::Ok, S::Error>
// where
//     S: serde::Serializer,
// {
//     serializer.serialize_str(&format!("{:?}", instant))
// }

// fn deserialize_date<'de, D>(deserializer: D) -> Result<String, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     use serde::{Serializer, Deserializer, Deserialize, Serialize};
//     use serde::de::{self, Visitor};
//     use std::fmt;
//     let s = String::deserialize(deserializer)?;
//     // Convert the date from "DD-MM-YYYY" back to "YYYY-MM-DD"
//     let date_parts: Vec<&str> = s.split('-').collect();
//     if date_parts.len() == 3 {
//         let formatted_date = format!("{}-{}-{}", date_parts[2], date_parts[1], date_parts[0]);
//         Ok(formatted_date)
//     } else {
//         Err(D::Error::custom("Invalid date format"))
//     }
// }

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct SearchInfo {
    sub_board: SubBoard,
    depth: Depth,
    seldepth: Ply,
    score: Score,
    nodes: usize,
    hash_full: f64,
    overwrites: usize,
    zero_hit: usize,
    collisions: usize,
    // #[serde(serialize_with = "serialize_time_instant", deserialize_with = "deserialize_time_instant")]
    // #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_time_instant"))]
    clock: Instant,
    pv: Vec<Move>,
}

impl SearchInfo {
    pub fn new(searcher: &Searcher) -> Self {
        Self {
            sub_board: searcher.initial_sub_board.to_owned(),
            depth: searcher.get_current_depth(),
            seldepth: searcher.get_selective_depth(),
            score: searcher.get_score(),
            nodes: searcher.get_num_nodes_searched(),
            hash_full: searcher.get_hash_full(),
            overwrites: searcher.get_num_overwrites(),
            collisions: searcher.get_num_collisions(),
            zero_hit: searcher.get_zero_hit(),
            clock: searcher.timer.get_clock(),
            pv: searcher.get_pv().into_iter().copied().collect_vec(),
        }
    }

    pub fn get_depth(&self) -> Depth {
        self.depth
    }

    pub fn get_pv(&self) -> &[Move] {
        self.pv.as_slice()
    }

    pub fn set_pv(&mut self, pv: &[Move]) {
        self.pv = pv.to_vec();
    }

    pub fn get_score(&self) -> Score {
        let mut score = self.score;
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            score = self.sub_board.score_flipped(score);
        }
        score
    }

    pub fn get_time_elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    #[inline]
    pub fn format_info<T: fmt::Display>(desc: &str, info: T) -> String {
        format!(
            "{} {info}",
            desc.trim()
                .trim_end_matches(':')
                .colorize(SUCCESS_MESSAGE_STYLE)
        )
    }

    pub fn print_info(&self) {
        #[cfg(any(feature = "debug", not(feature = "binary")))]
        let hashfull_string = if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            format!("{:.2}%", self.hash_full)
        } else {
            (self.hash_full.round() as u8).to_string()
        };
        let nps = (self.nodes as u128 * 10_u128.pow(9)) / self.get_time_elapsed().as_nanos();
        let outputs = [
            "info".colorize(INFO_MESSAGE_STYLE),
            Self::format_info("depth", self.depth),
            Self::format_info("seldepth", self.seldepth),
            Self::format_info("score", self.get_score().stringify()),
            Self::format_info("nodes", self.nodes),
            Self::format_info("nps", nps),
            #[cfg(any(feature = "debug", not(feature = "binary")))]
            Self::format_info("hashfull", hashfull_string),
            #[cfg(feature = "debug")]
            Self::format_info("overwrites", self.overwrites),
            #[cfg(feature = "debug")]
            Self::format_info("collisions", self.collisions),
            #[cfg(feature = "debug")]
            Self::format_info("zero hit", self.zero_hit),
            Self::format_info("time", self.get_time_elapsed().stringify()),
            Self::format_info("pv", get_pv_string(&self.sub_board, &self.pv)),
        ];
        println_wasm!("{}", outputs.join(" "));
    }

    pub fn print_warning_message(&self, mut alpha: Score, mut beta: Score) {
        if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
            alpha = self.sub_board.score_flipped(alpha);
            beta = self.sub_board.score_flipped(beta);
        }
        let warning_message = format!(
            "info string resetting alpha to -INFINITY and beta to INFINITY at depth {} having alpha {}, beta {} and score {} with time {}",
            self.depth,
            alpha.stringify(),
            beta.stringify(),
            self.get_score().stringify(),
            self.get_time_elapsed().stringify(),
        );
        println_wasm!("{}", warning_message.colorize(WARNING_MESSAGE_STYLE));
    }
}

#[derive(Clone, Debug)]
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

    #[allow(unused_unsafe)]
    pub fn get_pv(&self, ply: Ply) -> Vec<&Move> {
        get_item_unchecked!(
            self.table,
            ply,
            0..unsafe { *self.length.get_unchecked(ply) }
        )
        .iter()
        .filter_map(|opt_move| opt_move.as_ref())
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

#[derive(Clone, Debug, Default)]
pub struct Searcher {
    id: usize,
    initial_sub_board: SubBoard,
    board: Board,
    transposition_table: Arc<TranspositionTable>,
    pv_table: PVTable,
    best_moves: Vec<Move>,
    move_sorter: MoveSorter,
    timer: Timer,
    num_nodes_searched: Arc<AtomicUsize>,
    selective_depth: Arc<AtomicUsize>,
    ply: Ply,
    score: Score,
    current_depth: Depth,
}

impl Searcher {
    pub fn new(
        id: usize,
        board: Board,
        transposition_table: Arc<TranspositionTable>,
        num_nodes_searched: Arc<AtomicUsize>,
        selective_depth: Arc<AtomicUsize>,
        stopper: Arc<AtomicBool>,
        move_overhead: Duration,
    ) -> Self {
        Self {
            id,
            initial_sub_board: board.get_sub_board().to_owned(),
            board,
            transposition_table,
            pv_table: PVTable::new(),
            best_moves: Vec::new(),
            move_sorter: MoveSorter::new(),
            timer: if id == 0 {
                Timer::new(stopper)
            } else {
                Timer::new_dummy(stopper)
            }
            .with_move_overhead(move_overhead),
            num_nodes_searched,
            selective_depth,
            ply: 0,
            score: 0,
            current_depth: 0,
        }
    }

    #[inline]
    fn is_main_threaded(&self) -> bool {
        self.id == 0
    }

    fn pop(&mut self) -> ValidOrNullMove {
        self.ply -= 1;
        self.board.pop()
    }

    fn print_root_node_info(
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
                Evaluator::is_easily_winning_position(&self.board, self.board.get_material_score()),
            )
            .map(|WeightedMove { move_, .. }| {
                let pv_move = self.get_best_move();
                (
                    move_,
                    MoveSorter::score_root_moves(&mut self.board, move_, pv_move, &self.best_moves),
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
        if self.board.is_game_over() {
            return if self.board.is_checkmate() {
                Some(-CHECKMATE_SCORE)
            } else {
                Some(0)
            };
        }
        let enable_timer = depth > 1 && self.is_main_threaded();
        if self.timer.check_stop(enable_timer) {
            return None;
        }
        let key = self.board.get_hash();
        let mut score = -CHECKMATE_SCORE;
        let mut max_score = score;
        let mut flag = HashAlpha;
        let is_endgame = self.board.is_endgame();
        let moves = self.get_sorted_root_node_moves();
        for (move_index, &(move_, _)) in moves.iter().enumerate() {
            if !is_endgame && self.is_draw_move(move_.into()) && max_score > -DRAW_SCORE {
                continue;
            }
            let clock = Instant::now();
            self.push_unchecked(move_);
            if move_index == 0
                || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, enable_timer)? > alpha
            {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, enable_timer)?;
                max_score = max_score.max(score);
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
        if !self.timer.check_stop(enable_timer) {
            self.transposition_table
                .write(key, depth, self.ply, alpha, flag, self.get_best_move());
        }
        self.update_best_moves();
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
        enable_timer: bool,
    ) -> Option<Score> {
        self.pv_table.set_length(self.ply, self.ply);
        let mate_score = CHECKMATE_SCORE - self.ply as Score;
        if self.board.is_other_draw() {
            return Some(0);
        }
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
            return Some(self.board.evaluate_flipped());
        }
        // enable_timer &= depth > 3;
        if self.timer.check_stop(enable_timer) {
            return None;
        }
        if depth == 0 {
            return Some(self.quiescence(alpha, beta));
        }
        self.num_nodes_searched.fetch_add(1, MEMORY_ORDERING);
        let not_in_check = checkers.is_empty();
        let mut futility_pruning = false;
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
                // let r = NULL_MOVE_MIN_REDUCTION
                //     + (depth.max(NULL_MOVE_MIN_DEPTH) as f64 / NULL_MOVE_DEPTH_DIVIDER as f64)
                //         .round() as Depth;
                // let reduced_depth = depth - r - 1;
                let r = 1920 + (depth as i32) * 2368;
                let reduced_depth = (((depth as u32) * 4096 - (r as u32)) / 4096) as Depth;
                self.push_unchecked(ValidOrNullMove::NullMove);
                let score = -self.alpha_beta(reduced_depth, -beta, -beta + 1, enable_timer)?;
                self.pop();
                if score >= beta {
                    return Some(beta);
                }
            }
            // razoring
            let d = 3;
            if !is_pv_node && depth <= d && !is_checkmate(beta) {
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
            // futility pruning condition
            if depth < 4 && alpha < mate_score {
                let futility_margin = match depth {
                    0 => 0,
                    1 => PAWN_VALUE,
                    2 => Knight.evaluate(),
                    3 => Rook.evaluate(),
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
            Evaluator::is_easily_winning_position(&self.board, self.board.get_material_score()),
        );
        if weighted_moves.is_empty() {
            return if not_in_check {
                Some(0)
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
                && !DISABLE_LMR
                && not_an_interesting_position;
            self.push_unchecked(move_);
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
        if !self.timer.check_stop(enable_timer) {
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
            // delta pruning
            let mut delta = Queen.evaluate();
            if let Some(piece) = move_.get_promotion() {
                delta += piece.evaluate() - PAWN_VALUE;
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
        self.transposition_table.get_hash_full()
    }

    pub fn get_num_overwrites(&self) -> usize {
        self.transposition_table.get_num_overwrites()
    }

    pub fn get_num_collisions(&self) -> usize {
        self.transposition_table.get_num_collisions()
    }

    pub fn get_zero_hit(&self) -> usize {
        self.transposition_table.get_zero_hit()
    }

    pub fn get_pv(&self) -> Vec<&Move> {
        self.pv_table.get_pv(0)
    }

    pub fn get_pv_from_t_table(&self) -> Vec<Move> {
        extract_pv_from_t_table(&self.initial_sub_board, &self.transposition_table)
            .into_iter()
            .map_into()
            .collect_vec()
    }

    pub fn get_nth_pv_move(&self, n: usize) -> Option<Move> {
        Some(**self.pv_table.get_pv(0).get(n)?)
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

    pub fn get_current_depth(&self) -> Depth {
        self.current_depth
    }

    pub fn get_search_info(&self) -> SearchInfo {
        SearchInfo::new(self)
    }

    pub fn go(&mut self, mut command: GoCommand, print_info: bool) {
        if self.board.generate_legal_moves().len() == 1 {
            command = GoCommand::Depth(1);
        } else if command.is_timed() || command.is_move_time() {
            self.timer.parse_time_based_command(&self.board, command);
        }
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        self.current_depth = 1;
        while self.current_depth < Depth::MAX {
            if self.timer.check_stop(true) {
                break;
            }
            let last_score = self.score;
            self.score = self
                .search(self.current_depth, alpha, beta, print_info)
                .unwrap_or(self.score);
            let search_info = self.get_search_info();
            if print_info && self.is_main_threaded() {
                search_info.print_info();
            }
            if self.score <= alpha || self.score >= beta {
                if print_info && self.is_main_threaded() {
                    search_info.print_warning_message(alpha, beta);
                }
                alpha = -INFINITY;
                beta = INFINITY;
                self.score = last_score;
                continue;
            } else if self.is_main_threaded() {
                self.timer.update_max_time(self.current_depth, self.score);
            }
            let cutoff = if is_checkmate(self.score) {
                5
            } else {
                ASPIRATION_WINDOW_CUTOFF
            };
            alpha = self.score - cutoff;
            beta = self.score + cutoff;
            if command == GoCommand::Depth(self.current_depth) {
                break;
            }
            self.current_depth += 1;
        }
        self.current_depth -= 1;
    }
}

impl SearcherMethodOverload<Move> for Searcher {
    fn push_unchecked(&mut self, move_: Move) {
        self.board.push_unchecked(move_);
        self.ply += 1;
    }
}

impl SearcherMethodOverload<ValidOrNullMove> for Searcher {
    fn push_unchecked(&mut self, valid_or_null_move: ValidOrNullMove) {
        self.board.push_unchecked(valid_or_null_move);
        self.ply += 1;
    }
}
