use super::*;
use sort::*;
use transpositionT_table::*;
use EntryFlag::*;

mod sort {
    use super::*;
    pub struct MoveSorter {
        killer_moves: [[Move; NUM_KILLER_MOVES]; MAX_DEPTH],
        history_moves: [[[u32; 64]; 2]; 6],
    }
    
    impl MoveSorter {
        pub fn update_killer_moves(&mut self, killer_move: Move, ply: Ply) {
            self.killer_moves[ply].rotate_right(1);
            self.killer_moves[ply][0] = killer_move;
        }
    
        pub fn add_history_move(&mut self, history_move: Move, board: &Board, depth: Depth) {
            let depth = depth as u32;
            let src = history_move.get_source();
            let dest = history_move.get_dest();
            self.history_moves[board.piece_at(src).unwrap().to_index()]
                [board.color_at(src).unwrap() as usize][dest.to_index()] += depth;
        }
    
        fn get_least_attackers_square(&self, square: Square, board: &chess::Board) -> Option<Square> {
            let mut capture_moves = chess::MoveGen::new_legal(&board);
            capture_moves.set_iterator_mask(BB_SQUARES[square.to_index()]);
            let mut least_attackers_square = None;
            let mut least_attacker_type = 6;
            for square in capture_moves.into_iter().map(|m| m.get_source()) {
                let attacker_type = board.piece_on(square).unwrap() as u8;
                if attacker_type == 0 {
                    return Some(square);
                }
                if attacker_type < least_attacker_type {
                    least_attackers_square = Some(square);
                    least_attacker_type = attacker_type;
                }
            }
            least_attackers_square
        }
    
        fn see(&self, square: Square, board: &mut chess::Board, evaluator: &Evaluator) -> Score {
            let least_attackers_square = match self.get_least_attackers_square(square, &board) {
                Some(square) => square,
                None => return 0,
            };
            let capture_piece = board.piece_on(square).unwrap_or(Pawn);
            board
                .clone()
                .make_move(Move::new(least_attackers_square, square, None), board);
            (evaluator.evaluate_piece(capture_piece) - self.see(square, board, evaluator)).max(0)
        }
    
        fn see_capture(&self, square: Square, board: &mut chess::Board, evaluator: &Evaluator) -> Score {
            let least_attackers_square = match self.get_least_attackers_square(square, &board) {
                Some(square) => square,
                None => return 0,
            };
            let capture_piece = board.piece_on(square).unwrap_or(Pawn);
            board
                .clone()
                .make_move(Move::new(least_attackers_square, square, None), board);
            evaluator.evaluate_piece(capture_piece) - self.see(square, board, evaluator)
        }
    
        fn capture_value(&self, _move: Move, board: &Board) -> u32 {
            // (self.see_capture(_move.get_dest(), &mut board.get_sub_board(), &board.evaluator) + 900) as u32
            MVV_LVA[board.piece_at(_move.get_source()).unwrap().to_index()]
                [board.piece_at(_move.get_dest()).unwrap_or(Pawn).to_index()]
        }
    
