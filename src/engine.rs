use super::*;
use sort::*;
use transposition_table::*;
use EntryFlag::*;

mod sort {
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
    pub struct WeightedMoveList {
        weighted_moves: [WeightedMove; MAX_MOVES_PER_POSITION],
        len: usize,
        idx: usize,
    }

    impl WeightedMoveList {
        pub fn next_best(&mut self) -> Option<WeightedMove> {
            if self.idx == self.len {
                return None;
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
            self.weighted_moves.swap(self.idx, max_idx);
            let best_move = self.weighted_moves[self.idx];
            self.idx += 1;
            Some(best_move)
        }
    }

    impl FromIterator<WeightedMove> for WeightedMoveList {
        fn from_iter<T: IntoIterator<Item = WeightedMove>>(iter: T) -> Self {
            let mut weighted_moves = [WeightedMove::default(); MAX_MOVES_PER_POSITION];
            let mut len = 0;
            for _move in iter {
                weighted_moves[len] = _move;
                len += 1;
            }
            Self {
                weighted_moves,
                len,
                idx: 0,
            }
        }
    }

    #[derive(Debug)]
    pub struct MoveSorter {
        killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH],
        history_move_scores: [[[MoveWeight; 64]; 2]; 6],
    }

    impl MoveSorter {
        pub fn update_killer_moves(&mut self, killer_move: Move, ply: Ply) {
            let arr = &mut self.killer_moves[ply];
            arr.rotate_right(1);
            arr[0] = killer_move;
        }

        pub fn is_killer_move(&self, _move: &Move, ply: Ply) -> bool {
            self.killer_moves[ply].contains(_move)
        }

        pub fn add_history_move(&mut self, history_move: Move, board: &Board, depth: Depth) {
            let depth = depth.pow(2) as MoveWeight;
            let src = history_move.get_source();
            let dest = history_move.get_dest();
            self.history_move_scores[board.piece_at(src).unwrap().to_index()]
                [board.color_at(src).unwrap() as usize][dest.to_index()] += depth;
        }

        fn get_least_attackers_move(&self, square: Square, board: &chess::Board) -> Option<Move> {
            let mut captute_moves = chess::MoveGen::new_legal(board);
            captute_moves.set_iterator_mask(get_square_bb(square));
            // match captute_moves.next() {
            //     Some(_move) => match _move.get_promotion() {
            //         Some(promotion) => Some(Move::new(
            //             _move.get_source(),
            //             _move.get_dest(),
            //             Some(Queen),
            //         )),
            //         None => Some(_move),
            //     },
            //     None => None,
            // }
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
        fn capture_value(&self, _move: Move, board: &Board) -> MoveWeight {
            self.see_capture(_move.get_dest(), &mut board.get_sub_board()) as MoveWeight
            // self.mvv_lva(_move, board)
        }

        fn threat_value(&self, _move: Move, sub_board: &chess::Board) -> MoveWeight {
            let dest = _move.get_dest();
            let attacker_piece_score =
                evaluate_piece(sub_board.piece_on(_move.get_source()).unwrap());
            let attacked_piece_score = evaluate_piece(sub_board.piece_on(dest).unwrap_or(Pawn));
            let mut threat_score = attacker_piece_score - attacked_piece_score;
            if sub_board.pinned() == &get_square_bb(dest) {
                threat_score *= 2;
            }
            (threat_score + 16 * PAWN_VALUE) as MoveWeight
        }

        fn move_value(
            &self,
            _move: Move,
            board: &Board,
            ply: Ply,
            best_move: Option<Move>,
        ) -> MoveWeight {
            if let Some(m) = best_move {
                if m == _move {
                    return 1294000000;
                }
            }
            let source = _move.get_source();
            let dest = _move.get_dest();
            let mut sub_board = board.get_sub_board();
            sub_board.clone().make_move(_move, &mut sub_board);
            let checkers = *sub_board.checkers();
            let moving_piece = board.piece_at(source).unwrap();
            if checkers != BB_EMPTY {
                return 1292000000
                    + 100 * checkers.popcnt() as MoveWeight
                    + moving_piece as MoveWeight;
            }
            if board.is_capture(_move) {
                let capture_value = self.capture_value(_move, board);
                return 1291000000 + capture_value;
            }
            for (i, killer_move) in self.killer_moves[ply].iter().enumerate() {
                if killer_move == &_move {
                    return 1290000000 - i as MoveWeight;
                }
            }
            if let Some(piece) = _move.get_promotion() {
                return 1289000000;
            }
            if board.is_endgame() && board.is_passed_pawn(source) {
                let promotion_distance = if board.turn() == White {
                    7 - dest.get_rank() as MoveWeight
                } else {
                    dest.get_rank() as MoveWeight
                };
                return 1288000000 - promotion_distance;
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
            self.history_move_scores[moving_piece.to_index()]
                [board.color_at(source).unwrap() as usize][dest.to_index()]
        }

        pub fn get_weighted_sort_moves<T: IntoIterator<Item = Move>>(
            &self,
            move_gen: T,
            board: &Board,
            ply: Ply,
            best_move: Option<Move>,
        ) -> WeightedMoveList {
            WeightedMoveList::from_iter(
                move_gen
                    .into_iter()
                    .map(|m| WeightedMove::new(m, self.move_value(m, board, ply, best_move))),
            )
        }

        pub fn get_weighted_capture_moves<T: IntoIterator<Item = Move>>(
            &self,
            move_gen: T,
            board: &Board,
        ) -> WeightedMoveList {
            WeightedMoveList::from_iter(move_gen.into_iter().map(|m| {
                WeightedMove::new(
                    m,
                    self.see_capture(m.get_dest(), &mut board.get_sub_board()) as MoveWeight,
                )
            }))
        }
    }

    impl Default for MoveSorter {
        fn default() -> Self {
            Self {
                killer_moves: [[Move::default(); NUM_KILLER_MOVES]; MAX_DEPTH],
                history_move_scores: [[[0; 64]; 2]; 6],
            }
        }
    }
}

mod transposition_table {
    use std::collections::hash_map::Entry;

    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum EntryFlag {
        HashExact,
        HashAlpha,
        HashBeta,
    }

