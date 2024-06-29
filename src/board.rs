#![doc = include_str!("../documentation/board/README.md")]

use super::*;

/// Ths code defines an enum `GameResult` that represents the result of a game. It has three
/// variants: `Win(Color)`, `Draw`, and `InProgress`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum GameResult {
    Win(Color),
    Draw,
    InProgress,
}

impl GameResult {
    /// Check if the game result is a win
    pub fn is_win(&self) -> bool {
        matches!(self, GameResult::Win(_))
    }

    /// Check if the game result is a draw
    pub fn is_draw(&self) -> bool {
        matches!(self, GameResult::Draw)
    }

    /// Checks if the game result is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self, GameResult::InProgress)
    }

    /// Returns the color of the winner, if there is one
    pub fn winner(&self) -> Option<Color> {
        match self {
            GameResult::Win(color) => Some(*color),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    sub_board: SubBoard,
    stack: Vec<(SubBoard, Option<Move>)>,
    starting_fen: String,
    repetition_table: RepetitionTable,
    #[cfg(feature = "inbuilt_nnue")]
    evaluator: Evaluator,
}

impl Board {
    /// The function `new` creates a new instance of `Board` from a starting position FEN string.
    ///
    /// Returns:
    ///
    /// A new instance of `Board` is being returned, initialized with the starting position FEN
    /// (Forsyth-Edwards Notation) string.
    pub fn new() -> Self {
        SubBoard::from_str(STARTING_POSITION_FEN).unwrap().into()
    }

    /// The function `set_fen` sets the board state based on a given FEN string.
    ///
    /// Arguments:
    ///
    /// * `fen`: FEN stands for Forsyth-Edwards Notation, which is a standard notation for describing a
    /// particular board position of a chess game. It includes information about the placement of pieces
    /// on the board, the player to move, castling rights, en passant square, halfmove clock, and full
    ///
    /// Returns:
    ///
    /// The `set_fen` function returns a `Result<()>`.
    pub fn set_fen(&mut self, fen: &str) -> Result<()> {
        let fen = simplify_fen(fen);
        if !Self::is_good_fen(&fen) {
            return Err(TimecatError::BadFen { fen });
        }
        if fen == self.get_fen() {
            self.starting_fen = self.get_fen();
            return Ok(());
        }
        self.sub_board = SubBoard::from_str(&fen)?;
        self.repetition_table.clear();
        self.repetition_table.insert(self.get_hash());
        self.starting_fen = self.get_fen();
        self.stack.clear();
        Ok(())
    }

    /// The function `from_fen` parses a FEN string to create a new board state.
    ///
    /// Arguments:
    ///
    /// * `fen`: Forsyth-Edwards Notation (FEN) is a standard notation for describing a particular board
    /// position of a chess game. It includes information about the placement of pieces on the board,
    /// the current player's turn, castling rights, en passant square, halfmove clock, and full
    ///
    /// Returns:
    ///
    /// The `from_fen` function is returning a `Result` containing either a `ChessBoard` instance if the
    /// FEN string was successfully parsed and set on the board, or an `TimecatError` if there was an
    /// error during the parsing or setting of the FEN string.
    pub fn from_fen(fen: &str) -> Result<Self> {
        let mut board = Self::new();
        board.set_fen(fen)?;
        Ok(board)
    }

    /// This function returns a reference to the SubBoard struct owned by the current struct.
    ///
    /// Returns:
    ///
    /// A reference to the `SubBoard` struct is being returned.
    pub fn get_sub_board(&self) -> &SubBoard {
        &self.sub_board
    }

    #[cfg(feature = "inbuilt_nnue")]
    pub fn get_evaluator(&self) -> &Evaluator {
        &self.evaluator
    }

    #[cfg(feature = "inbuilt_nnue")]
    pub fn get_evaluator_mut(&mut self) -> &mut Evaluator {
        &mut self.evaluator
    }

    /// The function `is_good_fen` checks if a given FEN string is valid based on certain
    /// criteria.
    ///
    /// Arguments:
    ///
    /// * `fen`: The function `is_good_fen` takes a FEN (Forsyth-Edwards Notation) string as input and
    /// checks if it is a valid FEN representation of a chess position. The function performs the
    /// following checks:
    ///
    /// Returns:
    ///
    /// The function `is_good_fen` returns a boolean value - `true` if the FEN string passed as input is
    /// considered good based on certain conditions, and `false` otherwise.
    pub fn is_good_fen(fen: &str) -> bool {
        let fen = simplify_fen(fen);
        if SubBoard::from_str(&fen).is_err() {
            return false;
        }
        let mut splitted_fen = fen.split(' ');
        if splitted_fen.nth(4).unwrap_or("0").parse().unwrap_or(-1) < 0
            || splitted_fen.next().unwrap_or("1").parse().unwrap_or(-1) < 0
            || splitted_fen.next().is_some()
        {
            return false;
        };
        true
    }

    /// The `reset` function sets the position to the starting position.
    pub fn reset(&mut self) {
        self.set_fen(STARTING_POSITION_FEN).unwrap();
    }

    /// The `clear` function sets the internal state to an empty state represented by a specific
    /// FEN string.
    pub fn clear(&mut self) {
        self.set_fen(EMPTY_FEN).unwrap();
    }

    /// The `flip_vertical` function flips the game board vertically and resets certain game
    /// state variables.
    pub fn flip_vertical(&mut self) {
        self.sub_board.flip_vertical();
        self.stack.clear();
        self.repetition_table.clear();
        self.starting_fen = self.get_fen();
    }

    /// The `flip_vertical_and_flip_turn` function flips the game board vertically and flips the turn and resets certain game
    /// state variables.
    pub fn flip_vertical_and_flip_turn(&mut self) {
        self.sub_board.flip_vertical_and_flip_turn();
        self.stack.clear();
        self.repetition_table.clear();
        self.starting_fen = self.get_fen();
    }

    /// The `flip_horizontal` function flips the game board horizontally and resets certain game
    /// state variables.
    pub fn flip_horizontal(&mut self) {
        self.sub_board.flip_horizontal();
        self.stack.clear();
        self.repetition_table.clear();
        self.starting_fen = self.get_fen();
    }

    #[inline]
    pub fn to_board_string(&self, use_unicode: bool) -> String {
        self.sub_board
            .to_board_string(self.stack.last().and_then(|(_, m)| *m), use_unicode)
    }

    #[inline]
    pub fn to_unicode_string(&self) -> String {
        self.sub_board
            .to_unicode_string(self.stack.last().and_then(|(_, m)| *m))
    }

    /// The function `result` determines the outcome of a game based on the current board status in a
    /// Rust program.
    ///
    /// Returns:
    ///
    /// The `result` function returns a `GameResult` enum based on the current state of the game. If the
    /// game is a draw due to some other reason, it returns `GameResult::Draw`. If the game is in a
    /// checkmate state, it returns `GameResult::Win` for the player who did not make the last move. If
    /// the game is in a stalemate state,
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

    /// The function `get_num_moves` returns the number of moves in a stack as a `NumMoves` type.
    ///
    /// Returns:
    ///
    /// The `get_num_moves` function is returning the number of elements in the `stack` as a `NumMoves`
    /// type.
    #[inline]
    pub fn get_num_moves(&self) -> NumMoves {
        self.stack.len() as NumMoves
    }

    /// The function `get_num_repetitions` returns the number of repetitions for a given hash value.
    ///
    /// Returns:
    ///
    /// The `get_num_repetitions` function is returning a `u8` value, which represents the number of
    /// repetitions for a given hash in the repetition table.
    #[inline]
    pub fn get_num_repetitions(&self) -> u8 {
        self.repetition_table.get_repetition(self.get_hash())
    }

    /// The function `is_repetition` checks if the number of repetitions is greater than or equal to a
    /// specified value.
    ///
    /// Arguments:
    ///
    /// * `n_times`: The `n_times` parameter in the `is_repetition` function represents the number of
    /// times a certain action or event should be repeated. The function checks if the number of
    /// repetitions recorded is greater than or equal to the specified `n_times` value.
    ///
    /// Returns:
    ///
    /// A boolean value is being returned.
    #[inline]
    pub fn is_repetition(&self, n_times: usize) -> bool {
        self.get_num_repetitions() as usize >= n_times
    }

    /// The function `gives_repetition` checks if a move results in a repetition based on the repetition
    /// table.
    ///
    /// Arguments:
    ///
    /// * `move_`: The `move_` parameter is of type `Move`. It is used as an argument for the
    /// `make_move_new` method of the `sub_board` field, and also as an argument for the
    /// `get_repetition` method of the `repetition_table` field.
    ///
    /// Returns:
    ///
    /// The function `gives_repetition` is returning a boolean value, which indicates whether the result
    /// of the expression
    /// `self.repetition_table.get_repetition(self.sub_board.make_move_new(move_).get_hash())` is not equal
    /// to 0.
    #[inline]
    pub fn gives_repetition(&self, move_: Move) -> bool {
        self.repetition_table
            .get_repetition(self.sub_board.make_move_new(move_).get_hash())
            != 0
    }

    /// The function checks if a move results in a threefold repetition in a Rust program.
    ///
    /// Arguments:
    ///
    /// * `move_`: The `move_` parameter in the `gives_threefold_repetition` function represents the
    /// move that is being made on the board. This move is used to calculate the hash of the resulting
    /// board state after the move is made.
    ///
    /// Returns:
    ///
    /// The function `gives_threefold_repetition` is returning a boolean value, indicating whether the
    /// given move results in a threefold repetition in the game. It checks if the move leads to a
    /// position that has been repeated twice before in the game.
    #[inline]
    pub fn gives_threefold_repetition(&self, move_: Move) -> bool {
        self.repetition_table
            .get_repetition(self.sub_board.make_move_new(move_).get_hash())
            == 2
    }

    /// The function `gives_claimable_threefold_repetition` checks if a move leads to a position with a
    /// claimable threefold repetition.
    ///
    /// Arguments:
    ///
    /// * `move_`: The `move_` parameter in the `gives_claimable_threefold_repetition` function
    /// represents the move that is being made on the board. This move is used to generate a new board
    /// state, and then the function checks if making any legal move from that new board state would
    /// result in a
    ///
    /// Returns:
    ///
    /// The function `gives_claimable_threefold_repetition` returns a boolean value indicating whether
    /// the given move results in a position where a threefold repetition can be claimed.
    pub fn gives_claimable_threefold_repetition(&self, move_: Move) -> bool {
        //TODO: check if this is correct
        let new_board = self.sub_board.make_move_new(move_);
        MoveGenerator::new_legal(&new_board).any(|m| {
            let hash = new_board.make_move_new(m).get_hash();
            self.repetition_table.get_repetition(hash) == 2
        })
    }

    // pub fn gives_claimable_threefold_repetition(&mut self, move_: Move) -> bool {
    //     self.push(Some(move_));
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

    /// The function `is_threefold_repetition` checks if a certain position has occurred three times in
    /// a game.
    ///
    /// Returns:
    ///
    /// The `is_threefold_repetition` function is being called, which in turn calls the `is_repetition`
    /// function with the argument 3. The `is_repetition` function checks if the current board position
    /// has been repeated a certain number of times (in this case, 3 times). The
    /// `is_threefold_repetition` function then returns the result of this check as a boolean value
    #[inline]
    pub fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    /// The function `is_other_draw` checks if the game is a draw based on fifty-move rule,
    /// threefold repetition, or insufficient material.
    ///
    /// Returns:
    ///
    /// The `is_other_draw` function is returning a boolean value. It returns `true` if any of the
    /// conditions `self.is_fifty_moves()`, `self.is_threefold_repetition()`, or
    /// `self.is_insufficient_material()` are true. Otherwise, it returns `false`.
    #[inline]
    pub fn is_other_draw(&self) -> bool {
        self.is_fifty_moves() || self.is_threefold_repetition() || self.is_insufficient_material()
    }

    /// The function `is_draw` checks if a game is a draw by calling two other functions.
    ///
    /// Returns:
    ///
    /// The `is_draw` function is returning a boolean value. It returns `true` if either
    /// `is_other_draw()` or `is_stalemate()` methods return `true`, otherwise it returns `false`.
    #[inline]
    pub fn is_draw(&self) -> bool {
        self.is_other_draw() || self.is_stalemate()
    }

    /// The function `is_game_over` checks if the game is over based on whether it is a draw or
    /// the board status is not ongoing.
    ///
    /// Returns:
    ///
    /// A boolean value is being returned. The method `is_game_over` returns `true` if either
    /// `is_other_draw()` method returns `true` or the `status()` method does not return
    /// `BoardStatus::Ongoing`. Otherwise, it returns `false`.
    #[inline]
    pub fn is_game_over(&self) -> bool {
        self.is_other_draw() || self.status() != BoardStatus::Ongoing
    }

    pub fn push_unchecked(&mut self, optional_move: impl Into<Option<Move>>) {
        let optional_move = optional_move.into();
        let sub_board_copy = self.sub_board.clone();
        self.sub_board = if let Some(move_) = optional_move {
            self.sub_board.make_move_new(move_)
        } else {
            self.sub_board
                .null_move()
                .expect("Trying to push null move while in check!")
        };
        self.repetition_table.insert(self.get_hash());
        self.stack.push((sub_board_copy, optional_move));
    }

    pub fn push(&mut self, optional_move: impl Into<Option<Move>>) -> Result<()> {
        let optional_move = optional_move.into();
        if let Some(move_) = optional_move {
            if !self.is_legal(move_) {
                return Err(TimecatError::IllegalMove { move_, board_fen: self.get_fen() });
            }
        } else if self.is_check() {
            return Err(TimecatError::NullMoveInCheck { fen: self.get_fen() });
        }
        self.push_unchecked(optional_move);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Move> {
        let (sub_board, optional_move) = self.stack.pop().unwrap();
        self.repetition_table.remove(self.get_hash());
        self.sub_board = sub_board;
        optional_move
    }

    /// The function `get_all_moves` returns a vector of references to all moves stored in a stack.
    ///
    /// Returns:
    ///
    /// A vector of references to `Option<Move>` values is being returned.
    #[inline]
    pub fn get_all_moves(&self) -> Vec<Option<Move>> {
        self.stack.iter().map(|(_, m)| *m).collect_vec()
    }

    /// The function `get_last_move` returns the last move made, if any, from a stack of moves.
    ///
    /// Returns:
    ///
    /// The `get_last_move` function returns an `Option` that contains either `Some(Move)` if there is a
    /// last move available in the stack, or `None` if the stack is empty.
    #[inline]
    pub fn get_last_move(&self) -> Option<Option<Move>> {
        self.stack.last().map(|(_, m)| *m)
    }

    /// The function checks if any element in the stack contains a null move.
    ///
    /// Returns:
    ///
    /// The `contains_null_move` function is returning a boolean value (`true` or `false`). It checks if
    /// there is any `None` value present in the `m` field of the tuples in the `stack` vector. If any
    /// `None` value is found, it returns `true`, indicating that a null move is present in the stack;
    /// otherwise, it returns `false`.
    #[inline]
    pub fn contains_null_move(&self) -> bool {
        self.stack.iter().any(|(_, m)| m.is_none())
    }

    /// The function `get_ply` returns the length of the stack.
    ///
    /// Returns:
    ///
    /// The `get_ply` function is returning the length of the `stack` vector, which represents the
    /// number of elements currently stored in the stack.
    #[inline]
    pub fn get_ply(&self) -> usize {
        self.stack.len()
    }

    /// The function `has_empty_stack` checks if the stack is empty.
    ///
    /// Returns:
    ///
    /// A boolean value indicating whether the stack is empty or not.
    #[inline]
    pub fn has_empty_stack(&self) -> bool {
        self.stack.is_empty()
    }

    /// The function `push_san` takes a SAN (Standard Algebraic Notation) string, parses it into a move,
    /// pushes the move onto the board, and returns the move.
    ///
    /// Arguments:
    ///
    /// * `san`: The `san` parameter in the `push_san` function is a reference to a string that
    /// represents a move in Standard Algebraic Notation (SAN).
    ///
    /// Returns:
    ///
    /// The `push_san` function returns a `Result` containing an `Option` of `Move` or an `TimecatError`.
    pub fn push_san(&mut self, san: &str) -> Result<Option<Move>> {
        let move_ = self.parse_san(san)?;
        self.push_unchecked(move_);
        Ok(move_)
    }

    /// The `push_sans` function removes double spaces and trims a string, splits it into
    /// individual words, and then pushes each word as a move into a vector.
    ///
    /// Arguments:
    ///
    /// * `sans`: The `sans` parameter in the `push_sans` function is a reference to a string (`&str`)
    /// that represents a sequence of chess moves in Standard Algebraic Notation (SAN). The function
    /// processes this input by removing double spaces and trimming the string, then splitting it into
    /// individual move tokens
    ///
    /// Returns:
    ///
    /// The `push_sans` function is returning a `Result` containing a `Vec` of `Option<Move>` or an
    /// `TimecatError`.
    #[inline]
    pub fn push_sans(&mut self, sans: &str) -> Result<Vec<Option<Move>>> {
        remove_double_spaces_and_trim(sans)
            .split(' ')
            .map(|san| self.push_san(san))
            .collect()
    }

    /// The function `push_uci` takes a UCI string, parses it into a move, pushes the move onto a stack,
    /// and returns the move.
    ///
    /// Arguments:
    ///
    /// * `uci`: The `uci` parameter in the `push_uci` function is a reference to a string that
    /// represents a move in UCI (Universal Chess Interface) notation.
    ///
    /// Returns:
    ///
    /// The function `push_uci` returns a `Result` containing an `Option` of `Move` or an `TimecatError`.
    pub fn push_uci(&mut self, uci: &str) -> Result<Option<Move>> {
        let move_ = self.parse_uci(uci)?;
        self.push_unchecked(move_);
        Ok(move_)
    }

    /// The `push_str` function pushes a string to a data structure using the UCI protocol.
    ///
    /// Arguments:
    ///
    /// * `s`: The parameter `s` in the `push_str` function is a reference to a string slice (`&str`).
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.push_uci(s).unwrap();
    }

    /// The function `push_uci_moves` takes a string of UCI moves, processes them, and pushes them onto
    /// a vector of optional Moves.
    ///
    /// Arguments:
    ///
    /// * `uci_moves`: The `uci_moves` parameter is a string containing a series of UCI (Universal Chess
    /// Interface) formatted moves separated by spaces.
    ///
    /// Returns:
    ///
    /// The `push_uci_moves` function returns a `Result` containing a `Vec` of `Option<Move>` or an
    /// `TimecatError`.
    #[inline]
    pub fn push_uci_moves(&mut self, uci_moves: &str) -> Result<Vec<Option<Move>>> {
        remove_double_spaces_and_trim(uci_moves)
            .split(' ')
            .map(|san| self.push_uci(san))
            .collect()
    }

    /// The function `algebraic_and_push` takes an optional move, determines if it is a check or
    /// checkmate, and returns the algebraic notation of the move with appropriate suffixes.
    ///
    /// Arguments:
    ///
    /// * `optional_move`: The `optional_move` parameter is of type `impl Into<Option<Move>>`, which
    /// means it can accept any type that can be converted into an `Option<Move>`. This parameter is
    /// used to provide an optional move that the function will process.
    /// * `long`: The `long` parameter in the `algebraic_and_push` function is a boolean flag that
    /// indicates whether the algebraic notation should include long notation or not. When `long` is
    /// true, the algebraic notation will include additional information, typically the starting and
    /// ending squares of the move. When
    ///
    /// Returns:
    ///
    /// The function `algebraic_and_push` returns a `Result<String>`. The result can either
    /// be an `Ok` containing a `String` value representing the algebraic notation of a move with
    /// optional suffixes like "#" for checkmate or "+" for check, or an `Err` containing a `BoardError`
    /// if an error occurs during the execution of the function.
    pub fn algebraic_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
        long: bool,
    ) -> Result<String> {
        let optional_move = optional_move.into();
        if optional_move.is_none() {
            return Ok("--".to_string());
        }
        let move_ = optional_move.unwrap();
        let san = move_.algebraic_without_suffix(self.get_sub_board(), long)?;

        // Look ahead for check or checkmate.
        self.push_unchecked(move_);
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

    /// The function `san_and_push` takes an optional move and converts it into algebraic notation
    /// before pushing it onto the board.
    ///
    /// Arguments:
    ///
    /// * `optional_move`: The `optional_move` parameter is a move that is optional and can be either
    /// `Some(Move)` or `None`. It is passed as an argument to the `san_and_push` method.
    ///
    /// Returns:
    ///
    /// The `san_and_push` function is returning a `Result<String>`.
    #[inline]
    pub fn san_and_push(&mut self, optional_move: impl Into<Option<Move>>) -> Result<String> {
        self.algebraic_and_push(optional_move.into(), false)
    }

    /// The function `lan_and_push` takes an optional move and converts it into algebraic notation
    /// before pushing it onto the board.
    ///
    /// Arguments:
    ///
    /// * `optional_move`: The `optional_move` parameter is of type `impl Into<Option<Move>>`, which
    /// means it can accept any type that can be converted into an `Option<Move>`. This parameter is
    /// used as an input to the `lan_and_push` method.
    ///
    /// Returns:
    ///
    /// The `lan_and_push` function returns a `Result<String>`.
    #[inline]
    pub fn lan_and_push(&mut self, optional_move: impl Into<Option<Move>>) -> Result<String> {
        self.algebraic_and_push(optional_move.into(), true)
    }

    /// The `variation_san` function processes a sequence of moves in a chess game, converting
    /// them to Standard Algebraic Notation (SAN) format.
    ///
    /// Arguments:
    ///
    /// * `board`: The `board` parameter in the `variation_san` function represents the current state of
    /// the chess board. It is of type `Board`, which likely contains information about the positions of
    /// the pieces on the board, the current player's turn, and other relevant data for playing a game
    /// of chess. The
    /// * `variation`: The `variation_san` function you provided takes in a reference to a `Board` and a
    /// vector of optional `Move`s called `variation`. The function iterates over the optional moves in
    /// the variation, checks if each move is legal on the board, and constructs the Standard Algebraic
    /// Notation (
    ///
    /// Returns:
    ///
    /// The function `variation_san` returns a `String` containing the Standard Algebraic Notation (SAN)
    /// representation of the moves in the provided variation on the chess board.
    pub fn variation_san(board: &Board, variation: Vec<Option<Move>>) -> String {
        let mut board = board.clone();
        let mut san = Vec::new();
        for optional_move in variation {
            if let Some(move_) = optional_move {
                if !board.is_legal(move_) {
                    panic!("illegal move {move_} in position {}", board.get_fen());
                }
            }

            if board.turn() == White {
                let san_str = board.san_and_push(optional_move);
                san.push(format!(
                    "{}. {}",
                    board.get_fullmove_number(),
                    san_str.unwrap()
                ));
            } else if san.is_empty() {
                let san_str = board.san_and_push(optional_move);
                san.push(format!(
                    "{}...{}",
                    board.get_fullmove_number(),
                    san_str.unwrap()
                ));
            } else {
                san.push(board.san_and_push(optional_move).unwrap().to_string());
            }
        }
        let mut san_string = String::new();
        for s in san {
            san_string += &(s + " ");
        }
        san_string.trim().to_string()
    }

    /// The function `get_pgn` constructs a PGN (Portable Game Notation) string representation
    /// of a chess game, including FEN (Forsyth-Edwards Notation) and move information.
    ///
    /// Returns:
    ///
    /// The `get_pgn` function returns a `String` containing the PGN (Portable Game Notation)
    /// representation of a chess game. The PGN includes information such as the FEN (Forsyth-Edwards
    /// Notation) of the starting position and the sequence of moves in standard algebraic notation.
    pub fn get_pgn(&self) -> String {
        let mut pgn = String::new();
        if self.starting_fen != STARTING_POSITION_FEN {
            pgn += &format!("[FEN \"{}\"]\n", self.starting_fen);
        }
        pgn += &Self::variation_san(
            &Self::from_fen(&self.starting_fen).unwrap(),
            self.stack
                .clone()
                .into_iter()
                .map(|(_, optional_m)| optional_m)
                .collect_vec(),
        );
        pgn
    }

    /// The function `perft_helper` recursively calculates the number of possible moves at a given depth
    /// in a chess game, optionally printing the moves.
    ///
    /// Arguments:
    ///
    /// * `depth`: The `depth` parameter in the `perft_helper` function represents the depth to which the
    /// function should calculate the number of possible moves. It determines how many moves ahead the
    /// function should look to calculate the perft value.
    /// * `print_move`: The `print_move` parameter in the `perft_helper` function is a boolean flag that
    /// determines whether the function should print out the moves and their corresponding counts during
    /// the perft calculation. If `print_move` is set to `true`, the function will display the move and
    /// its count for each
    ///
    /// Returns:
    ///
    /// The `perft_helper` function is returning the total number of positions reached after exploring the
    /// specified depth of the game tree.
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
                println!(
                    "{}: {}",
                    move_.colorize(PERFT_MOVE_STYLE),
                    c_count.colorize(PERFT_COUNT_STYLE),
                );
            }
            count += c_count;
        }
        count
    }

    /// The function `perft` calculates the number of possible moves at a given depth in a game.
    ///
    /// Arguments:
    ///
    /// * `depth`: The `depth` parameter represents the depth of the search tree to which the Perft
    /// algorithm will be applied. It determines how many moves ahead the algorithm will explore to
    /// calculate the number of possible positions.
    ///
    /// Returns:
    ///
    /// The `perft` function is returning the result of calling the `perft_helper` function with the
    /// specified depth and a boolean value of `true`.
    #[inline]
    pub fn perft(&mut self, depth: Depth) -> usize {
        self.perft_helper(depth, true)
    }

    #[cfg(feature = "inbuilt_nnue")]
    #[inline]
    pub fn evaluate(&mut self) -> Score {
        self.evaluator.evaluate(&self.sub_board)
    }

    #[cfg(feature = "inbuilt_nnue")]
    #[inline]
    pub fn evaluate_flipped(&mut self) -> Score {
        let score = self.evaluate();
        self.score_flipped(score)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_board_string(false))
    }
}

impl Default for Board {
    fn default() -> Self {
        STARTING_POSITION_FEN.into()
    }
}

impl FromStr for Board {
    type Err = TimecatError;

    fn from_str(fen: &str) -> Result<Self> {
        Self::from_fen(fen)
    }
}

impl From<&str> for Board {
    fn from(fen: &str) -> Self {
        Self::from_fen(fen).unwrap()
    }
}

impl From<SubBoard> for Board {
    fn from(sub_board: SubBoard) -> Self {
        let mut board = Self {
            #[cfg(feature = "inbuilt_nnue")]
            evaluator: Evaluator::new(&sub_board),
            starting_fen: sub_board.get_fen(),
            sub_board,
            stack: Vec::new(),
            repetition_table: RepetitionTable::new(),
        };
        board.repetition_table.insert(board.get_hash());
        board
    }
}

impl From<&SubBoard> for Board {
    fn from(sub_board: &SubBoard) -> Self {
        sub_board.to_owned().into()
    }
}

impl Deref for Board {
    type Target = SubBoard;

    fn deref(&self) -> &Self::Target {
        &self.sub_board
    }
}
