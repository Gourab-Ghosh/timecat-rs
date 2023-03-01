use super::*;
use sort::*;
use transposition_table::*;
use EntryFlag::*;

mod sort {
    use super::*;
    pub struct MoveSorter {
        killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH],
        history_moves: [[[u32; 64]; 2]; 6],
    }

    impl MoveSorter {
        pub fn update_killer_moves(&mut self, killer_move: Move, ply: Ply) {
            let arr = &mut self.killer_moves[ply];
            arr.rotate_right(1);
            arr[0] = killer_move;
        }

        pub fn add_history_move(&mut self, history_move: Move, board: &Board, depth: Depth) {
            let depth = depth as u32;
            let src = history_move.get_source();
            let dest = history_move.get_dest();
            self.history_moves[board.piece_at(src).unwrap().to_index()]
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

        fn mvv_lva(&self, _move: Move, board: &Board) -> u32 {
            MVV_LVA[board
                .piece_at(_move.get_source())
                .unwrap_or(Pawn)
                .to_index()][board.piece_at(_move.get_dest()).unwrap().to_index()]
        }

        #[inline(always)]
        fn capture_value(&self, _move: Move, board: &Board) -> u32 {
            (self.see_capture(_move.get_dest(), &mut board.get_sub_board()) + 9 * PAWN_VALUE) as u32
        }

        fn threat_value(&self, _move: Move, sub_board: &chess::Board) -> u32 {
            let dest = _move.get_dest();
            let attacker_piece_score =
                evaluate_piece(sub_board.piece_on(_move.get_source()).unwrap());
            let attacked_piece_score = evaluate_piece(sub_board.piece_on(dest).unwrap_or(Pawn));
            let mut threat_score = attacker_piece_score - attacked_piece_score;
            if sub_board.pinned() == &get_square_bb(dest) {
                threat_score *= 2;
            }
            (threat_score + 16 * PAWN_VALUE) as u32
        }

        fn move_value(&self, _move: Move, board: &Board, ply: Ply, best_move: Option<Move>) -> u32 {
            if let Some(m) = best_move {
                if m == _move {
                    return 4294000000;
                }
            }
            let mut sub_board = board.get_sub_board();
            sub_board.clone().make_move(_move, &mut sub_board);
            let checkers = *sub_board.checkers();
            if checkers != BB_EMPTY {
                return 4292000000 + checkers.popcnt();
            }
            if board.is_capture(_move) {
                let capture_value = self.capture_value(_move, board);
                return 4291000000 + capture_value;
            }
            for (i, killer_move) in self.killer_moves[ply].iter().enumerate() {
                if killer_move == &_move {
                    return 4290000000 - i as u32;
                }
            }
            if let Some(piece) = _move.get_promotion() {
                return 4289000000 + piece as u32;
            }
            // let threat_score = match sub_board.null_move() {
            //     Some(board) => {
            //         let mut moves = chess::MoveGen::new_legal(&board);
            //         moves.set_iterator_mask(*board.color_combined(!board.side_to_move()));
            //         moves.into_iter().map(|m| self.threat_value(m, &board)).sum::<u32>()
            //     },
            //     None => 0,
            // };

            // 10 * history_moves_score + threat_score
            self.history_moves[board.piece_at(_move.get_source()).unwrap().to_index()]
                [board.color_at(_move.get_source()).unwrap() as usize][_move.get_dest().to_index()]
        }

        pub fn sort_moves<T: IntoIterator<Item = Move>>(
            &self,
            move_gen: T,
            board: &Board,
            ply: Ply,
            best_move: Option<Move>,
        ) -> Vec<Move> {
            let mut moves = Vec::new();
            let mut moves_score_dict = HashMap::default();
            for _move in move_gen {
                moves.push(_move);
                moves_score_dict.insert(_move, self.move_value(_move, board, ply, best_move));
            }
            moves.sort_by(|a, b| moves_score_dict[b].cmp(&moves_score_dict[a]));
            moves
        }

        pub fn sort_captures<T: IntoIterator<Item = Move>>(
            &self,
            move_gen: T,
            board: &Board,
        ) -> Vec<Move> {
            let mut moves = Vec::new();
            let mut moves_score_dict = HashMap::default();
            for _move in move_gen {
                moves.push(_move);
                moves_score_dict.insert(_move, self.capture_value(_move, board));
            }
            moves.sort_by(|a, b| moves_score_dict[b].cmp(&moves_score_dict[a]));
            moves
        }
    }

    impl Default for MoveSorter {
        fn default() -> Self {
            Self {
                killer_moves: [[Move::default(); NUM_KILLER_MOVES]; MAX_DEPTH],
                history_moves: [[[0; 64]; 2]; 6],
            }
        }
    }
}