    impl Default for EntryFlag {
        fn default() -> Self {
            HashExact
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct TranspositionTableData {
        depth: Depth,
        score: Score,
        flag: EntryFlag,
    }

    impl TranspositionTableData {
        pub fn depth(&self) -> Depth {
            self.depth
        }

        pub fn score(&self) -> Score {
            self.score
        }

        pub fn flag(&self) -> EntryFlag {
            self.flag
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct TranspositionTableEntry {
        option_data: Option<TranspositionTableData>,
        best_move: Option<Move>,
    }

    #[derive(Debug)]
    pub struct TranspositionTable {
        table: Arc<Mutex<HashMap<u64, TranspositionTableEntry>>>,
    }

    impl TranspositionTable {
        pub fn new() -> Self {
            Self {
                table: Arc::new(Mutex::new(HashMap::default())),
            }
        }

        pub fn read(
            &self,
            key: u64,
            depth: Depth,
            alpha: Score,
            beta: Score,
        ) -> Option<(Option<Score>, Option<Move>)> {
            if DISABLE_T_TABLE {
                return None;
            }
            match self.table.lock().unwrap().get(&key) {
                Some(tt_entry) => {
                    let best_move = tt_entry.best_move;
                    if let Some(data) = tt_entry.option_data {
                        if data.depth >= depth {
                            return match data.flag {
                                HashExact => Some((Some(data.score), best_move)),
                                HashAlpha => {
                                    if data.score <= alpha {
                                        Some((Some(data.score), best_move))
                                    } else {
                                        Some((None, best_move))
                                    }
                                }
                                HashBeta => {
                                    if data.score >= beta {
                                        Some((Some(data.score), best_move))
                                    } else {
                                        Some((None, best_move))
                                    }
                                }
                            };
                        }
                    }
                    Some((None, best_move))
                }
                None => None,
            }
        }

        pub fn read_best_move(&self, key: u64) -> Option<Move> {
            match self.table.lock().unwrap().get(&key) {
                Some(tt_entry) => tt_entry.best_move,
                None => None,
            }
        }

        fn update_tt_entry(
            tt_entry: &mut TranspositionTableEntry,
            option_data: Option<TranspositionTableData>,
            best_move: Option<Move>,
        ) {
            tt_entry.best_move = best_move;

            if let Some(data) = tt_entry.option_data {
                if let Some(curr_data) = option_data {
                    if data.depth < curr_data.depth {
                        tt_entry.option_data = option_data;
                    }
                }
            } else {
                tt_entry.option_data = option_data;
            }

            // tt_entry.option_data = option_data;
        }

        pub fn write(
            &self,
            key: u64,
            depth: Depth,
            score: Score,
            flag: EntryFlag,
            best_move: Option<Move>,
        ) {
            // let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score) || score.abs() < PAWN_VALUE / 10;
            let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score);
            let option_data = if not_to_save_score {
                None
            } else {
                Some(TranspositionTableData { depth, score, flag })
            };
            let mut table_entry = self.table.lock().unwrap();
            match table_entry.entry(key) {
                Entry::Occupied(tt_entry) => {
                    let tt_entry = tt_entry.into_mut();
                    Self::update_tt_entry(tt_entry, option_data, best_move);
                }
                Entry::Vacant(tt_entry) => {
                    tt_entry.insert(TranspositionTableEntry {
                        option_data,
                        best_move,
                    });
                }
            }
        }

        pub fn clear(&self) {
            self.table.lock().unwrap().clear();
        }
    }

    impl Default for TranspositionTable {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[derive(Debug)]
pub struct Engine {
    pub board: Board,
    num_nodes_searched: usize,
    ply: Ply,
    pv_length: [usize; MAX_DEPTH],
    pv_table: [[Move; MAX_DEPTH]; MAX_DEPTH],
    move_sorter: MoveSorter,
    transposition_table: TranspositionTable,
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            num_nodes_searched: 0,
            ply: 0,
            pv_length: [0; MAX_DEPTH],
            pv_table: [[Move::default(); MAX_DEPTH]; MAX_DEPTH],
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
        for i in 0..MAX_DEPTH {
            self.pv_length[i] = 0;
            for j in 0..MAX_DEPTH {
                self.pv_table[i][j] = Move::default();
            }
        }
        self.move_sorter = MoveSorter::default();
        self.transposition_table = TranspositionTable::default();
    }

    fn update_pv_table(&mut self, _move: Move) {
        self.pv_table[self.ply][self.ply] = _move;
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
        let mut score = -CHECKMATE_SCORE;
        let mut flag = HashAlpha;
        let mut weighted_moves = self.move_sorter.get_weighted_sort_moves(
            self.board.generate_legal_moves(),
            &self.board,
            self.ply,
            self.transposition_table.read_best_move(key),
        );
        let mut move_index = 0;
        while let Some(weighted_move) = weighted_moves.next_best() {
            let _move = weighted_move._move;
            self.push(_move);
            if move_index == 0 || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, true) > alpha {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, true);
            }
            self.pop();
            if score > alpha {
                flag = HashExact;
                alpha = score;
                self.update_pv_table(_move);
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, beta, HashBeta, Some(_move));
                    return beta;
                }
            }
            move_index += 1;
        }
        self.transposition_table.write(
            key,
            depth,
            alpha,
            flag,
            Some(self.pv_table[self.ply][self.ply]),
        );
        alpha
    }