        fn move_value(&self, _move: Move, board: &Board, ply: Ply, best_move: Option<Move>) -> u32 {
            match best_move {
                Some(m) => {
                    if m == _move {
                        return 4294000000;
                    }
                },
                None => {},
            }
            let mut sub_board = board.get_sub_board();
            sub_board.clone().make_move(_move, &mut sub_board);
            let checkers = *sub_board.checkers();
            if checkers != BB_EMPTY {
                return 4292000000 + checkers.popcnt();
            }
            if board.is_capture(_move) {
                return 4291000000 + self.capture_value(_move, board);
            }
            for (i, killer_move) in self.killer_moves[ply].iter().enumerate() {
                if killer_move == &_move {
                    return 4290000000 - i as u32;
                }
            }
            match _move.get_promotion() {
                Some(piece) => return 4289000000 + piece as u32,
                None => {}
            }
            let history_moves_score = self.history_moves
                [board.piece_at(_move.get_source()).unwrap().to_index()]
                [board.color_at(_move.get_source()).unwrap() as usize][_move.get_dest().to_index()];
            history_moves_score
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

mod transpositionT_table {
    use super::*;

    #[derive(Clone)]
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

        pub fn read(&self, key: u64, depth: Depth, alpha: Score, beta: Score) -> Option<(Option<Score>, Option<Move>)> {
            if DISABLE_T_TABLE {
                return None;
            }
            match self.table.lock().unwrap().get(&key) {
                Some(tt_entry) => {
                    let best_move = tt_entry.best_move;
                    if tt_entry.depth >= depth {
                        match tt_entry.flag {
                            HashExact => Some((Some(tt_entry.score), best_move)),
                            HashAlpha => {
                                if tt_entry.score <= alpha {
                                    Some((Some(tt_entry.score), best_move))
                                } else {
                                    Some((None, best_move))
                                }
                            }
                            HashBeta => {
                                if tt_entry.score >= beta {
                                    Some((Some(tt_entry.score), best_move))
                                } else {
                                    Some((None, best_move))
                                }
                            }
                        }
                    } else {
                        return Some((None, best_move));
                    }
                }
                None => None,
            }
        }
        
        pub fn write(&self, key: u64, depth: Depth, score: Score, flag: EntryFlag, best_move: Option<Move>) {
            // let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score) || score.abs() < 5;
            let not_to_save_score = DISABLE_T_TABLE || is_checkmate(score);
            if not_to_save_score {
                return;
            }
            let mut table_entry = self.table.lock().unwrap();
            let mut tt_entry = table_entry.entry(key).or_insert(TranspositionTableEntry{
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
    }


    impl Default for TranspositionTable {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub struct Engine {
    pub board: Board,
    num_nodes_searched: u32,
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

    fn reset_constants(&mut self) {
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

    fn search(&mut self, depth: Depth, alpha: Score, beta: Score) -> (Option<Move>, Score) {
        self.pv_length[self.ply] = self.ply;
        if self.board.is_game_over() {
            return (
                None,
                if self.board.is_checkmate() {
                    -CHECKMATE_SCORE
                } else {
                    0
                },
            );
        }
        let mut alpha = alpha;
        let moves =
            self.move_sorter
                .sort_moves(self.board.generate_legal_moves(), &self.board, self.ply, None);
        for _move in moves {
            self.push(_move);
            let score = -self.alpha_beta(depth - 1, -beta, -alpha, true);
            self.pop();
            if score > alpha {
                alpha = score;
                self.update_pv_table(_move);
                if score >= beta {
                    return (Some(_move), beta);
                }
            }
        }
        (Some(self.get_best_move()), alpha)
    }

    fn alpha_beta(&mut self, depth: Depth, alpha: Score, beta: Score, apply_null_move: bool) -> Score {
        self.pv_length[self.ply] = self.ply;
        let not_in_check = !self.board.is_check();
        let draw_score = 0;
        if self.board.is_other_draw() {
            return draw_score;
        }
        let best_move = match self.transposition_table.read(self.board.get_hash(), depth, alpha, beta) {
            Some((Some(score), best_move)) => {
                if best_move.is_some() {
                    self.update_pv_table(best_move.unwrap());
                }
                return score;
            }
            Some((None, best_move)) => {
                best_move
            }
            None => None,
        };
        if depth == 0 {
            return self.quiescence(alpha, beta);
        }
        let moves_gen = self.board.generate_legal_moves();
        if moves_gen.len() == 0 {
            if not_in_check {
                return draw_score;
            }
            return -CHECKMATE_SCORE + self.ply as Score;
        }
        self.num_nodes_searched += 1;
        let mut alpha = alpha;
        let mut flag = HashAlpha;
        let moves = self
            .move_sorter
            .sort_moves(moves_gen, &self.board, self.ply, best_move);
        for _move in moves {
            let not_capture_move = !self.board.is_capture(_move);
            self.push(_move);
            let score = -self.alpha_beta(depth - 1, -beta, -alpha, apply_null_move);
            self.pop();
            if score > alpha {
                flag = HashExact;
                self.update_pv_table(_move);
                alpha = score;
                if not_capture_move {
                    let depth_u32 = depth as u32;
                    self.move_sorter.add_history_move(_move, &self.board, depth);
                }
                if score >= beta {
                    self.transposition_table.write(
                        self.board.get_hash(),
                        depth,
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
            self.board.get_hash(),
            depth,
            alpha,
            flag,
            Some(self.pv_table[self.ply][self.ply]),
        );
        alpha
    }

    fn quiescence(&mut self, alpha: Score, beta: Score) -> Score {
        let mut alpha = alpha;
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

    fn get_pv_as_san(&self, depth: u8) -> String {
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

    fn get_pv_as_lan(&self, depth: u8) -> String {
        let mut board = self.board.clone();
        let mut pv_string = String::new();
        for _move in self.get_pv(depth) {
            pv_string += (if board.is_legal(_move) {
                board.lan_and_push(_move)
            } else {
                colorize(_move, ERROR_MESSAGE_STYLE)
            })
            .as_str();
            pv_string += " ";
        }
        return pv_string.trim().to_string();
    }

    pub fn get_pv_string(&self) -> String {
        self.get_pv_as_san(0)
    }

    pub fn get_num_nodes_searched(&self) -> u32 {
        self.num_nodes_searched
    }

    pub fn get_best_move(&self) -> Move {
        return self.pv_table[0][0];
    }

    pub fn get_best_move_and_score(&mut self, depth: u8) -> (Move, Score) {
        self.reset_constants();
        let (best_move, score) = self.search(depth, -INFINITY, INFINITY);
        (
            best_move.unwrap(),
            if self.board.turn() == White {
                score
            } else {
                -score
            },
        )
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Board::new())
    }
}