mod transposition_table {
    use super::*;

    #[derive(Clone)]
    pub enum EntryFlag {
        ExactHAsh,
        AlphaHash,
        BetaHash,
    }

    impl Default for EntryFlag {
        fn default() -> Self {
            ExactHAsh
        }
    }

    #[derive(Default)]
    struct TranspositionTableEntry {
        depth: Depth,
        score: Score,
        flag: EntryFlag,
        best_move: Option<Move>,
    }

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
                    if tt_entry.depth >= depth {
                        match tt_entry.flag {
                            ExactHAsh => Some((Some(tt_entry.score), best_move)),
                            AlphaHash => {
                                if tt_entry.score <= alpha {
                                    Some((Some(tt_entry.score), best_move))
                                } else {
                                    Some((None, best_move))
                                }
                            }
                            BetaHash => {
                                if tt_entry.score >= beta {
                                    Some((Some(tt_entry.score), best_move))
                                } else {
                                    Some((None, best_move))
                                }
                            }
                        }
                    } else {
                        Some((None, best_move))
                    }
                }
                None => None,
            }
        }

        pub fn read_best_move(&self, key: u64) -> Option<Move> {
            if DISABLE_T_TABLE {
                return None;
            }
            match self.table.lock().unwrap().get(&key) {
                Some(tt_entry) => tt_entry.best_move,
                None => None,
            }
        }

        pub fn write(
            &self,
            key: u64,
            depth: Depth,
            score: Score,
            flag: EntryFlag,
            best_move: Option<Move>,
        ) {
            // let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score) || score.abs() < 5;
            let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score);
            if not_to_save_score {
                return;
            }
            let mut table_entry = self.table.lock().unwrap();
            let mut tt_entry = table_entry.entry(key).or_insert(TranspositionTableEntry {
                depth,
                score,
                flag: flag.clone(),
                best_move,
            });
            if tt_entry.depth >= depth {
                return;
            }
            tt_entry.depth = depth;
            tt_entry.score = score;
            tt_entry.flag = flag;
            tt_entry.best_move = best_move;
        }

        // pub fn write(&self, key: u64, depth: Depth, score: Score, flag: EntryFlag, best_move: Move) {
        //     // let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score) || score.abs() < 5;
        //     let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score);
        //     if not_to_save_score {
        //         return;
        //     }
        //     match self.table.lock().unwrap().get_mut(&key) {
        //         Some(tt_entry) => {
        //             if tt_entry.depth > depth {
        //                 return;
        //             }
        //             tt_entry.depth = depth;
        //             tt_entry.score = score;
        //             tt_entry.flag = flag;
        //             tt_entry.best_move = best_move;
        //         }
        //         None => {
        //             self.table.lock().unwrap().insert(key, TranspositionTableEntry {
        //                 depth,
        //                 score,
        //                 flag,
        //                 best_move,
        //             });
        //         }
        //     }
        // }

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
        let mut flag = AlphaHash;
        let moves = self.move_sorter.sort_moves(
            self.board.generate_legal_moves(),
            &self.board,
            self.ply,
            self.transposition_table.read_best_move(key),
        );
        for (i, &_move) in moves.iter().enumerate() {
            self.push(_move);
            if i == 0 || -self.alpha_beta(depth - 1, -alpha - 1, -alpha, true) > alpha {
                score = -self.alpha_beta(depth - 1, -beta, -alpha, true);
            }
            self.pop();
            if score > alpha {
                flag = ExactHAsh;
                alpha = score;
                self.update_pv_table(_move);
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, beta, BetaHash, Some(_move));
                    return beta;
                }
            }
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
        self.pv_length[self.ply] = self.ply;
        let is_endgame = self.board.is_endgame();
        let not_in_check = !self.board.is_check();
        let draw_score = if is_endgame { 0 } else { DRAW_SCORE };
        if self.board.is_other_draw() {
            return draw_score;
        }
        // let is_pvs_node = (beta as i32 - alpha as i32) > 1;
        let is_pvs_node = alpha != beta - 1;
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
        if depth == 0 {
            return self.quiescence(alpha, beta);
        }
        let mate_score = CHECKMATE_SCORE - self.ply as i16;
        let moves_gen = self.board.generate_legal_moves();
        if moves_gen.len() == 0 {
            if not_in_check {
                return draw_score;
            }
            return -mate_score;
        }
        // TODO: check if this is correct
        // mate distance pruning
        alpha = alpha.max(-mate_score);
        beta = beta.min(mate_score - 1);
        if alpha >= beta {
            return alpha;
        }
        // // mate distance pruning
        // if mate_score < beta {
        //     beta = mate_score;
        //     if alpha >= mate_score {
        //         return mate_score;
        //     }
        // }
        let mut futility_pruning = false;
        if not_in_check {
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
            if apply_null_move && depth > NULL_MOVE_REDUCTION_LIMIT {
                self.board.push_null_move();
                self.ply += 1;
                let score = -self.alpha_beta(
                    depth - 1 - NULL_MOVE_REDUCTION_LIMIT,
                    -beta,
                    -beta + 1,
                    false,
                );
                self.board.pop_null_move();
                self.ply -= 1;
                if score >= beta {
                    return beta;
                }
            }
            // futility pruning condition
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
        self.num_nodes_searched += 1;
        let mut flag = AlphaHash;
        let moves = self
            .move_sorter
            .sort_moves(moves_gen, &self.board, self.ply, best_move);
        for (move_index, &_move) in moves.iter().enumerate() {
            let not_capture_move = !self.board.is_capture(_move);
            if move_index != 0
                && futility_pruning
                && not_capture_move
                && _move.get_promotion().is_none()
                && not_in_check
            {
                continue;
            }
            self.push(_move);
            let score = -self.alpha_beta(depth - 1, -beta, -alpha, apply_null_move);
            self.pop();
            if score > alpha {
                flag = ExactHAsh;
                self.update_pv_table(_move);
                alpha = score;
                if not_capture_move {
                    self.move_sorter.add_history_move(_move, &self.board, depth);
                }
                if score >= beta {
                    self.transposition_table
                        .write(key, depth, beta, BetaHash, Some(_move));
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
            alpha,
            flag,
            Some(self.pv_table[self.ply][self.ply]),
        );
        alpha
    }

    fn quiescence(&mut self, mut alpha: Score, beta: Score) -> Score {
        if self.board.is_draw() {
            return DRAW_SCORE;
        }
        self.num_nodes_searched += 1;
        let evaluation = self.board.evaluate_flipped();
        if evaluation >= beta {
            return beta;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        for _move in self
            .move_sorter
            .sort_captures(self.board.generate_legal_captures(), &self.board)
        {
            self.push(_move);
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
            pv_string += (if board.is_legal(_move) {
                board.san_and_push(_move)
            } else {
                colorize(_move, ERROR_MESSAGE_STYLE)
            })
            .as_str();
            pv_string += " ";
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
            (self.num_nodes_searched as f64 / time_passed.as_secs_f64()) as u32,
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