    fn alpha_beta(
        &mut self,
        depth: Depth,
        mut alpha: Score,
        mut beta: Score,
        apply_null_move: bool,
    ) -> Score {
        self.num_nodes_searched += 1;
        self.pv_length[self.ply] = self.ply;
        let num_pieces = self.board.get_num_pieces();
        let is_endgame = num_pieces <= ENDGAME_PIECE_THRESHOLD;
        let not_in_check = !self.board.is_check();
        let draw_score = if is_endgame { 0 } else { DRAW_SCORE };
        let is_pvs_node = alpha != beta - 1;
        let is_draw = if is_pvs_node {
            self.board.is_insufficient_material()
        } else {
            self.board.is_other_draw()
        };
        if is_draw {
            return draw_score;
        }
        // if !not_in_check {
        //     depth += 1
        // }
        let key = self.board.get_hash();
        let best_move = if is_pvs_node {
            self.transposition_table.read_best_move(key)
        } else {
            match self.transposition_table.read(key, depth, alpha, beta) {
                Some((Some(score), _)) => return score,
                Some((None, best_move)) => best_move,
                None => None,
            }
        };
        let mate_score = CHECKMATE_SCORE - self.ply as i16;
        let moves_gen = self.board.generate_legal_moves();
        if moves_gen.len() == 0 {
            if not_in_check {
                return draw_score;
            }
            return -mate_score;
        }
        if depth <= 0 {
            return self.quiescence(alpha, beta);
        }
        // mate distance pruning
        alpha = alpha.max(-mate_score);
        beta = beta.min(mate_score - 1);
        if alpha >= beta {
            return alpha;
        }
        let mut futility_pruning = false;
        if not_in_check && !is_pvs_node && !is_endgame {
            // static evaluation pruning
            let static_evaluation = self.board.evaluate_flipped();
            if depth < 3 && (beta - 1).abs() > -mate_score + PAWN_VALUE {
                let evaluation_margin = PAWN_VALUE * depth as i16;
                let evaluation_diff = static_evaluation - evaluation_margin;
                if evaluation_diff >= beta {
                    return evaluation_diff;
                }
            }
            // null move reduction
            if apply_null_move && static_evaluation > beta {
                let r = NULL_MOVE_MIN_REDUCTION
                    + (depth - NULL_MOVE_MIN_DEPTH) / NULL_MOVE_DEPTH_DIVIDER;
                // let r = NULL_MOVE_MIN_REDUCTION;
                if depth > r {
                    self.push_null_move();
                    let score = -self.alpha_beta(depth - 1 - r, -beta, -beta + 1, false);
                    self.pop_null_move();
                    if score >= beta {
                        return beta;
                    }
                }
                // razoring
                if !is_pvs_node && self.board.has_non_pawn_material() {
                    let mut evaluation = static_evaluation + PAWN_VALUE;
                    if evaluation < beta && depth == 1 {
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
            // futility pruning
            if depth < 4 && alpha < mate_score {
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
        let mut weighted_moves =
            self.move_sorter
                .get_weighted_sort_moves(moves_gen, &self.board, self.ply, best_move);
        let mut move_index = 0;
        while let Some(weighted_move) = weighted_moves.next_best() {
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
                score = -self.alpha_beta(depth - 1, -beta, -alpha, true);
            } else {
                if move_index >= FULL_DEPTH_SEARCH_LMR
                    && depth >= REDUCTION_LIMIT_LMR
                    && safe_to_apply_lmr
                    && !DISABLE_LMR
                {
                    // let lmr_reduction = if is_endgame {
                    //     1 + ((move_index - 1) as f32).sqrt() as Depth
                    // } else {
                    //     (LMR_BASE_REDUCTION + ((depth as usize * move_index) as f32).sqrt() / LMR_MOVE_DIVIDER) as Depth
                    // };
                    let lmr_reduction = (LMR_BASE_REDUCTION
                        + ((depth as usize * move_index) as f32).sqrt() / LMR_MOVE_DIVIDER)
                        as Depth;
                    score = -self.alpha_beta(depth - 1 - lmr_reduction, -alpha - 1, -alpha, true);
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    score = -self.alpha_beta(depth - 1, -alpha - 1, -alpha, true);
                    if score > alpha && score < beta {
                        score = -self.alpha_beta(depth - 1, -beta, -alpha, true);
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
                    self.transposition_table
                        .write(key, depth, beta, HashBeta, Some(_move));
                    if not_capture_move {
                        self.move_sorter.update_killer_moves(_move, self.ply);
                    }
                    return beta;
                }
            }
            move_index += 1;
        }
        self.transposition_table.write(
            key,
            depth,
            alpha,
            flag,
            Some(self.pv_table[self.ply][self.ply]),
        );
        alpha
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
        let mut weighted_moves = self
            .move_sorter
            .get_weighted_capture_moves(self.board.generate_legal_captures(), &self.board);
        while let Some(weighted_move) = weighted_moves.next_best() {
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
            pv.push(self.pv_table[depth as usize][i]);
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
                board.san_and_push(_move)
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

    pub fn get_best_move(&self) -> Move {
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
        self.transposition_table.clear();
        let mut current_depth = 1;
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut score;
        let clock = Instant::now();
        loop {
            // self.follow_pv = true;
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
        (self.get_best_move(), score)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::new())
    }
}
