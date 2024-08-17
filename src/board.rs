#![doc = include_str!("../documentation/board/README.md")]

use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GameResult {
    Win(Color),
    Draw,
    InProgress,
}

impl GameResult {
    pub fn is_win(&self) -> bool {
        matches!(self, GameResult::Win(_))
    }

    pub fn is_draw(&self) -> bool {
        matches!(self, GameResult::Draw)
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self, GameResult::InProgress)
    }

    pub fn winner(&self) -> Option<Color> {
        match self {
            GameResult::Win(color) => Some(*color),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Board {
    mini_board: MiniBoard,
    stack: Vec<(MiniBoard, ValidOrNullMove)>,
    repetition_table: RepetitionTable,
    #[cfg(feature = "extras")]
    evaluator: Evaluator,
}

impl Board {
    pub fn new() -> Self {
        MiniBoard::from_str(STARTING_POSITION_FEN).unwrap().into()
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<()> {
        self.mini_board.set_fen(fen)?;
        self.stack.clear();
        self.update_repetition_table();
        Ok(())
    }

    pub fn from_fen(fen: &str) -> Result<Self> {
        let mut board = Self::new();
        board.set_fen(fen)?;
        Ok(board)
    }

    pub fn get_mini_board(&self) -> &MiniBoard {
        &self.mini_board
    }

    #[cfg(feature = "extras")]
    pub fn get_evaluator(&self) -> &Evaluator {
        &self.evaluator
    }

    #[cfg(feature = "extras")]
    pub fn get_evaluator_mut(&mut self) -> &mut Evaluator {
        &mut self.evaluator
    }

    pub fn reset(&mut self) {
        self.set_fen(STARTING_POSITION_FEN).unwrap();
    }

    pub fn clear(&mut self) {
        self.set_fen(EMPTY_FEN).unwrap();
    }

    pub fn flip_vertical(&mut self) {
        self.mini_board.flip_vertical();
        self.stack.clear();
        self.update_repetition_table();
    }

    pub fn flip_vertical_and_flip_turn_unchecked(&mut self) {
        self.mini_board.flip_vertical_and_flip_turn_unchecked();
        self.stack.clear();
        self.update_repetition_table();
    }

    pub fn flip_horizontal(&mut self) {
        self.mini_board.flip_horizontal();
        self.stack.clear();
        self.update_repetition_table();
    }

    #[inline]
    pub fn to_board_string(&self, use_unicode: bool) -> String {
        self.mini_board.to_board_string(
            self.stack.last().map_or(Default::default(), |(_, m)| *m),
            use_unicode,
        )
    }

    #[inline]
    pub fn to_unicode_string(&self) -> String {
        self.mini_board
            .to_unicode_string(self.stack.last().map_or(Default::default(), |(_, m)| *m))
    }

    pub fn result(&self) -> GameResult {
        if self.is_other_draw() {
            return GameResult::Draw;
        }
        match self.status() {
            BoardStatus::Checkmate => GameResult::Win(!self.turn()),
            BoardStatus::Stalemate => GameResult::Draw,
            BoardStatus::Ongoing => GameResult::InProgress,
        }
    }

    #[inline]
    pub fn get_num_moves(&self) -> NumMoves {
        self.stack.len() as NumMoves
    }

    pub fn update_repetition_table(&mut self) {
        self.repetition_table.clear();
        for (mini_board, _) in &self.stack {
            self.repetition_table.insert(mini_board.get_hash())
        }
        self.repetition_table.insert(self.get_hash())
    }

    #[inline]
    pub fn get_num_repetitions(&self) -> u8 {
        self.repetition_table.get_repetition(self.get_hash())
    }

    #[inline]
    pub fn is_repetition(&self, n_times: usize) -> bool {
        self.get_num_repetitions() as usize >= n_times
    }

    // pub fn gives_claimable_threefold_repetition(&mut self, valid_or_null_move: ValidOrNullMove) -> bool {
    //     self.push(Some(valid_or_null_move));
    //     if self.is_threefold_repetition() {
    //         self.pop();
    //         return true;
    //     }
    //     if self
    //         .generate_legal_moves()
    //         .any(|m| self.gives_threefold_repetition(m))
    //     {
    //         self.pop();
    //         return true;
    //     }
    //     self.pop();
    //     false
    // }

    #[inline]
    pub fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    #[inline]
    pub(crate) fn is_other_draw(&self) -> bool {
        self.is_fifty_moves() || self.is_threefold_repetition() || self.is_insufficient_material()
    }

    #[inline]
    pub fn is_draw(&self) -> bool {
        self.is_other_draw() || self.is_stalemate()
    }

    #[inline]
    pub fn is_game_over(&self) -> bool {
        self.is_other_draw() || self.status() != BoardStatus::Ongoing
    }

    pub fn pop(&mut self) -> ValidOrNullMove {
        let (mini_board, valid_or_null_move) = self.stack.pop().unwrap();
        self.repetition_table.remove(self.get_hash());
        self.mini_board = mini_board;
        valid_or_null_move
    }

    #[inline]
    pub fn get_all_stack_moves(&self) -> Vec<ValidOrNullMove> {
        self.stack.iter().map(|(_, m)| *m).collect_vec()
    }

    #[inline]
    pub fn get_last_stack_move(&self) -> Option<ValidOrNullMove> {
        self.stack.last().map(|(_, m)| *m)
    }

    #[inline]
    pub fn stack_contains_null_move(&self) -> bool {
        self.stack.iter().any(|(_, m)| m.is_null())
    }

    #[inline]
    pub fn get_ply(&self) -> usize {
        self.stack.len()
    }

    #[inline]
    pub fn has_empty_stack(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push_san(&mut self, san: &str) -> Result<ValidOrNullMove> {
        let valid_or_null_move = self.parse_san(san)?;
        self.push_unchecked(valid_or_null_move);
        Ok(valid_or_null_move)
    }

    #[inline]
    pub fn push_sans(&mut self, sans: &str) -> Result<Vec<ValidOrNullMove>> {
        remove_double_spaces_and_trim(sans)
            .split(' ')
            .map(|san| self.push_san(san))
            .collect()
    }

    pub fn push_uci(&mut self, uci: &str) -> Result<ValidOrNullMove> {
        let valid_or_null_move = self.parse_uci(uci)?;
        self.push(valid_or_null_move)?;
        Ok(valid_or_null_move)
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.push_uci(s).unwrap();
    }

    #[inline]
    pub fn push_uci_moves(&mut self, uci_moves: &str) -> Result<Vec<ValidOrNullMove>> {
        remove_double_spaces_and_trim(uci_moves)
            .split(' ')
            .map(|san| self.push_uci(san))
            .collect()
    }

    pub fn algebraic_and_push(
        &mut self,
        valid_or_null_move: ValidOrNullMove,
        long: bool,
    ) -> Result<String> {
        if valid_or_null_move.is_null() {
            self.push(valid_or_null_move)?;
            return Ok("--".to_string());
        }
        let san = valid_or_null_move.algebraic_without_suffix(self.get_mini_board(), long)?;

        // Look ahead for check or checkmate.
        self.push(valid_or_null_move)?;
        let is_checkmate = self.is_checkmate();

        // Add check or checkmate suffix.
        if is_checkmate {
            Ok(san + "#")
        } else if self.is_check() {
            Ok(san + "+")
        } else {
            Ok(san)
        }
    }

    #[inline]
    pub fn san_and_push(&mut self, valid_or_null_move: ValidOrNullMove) -> Result<String> {
        self.algebraic_and_push(valid_or_null_move, false)
    }

    #[inline]
    pub fn lan_and_push(&mut self, valid_or_null_move: ValidOrNullMove) -> Result<String> {
        self.algebraic_and_push(valid_or_null_move, true)
    }

    pub fn variation_san(board: &Board, variation: Vec<ValidOrNullMove>) -> String {
        let mut board = board.clone();
        let mut san = Vec::new();
        for valid_or_null_move in variation {
            if board.turn() == White {
                let san_str = board.san_and_push(valid_or_null_move);
                san.push(format!(
                    "{}. {}",
                    board.get_fullmove_number(),
                    san_str.unwrap()
                ));
            } else if san.is_empty() {
                let san_str = board.san_and_push(valid_or_null_move);
                san.push(format!(
                    "{}...{}",
                    board.get_fullmove_number(),
                    san_str.unwrap()
                ));
            } else {
                san.push(board.san_and_push(valid_or_null_move).unwrap().to_string());
            }
        }
        let mut san_string = String::new();
        for s in san {
            san_string += &(s + " ");
        }
        san_string.trim().to_string()
    }

    pub fn get_starting_board_fen(&self) -> String {
        if let Some((mini_board, _)) = self.stack.get(0) {
            mini_board.get_fen()
        } else {
            self.get_fen()
        }
    }

    pub fn get_pgn(&self) -> String {
        let mut pgn = String::new();
        let starting_fen = &self.get_starting_board_fen();
        if starting_fen != STARTING_POSITION_FEN {
            pgn += &format!("[FEN \"{}\"]\n", starting_fen);
        }
        pgn += &Self::variation_san(
            &Self::from_fen(starting_fen).unwrap(),
            self.stack
                .clone()
                .into_iter()
                .map(|(_, optional_m)| optional_m)
                .collect_vec(),
        );
        pgn
    }

    fn perft_helper(&mut self, depth: Depth, print_move: bool) -> usize {
        let moves = self.generate_legal_moves();
        if depth == 1 {
            return moves.len();
        }
        let mut count: usize = 0;
        for move_ in moves {
            self.push_unchecked(move_);
            let c_count = self.perft_helper(depth - 1, false);
            self.pop();
            if print_move {
                println_wasm!(
                    "{}: {}",
                    move_.colorize(PERFT_MOVE_STYLE),
                    c_count.colorize(PERFT_COUNT_STYLE),
                );
            }
            count += c_count;
        }
        count
    }

    #[inline]
    pub fn perft(&mut self, depth: Depth) -> usize {
        self.perft_helper(depth, true)
    }

    #[cfg(feature = "extras")]
    #[inline]
    pub fn evaluate(&mut self) -> Score {
        self.evaluator.evaluate(&self.mini_board)
    }

    #[cfg(feature = "extras")]
    #[inline]
    pub fn evaluate_flipped(&mut self) -> Score {
        let score = self.evaluate();
        self.score_flipped(score)
    }
}

impl BoardMethodOverload<Move> for Board {
    fn push_unchecked(&mut self, move_: Move) {
        let mini_board_copy = self.mini_board.clone();
        self.mini_board.make_move(move_);
        self.repetition_table.insert(self.get_hash());
        self.stack.push((mini_board_copy, move_.into()));
    }

    fn push(&mut self, move_: Move) -> Result<()> {
        if !self.is_legal(move_) {
            return Err(TimecatError::IllegalMove {
                valid_or_null_move: move_.into(),
                board_fen: self.get_fen(),
            });
        }
        self.push_unchecked(move_);
        Ok(())
    }

    #[inline]
    fn gives_repetition(&self, move_: Move) -> bool {
        self.repetition_table
            .get_repetition(self.mini_board.make_move_new(move_).get_hash())
            != 0
    }

    #[inline]
    fn gives_threefold_repetition(&self, move_: Move) -> bool {
        self.repetition_table
            .get_repetition(self.mini_board.make_move_new(move_).get_hash())
            == 2
    }

    fn gives_claimable_threefold_repetition(&self, move_: Move) -> bool {
        //TODO: check if this is correct
        let new_board = self.mini_board.make_move_new(move_);
        new_board.generate_legal_moves().any(|m| {
            let hash = new_board.make_move_new(m).get_hash();
            self.repetition_table.get_repetition(hash) == 2
        })
    }
}

impl BoardMethodOverload<ValidOrNullMove> for Board {
    fn push_unchecked(&mut self, valid_or_null_move: ValidOrNullMove) {
        let mini_board_copy = self.mini_board.clone();
        self.mini_board.make_move(valid_or_null_move);
        self.repetition_table.insert(self.get_hash());
        self.stack.push((mini_board_copy, valid_or_null_move));
    }

    fn push(&mut self, valid_or_null_move: ValidOrNullMove) -> Result<()> {
        if let Some(move_) = *valid_or_null_move {
            self.push(move_)
        } else {
            if self.is_check() {
                return Err(TimecatError::NullMoveInCheck {
                    fen: self.get_fen(),
                });
            }
            self.push_unchecked(valid_or_null_move);
            Ok(())
        }
    }

    #[inline]
    fn gives_repetition(&self, valid_or_null_move: ValidOrNullMove) -> bool {
        self.repetition_table
            .get_repetition(self.mini_board.make_move_new(valid_or_null_move).get_hash())
            != 0
    }

    #[inline]
    fn gives_threefold_repetition(&self, valid_or_null_move: ValidOrNullMove) -> bool {
        self.repetition_table
            .get_repetition(self.mini_board.make_move_new(valid_or_null_move).get_hash())
            == 2
    }

    fn gives_claimable_threefold_repetition(&self, valid_or_null_move: ValidOrNullMove) -> bool {
        //TODO: check if this is correct
        let new_board = self.mini_board.make_move_new(valid_or_null_move);
        new_board.generate_legal_moves().any(|m| {
            let hash = new_board.make_move_new(m).get_hash();
            self.repetition_table.get_repetition(hash) == 2
        })
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_board_string(false))
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::from_fen(STARTING_POSITION_FEN).unwrap()
    }
}

impl FromStr for Board {
    type Err = TimecatError;

    fn from_str(fen: &str) -> Result<Self> {
        Self::from_fen(fen)
    }
}

impl From<MiniBoard> for Board {
    fn from(mini_board: MiniBoard) -> Self {
        let mut board = Self {
            #[cfg(feature = "extras")]
            evaluator: Evaluator::new(&mini_board),
            mini_board,
            stack: Vec::new(),
            repetition_table: RepetitionTable::new(),
        };
        board.repetition_table.insert(board.get_hash());
        board
    }
}

impl From<&MiniBoard> for Board {
    fn from(mini_board: &MiniBoard) -> Self {
        mini_board.to_owned().into()
    }
}

impl Deref for Board {
    type Target = MiniBoard;

    fn deref(&self) -> &Self::Target {
        &self.mini_board
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for Board {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(mini_board) = ob.extract::<MiniBoard>() {
            let mut board = Board::from(mini_board);
            if let (Ok(moves_py_object), Ok(states_py_object)) =
                (ob.getattr("move_stack"), ob.getattr("_stack"))
            {
                let states = states_py_object
                    .extract::<Vec<MiniBoard>>()
                    .unwrap_or_default();
                let moves = moves_py_object
                    .extract::<Vec<ValidOrNullMove>>()
                    .unwrap_or_default();
                board.stack = states.into_iter().zip(moves).collect_vec();
                if !board.stack.windows(2).all(|window| {
                    let (prev_board, move_) = get_item_unchecked!(window, 0);
                    let (new_board, _) = get_item_unchecked!(window, 1);
                    prev_board.make_move_new(*move_) == *new_board
                }) {
                    board.stack.clear()
                }
                board.update_repetition_table();
            }
            return Ok(board);
        }
        Err(Pyo3Error::Pyo3ConvertError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
