#![doc = include_str!("../documentation/board/README.md")]

use super::*;

/// This code defines an enum `BoardError` that represents different types of errors that can occur in a board-related context. The enum has two variants.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum BoardError {
    InvalidSanMove { move_: Move, fen: String },
    CustomError { err_msg: String },
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSanMove { move_, fen } => write!(
                f,
                "san() and lan() expect move to be legal or null, but got {} in {}",
                move_, fen
            ),
            Self::CustomError { err_msg } => write!(f, "{err_msg}"),
        }
    }
}

impl Error for BoardError {}

/// Ths code defines an enum `GameResult` that represents the result of a game. It has three
/// variants: `Win(Color)`, `Draw`, and `InProgress`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// The `Board` struct represents a chess board with various methods for game logic and move generation.
///
/// Properties:
///
/// * `sub_board`: The `sub_board` property represents the sub-board within the main board. It contains
/// the current state of the game board, including the positions of all pieces and other relevant game
/// information.
/// * `stack`: The `stack` property in the `Board` struct is a vector that stores the history of moves
/// made on the board. Each element in the vector represents a tuple containing the previous `SubBoard`
/// state and the optional move that was made from that state. This history allows for tracking and
/// undoing moves.
/// * `starting_fen`: The `starting_fen` property in the `Board` struct represents the FEN
/// (Forsyth-Edwards Notation) string of the starting position of the chess board. This property stores
/// the initial configuration of the chess pieces on the board.
/// * `repetition_table`: The `repetition_table` property in the `Board` struct is used to keep track of
/// the positions that have occurred in the game to detect threefold repetition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Board {
    sub_board: SubBoard,
    stack: Vec<(SubBoard, Option<Move>)>,
    starting_fen: String,
    repetition_table: RepetitionTable,
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
    /// The `set_fen` function returns a `Result<(), EngineError>`.
    pub fn set_fen(&mut self, fen: &str) -> Result<(), EngineError> {
        let fen = simplify_fen(fen);
        if !Self::is_good_fen(&fen) {
            return Err(EngineError::BadFen { fen });
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
    /// FEN string was successfully parsed and set on the board, or an `EngineError` if there was an
    /// error during the parsing or setting of the FEN string.
    pub fn from_fen(fen: &str) -> Result<Self, EngineError> {
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

    /// The function `empty` returns a chess board with no pieces on it.
    ///
    /// Returns:
    ///
    /// The `empty` function is returning a chess board that is initialized with an empty position. It
    /// is using the `from_fen` method to create the board from a FEN (Forsyth-Edwards Notation) string
    /// representation of an empty board.
    pub fn empty() -> Self {
        Self::from_fen(EMPTY_FEN).unwrap()
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

    /// The function `piece_symbol_at` returns the symbol of the piece at a given square or an empty
    /// space symbol if there is no piece.
    ///
    /// Arguments:
    ///
    /// * `square`: Square is a parameter representing a specific square on a chessboard. It could be a
    /// coordinate like "A1" or a numerical representation like 0-63, depending on how the chessboard is
    /// implemented in your code.
    ///
    /// Returns:
    ///
    /// The `piece_symbol_at` function returns a `String` representing the symbol of the piece at the
    /// given square. If there is a piece at the square, it returns the string representation of that
    /// piece. If there is no piece at the square, it returns the string representation of an empty
    /// space symbol.
    pub fn piece_symbol_at(&self, square: Square) -> String {
        match self.piece_at(square) {
            Some(piece) => piece.to_string(),
            None => EMPTY_SPACE_SYMBOL.to_string(),
        }
    }

    /// This function returns the Unicode symbol representing a chess piece at a given square, with an
    /// option to flip the color.
    ///
    /// Arguments:
    ///
    /// * `square`: The `square` parameter represents a specific square on a chessboard where a piece is
    /// located. It is typically represented by a combination of a file (a-h) and a rank (1-8), such as
    /// "e4" or "h7".
    /// * `flip_color`: The `flip_color` parameter is a boolean flag that determines whether to flip the
    /// colors of the pieces. If `flip_color` is `true`, the colors of the pieces will be flipped (e.g.,
    /// white pieces will be displayed as black and vice versa). If `flip_color` is `
    ///
    /// Returns:
    ///
    /// The function `piece_unicode_symbol_at` returns a `String` value representing the Unicode symbol
    /// for the piece at the specified square. If there is a piece at the square, it returns the Unicode
    /// symbol for that piece based on its color and type. If there is no piece at the square, it
    /// returns the Unicode symbol for an empty space.
    pub fn piece_unicode_symbol_at(&self, square: Square, flip_color: bool) -> String {
        if let Some(piece) = self.piece_at(square) {
            let piece_index = piece.get_piece_type().to_index();
            let (white_pieces, black_pieces) = match flip_color {
                true => (BLACK_PIECE_UNICODE_SYMBOLS, WHITE_PIECE_UNICODE_SYMBOLS),
                false => (WHITE_PIECE_UNICODE_SYMBOLS, BLACK_PIECE_UNICODE_SYMBOLS),
            };
            return match piece.get_color() {
                White => get_item_unchecked!(white_pieces, piece_index),
                Black => get_item_unchecked!(black_pieces, piece_index),
            }
            .to_string();
        }
        EMPTY_SPACE_UNICODE_SYMBOL.to_string()
    }

    /// The function `to_board_string` generates a string representation of a board with various
    /// styles and information displayed, such as pieces, last move highlights, FEN, transposition key,
    /// checkers, and evaluation.
    ///
    /// Arguments:
    ///
    /// * `use_unicode`: The `use_unicode` parameter in the `to_board_string` function is a boolean flag
    /// that determines whether to use Unicode symbols for pieces on the board. If `use_unicode` is
    /// `true`, the function will use Unicode symbols for pieces, otherwise it will use non-Unicode
    /// symbols.
    ///
    /// Returns:
    ///
    /// The function `to_board_string` is returning a formatted string representation of the board
    /// state, including the pieces, styles, last move highlight, and additional information such as
    /// FEN, Transposition Key, Checkers, and Current Evaluation (if the "nnue" feature is enabled).
    fn to_board_string(&self, use_unicode: bool) -> String {
        let mut skeleton = get_board_skeleton();
        let checkers = self.get_checkers();
        let king_square = self.get_king_square(self.sub_board.turn());
        let last_move = self.stack.last().and_then(|(_, m)| *m);
        for square in SQUARES_HORIZONTAL_MIRROR {
            let symbol = if use_unicode {
                self.piece_unicode_symbol_at(square, false)
            } else {
                self.piece_symbol_at(square)
            };
            let mut styles = vec![];
            if symbol != " " {
                styles.extend_from_slice(match self.color_at(square).unwrap() {
                    White => WHITE_PIECES_STYLE,
                    Black => BLACK_PIECES_STYLE,
                });
                if square == king_square && !checkers.is_empty() {
                    styles.extend_from_slice(CHECK_STYLE);
                }
            }
            if last_move.is_some()
                && [
                    last_move.unwrap().get_source(),
                    last_move.unwrap().get_dest(),
                ]
                .contains(&square)
            {
                styles.extend_from_slice(LAST_MOVE_HIGHLIGHT_STYLE);
            }
            styles.dedup();
            skeleton = skeleton.replacen('O', &symbol.colorize(&styles), 1);
        }
        skeleton.push('\n');
        skeleton.push_str(
            &[
                String::new(),
                format_info("Fen", self.get_fen(), true),
                format_info("Transposition Key", self.get_hash().stringify(), true),
                format_info(
                    "Checkers",
                    checkers.stringify().colorize(CHECKERS_STYLE),
                    true,
                ),
            ]
            .join("\n"),
        );
        #[cfg(feature = "nnue")]
        skeleton.push_str(&format!(
            "\n{}",
            format_info("Current Evaluation", self.evaluate().stringify(), true)
        ));
        skeleton
    }

    /// The `to_unicode_string` function returns a string representation of a board with Unicode
    /// characters.
    ///
    /// Returns:
    ///
    /// The `to_unicode_string` method is being called on `self`, which is likely a struct or object
    /// that has a method called `to_board_string`. The `to_board_string` method is being called with
    /// the argument `true`, and its return value is being returned by the `to_unicode_string` method.
    /// Therefore, the return value of the `to_unicode_string` method is a `String
    #[inline(always)]
    pub fn to_unicode_string(&self) -> String {
        self.to_board_string(true)
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
    #[inline(always)]
    pub fn get_num_moves(&self) -> NumMoves {
        self.stack.len() as NumMoves
    }

    /// The function `get_num_repetitions` returns the number of repetitions for a given hash value.
    ///
    /// Returns:
    ///
    /// The `get_num_repetitions` function is returning a `u8` value, which represents the number of
    /// repetitions for a given hash in the repetition table.
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    pub fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    /// The function `is_halfmoves` checks if the halfmove clock is greater than or equal to a given
    /// number `n`.
    ///
    /// Arguments:
    ///
    /// * `n`: The parameter `n` represents the number of halfmoves to check against the halfmove clock.
    /// The function `is_halfmoves` will return `true` if the halfmove clock value is greater than or
    /// equal to `n`, indicating that `n` or more halfmoves have been made since the
    ///
    /// Returns:
    ///
    /// A boolean value is being returned.
    #[inline(always)]
    fn is_halfmoves(&self, n: u8) -> bool {
        self.get_halfmove_clock() >= n
    }

    /// The function `is_fifty_moves` checks if the game has reached the fifty-move rule
    /// condition.
    ///
    /// Returns:
    ///
    /// The `is_fifty_moves` function is returning a boolean value indicating whether the number of
    /// halfmoves is equal to 100.
    #[inline(always)]
    pub fn is_fifty_moves(&self) -> bool {
        self.is_halfmoves(100)
    }

    /// The function `is_stalemate` checks if the current board status is a stalemate.
    ///
    /// Returns:
    ///
    /// A boolean value indicating whether the current board status is a stalemate.
    #[inline(always)]
    pub fn is_stalemate(&self) -> bool {
        self.status() == BoardStatus::Stalemate
    }

    /// The function `is_other_draw` checks if the game is a draw based on fifty-move rule,
    /// threefold repetition, or insufficient material.
    ///
    /// Returns:
    ///
    /// The `is_other_draw` function is returning a boolean value. It returns `true` if any of the
    /// conditions `self.is_fifty_moves()`, `self.is_threefold_repetition()`, or
    /// `self.is_insufficient_material()` are true. Otherwise, it returns `false`.
    #[inline(always)]
    pub fn is_other_draw(&self) -> bool {
        self.is_fifty_moves() || self.is_threefold_repetition() || self.is_insufficient_material()
    }

    /// The function `is_draw` checks if a game is a draw by calling two other functions.
    ///
    /// Returns:
    ///
    /// The `is_draw` function is returning a boolean value. It returns `true` if either
    /// `is_other_draw()` or `is_stalemate()` methods return `true`, otherwise it returns `false`.
    #[inline(always)]
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
    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.is_other_draw() || self.status() != BoardStatus::Ongoing
    }

    /// Check if the move is a double pawn push
    pub fn is_double_pawn_push(&self, move_: Move) -> bool {
        let source = move_.get_source();
        let dest = move_.get_dest();
        source.get_rank() == self.turn().to_second_rank()
            && source
                .get_rank()
                .to_int()
                .abs_diff(dest.get_rank().to_int())
                == 2
            && !self.get_piece_mask(Pawn).contains(source)
    }

    /// The function `is_quiet` determines if a move is not a capture and does not give check.
    ///
    /// Arguments:
    ///
    /// * `move_`: The `move_` parameter in the `is_quiet` function represents a move in a chess game.
    /// It is used to check whether the move is a quiet move, meaning it is not a capture move and does
    /// not give check to the opponent's king.
    ///
    /// Returns:
    ///
    /// The function `is_quiet` is returning a boolean value, which is determined by the logical NOT
    /// operation applied to the result of the expression `(self.is_capture(move_) ||
    /// self.gives_check(move_))`.
    #[inline(always)]
    pub fn is_quiet(&self, move_: Move) -> bool {
        !(self.is_capture(move_) || self.gives_check(move_))
    }

    /// The function `has_legal_en_passant` checks if there is a legal en passant move available in a
    /// game of chess.
    ///
    /// Returns:
    ///
    /// The `has_legal_en_passant` function is returning a boolean value. It returns `true` if the
    /// `ep_square` is not `None`, indicating that there is a legal en passant move available.
    /// Otherwise, it returns `false`.
    #[inline(always)]
    pub fn has_legal_en_passant(&self) -> bool {
        self.ep_square().is_some()
    }

    /// The function `clean_castling_rights` cleans up castling rights for both white and black
    /// players.
    ///
    /// Returns:
    ///
    /// The `clean_castling_rights` function returns a `BitBoard` representing the castling rights for
    /// both white and black players after cleaning up any invalid or unnecessary rights.
    pub fn clean_castling_rights(&self) -> BitBoard {
        let white_castling_rights = match self.sub_board.castle_rights(White) {
            CastleRights::Both => BB_A1 ^ BB_H1,
            CastleRights::KingSide => BB_H1,
            CastleRights::QueenSide => BB_A1,
            CastleRights::None => BB_EMPTY,
        };
        let black_castling_rights = match self.sub_board.castle_rights(Black) {
            CastleRights::Both => BB_A8 ^ BB_H8,
            CastleRights::KingSide => BB_H8,
            CastleRights::QueenSide => BB_A8,
            CastleRights::None => BB_EMPTY,
        };
        white_castling_rights ^ black_castling_rights
    }

    // fn reduces_castling_rights(&self, move_: Move) -> bool {
    //     let cr = self.clean_castling_rights();
    //     let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
    //     let touched_cr = touched & cr;
    //     let kings = self.get_piece_mask(King);
    //     let touched_kings_cr = touched_cr & kings;
    //     !touched_cr.is_empty()
    //         || !(BB_RANK_1 & touched_kings_cr & self.occupied_co(White)).is_empty()
    //         || !(BB_RANK_8 & touched_kings_cr & self.occupied_co(Black)).is_empty()
    // }

    /// The function reduces_castling_rights checks if certain conditions are met to reduce
    /// castling rights after a move.
    ///
    /// Arguments:
    ///
    /// * `move_`: The `move_` parameter in the `reduces_castling_rights` function represents the move
    /// that is being made on the chessboard. It contains information about the source square and the
    /// destination square of the move. The function is checking the logic to determine if the castling
    /// rights should be reduced
    ///
    /// Returns:
    ///
    /// The function `reduces_castling_rights` returns a boolean value.
    fn reduces_castling_rights(&self, move_: Move) -> bool {
        // TODO: Check Logic
        let cr = self.clean_castling_rights();
        let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
        let touched_cr = touched & cr;
        let touched_kings_cr_is_empty = (touched_cr & self.get_piece_mask(King)).is_empty();
        !(touched_cr.is_empty()
            && touched_kings_cr_is_empty
            && BB_RANK_1.is_empty()
            && self.occupied_co(White).is_empty()
            && BB_RANK_8.is_empty()
            && self.occupied_co(Black).is_empty())
    }

    /// The function `is_irreversible` checks if a move is irreversible based on certain
    /// conditions.
    ///
    /// Arguments:
    ///
    /// * `move_`: The `move_` parameter in the `is_irreversible` function represents a move in a chess
    /// game. It is used to check if the move is irreversible based on certain conditions like having a
    /// legal en passant move, being a zeroing move, or reducing castling rights.
    ///
    /// Returns:
    ///
    /// A boolean value is being returned.
    #[inline(always)]
    pub fn is_irreversible(&self, move_: Move) -> bool {
        self.has_legal_en_passant() || self.is_zeroing(move_) || self.reduces_castling_rights(move_)
    }

    /// The function `is_endgame` determines if the current game state is in the endgame phase
    /// based on the number and types of pieces remaining on the board.
    ///
    /// Returns:
    ///
    /// The `is_endgame` function returns a boolean value based on certain conditions. If the number of
    /// pieces on the board is less than or equal to a threshold defined by `ENDGAME_PIECE_THRESHOLD`,
    /// it returns `true`. Otherwise, it checks the number of Queens on the board and applies different
    /// conditions based on the count of Queens to determine if the game is in the endgame phase.
    #[inline(always)]
    pub fn is_endgame(&self) -> bool {
        if self.get_num_pieces() <= ENDGAME_PIECE_THRESHOLD {
            return true;
        }
        match self.get_piece_mask(Queen).popcnt() {
            0 => {
                (self.get_piece_mask(Rook)
                    ^ self.get_piece_mask(Bishop)
                    ^ self.get_piece_mask(Knight))
                .popcnt()
                    <= 4
            }
            1 => {
                self.get_piece_mask(Rook).popcnt() <= 2
                    && (self.get_piece_mask(Bishop) ^ self.get_piece_mask(Knight)).is_empty()
            }
            2 => (self.get_piece_mask(Rook)
                ^ self.get_piece_mask(Bishop)
                ^ self.get_piece_mask(Knight))
            .is_empty(),
            _ => false,
        }
    }

    /// The `push` function takes an optional move, updates the sub-board accordingly, and adds
    /// the previous sub-board state and move to a stack.
    ///
    /// Arguments:
    ///
    /// * `optional_move`: The `optional_move` parameter in the `push` function is of type `impl
    /// Into<Option<Move>>`. This means it can accept any type that can be converted into an
    /// `Option<Move>`. Inside the function, the `optional_move` is converted into an `Option<Move>`
    pub fn push(&mut self, optional_move: impl Into<Option<Move>>) {
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

    /// The `pop` function removes and returns the top element from a stack, updating internal
    /// state accordingly.
    ///
    /// Returns:
    ///
    /// The `pop` function returns an `Option<Move>`.
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
    #[inline(always)]
    pub fn get_all_moves(&self) -> Vec<&Option<Move>> {
        self.stack.iter().map(|(_, m)| m).collect_vec()
    }

    /// The function `get_last_move` returns the last move made, if any, from a stack of moves.
    ///
    /// Returns:
    ///
    /// The `get_last_move` function returns an `Option` that contains either `Some(Move)` if there is a
    /// last move available in the stack, or `None` if the stack is empty.
    #[inline(always)]
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
    #[inline(always)]
    pub fn contains_null_move(&self) -> bool {
        self.stack.iter().any(|(_, m)| m.is_none())
    }

    /// The function `get_ply` returns the length of the stack.
    ///
    /// Returns:
    ///
    /// The `get_ply` function is returning the length of the `stack` vector, which represents the
    /// number of elements currently stored in the stack.
    #[inline(always)]
    pub fn get_ply(&self) -> usize {
        self.stack.len()
    }

    /// The function `has_empty_stack` checks if the stack is empty.
    ///
    /// Returns:
    ///
    /// A boolean value indicating whether the stack is empty or not.
    #[inline(always)]
    pub fn has_empty_stack(&self) -> bool {
        self.stack.is_empty()
    }

    /// The function `parse_san` parses a given SAN (Standard Algebraic Notation) string to
    /// return a corresponding chess move or an error if the move is invalid.
    ///
    /// Arguments:
    ///
    /// * `san`: The `san` parameter in the `parse_san` function represents the Standard Algebraic
    /// Notation (SAN) string of a chess move that needs to be parsed and converted into a `Move`
    /// object. The function first trims any leading or trailing whitespace from the input `san` string
    /// and then
    ///
    /// Returns:
    ///
    /// The `parse_san` function returns a `Result` containing either an `Option<Move>` or an
    /// `EngineError`. If the parsing is successful, it returns `Ok(Some(move))` with the parsed move.
    /// If the parsing fails due to an invalid SAN move string, it returns an `EngineError` with the
    /// specific error message.
    pub fn parse_san(&self, mut san: &str) -> Result<Option<Move>, EngineError> {
        // TODO: Make the logic better
        san = san.trim();
        if san == "--" {
            return Ok(None);
        }
        let san = san.replace('0', "O");
        for move_ in self.generate_legal_moves() {
            if move_.san(self.get_sub_board()).unwrap() == san {
                return Ok(Some(move_));
            }
        }
        Err(EngineError::InvalidSanMoveString { s: san.to_string() })
        // Move::from_san(&self.sub_board, &san.replace('0', "O"))
    }

    /// The function `parse_uci` parses a UCI string into a Move object or returns None if the
    /// input is "0000".
    ///
    /// Arguments:
    ///
    /// * `uci`: The `uci` parameter is a string that represents a move in UCI (Universal Chess
    /// Interface) format. It is used to specify chess moves in a standardized way for communication
    /// between chess engines and graphical user interfaces.
    ///
    /// Returns:
    ///
    /// The `parse_uci` function returns a `Result` containing either `None` if the input `uci` is
    /// "0000", or `Some(Move)` if the input `uci` can be successfully parsed into a `Move` object.
    #[inline(always)]
    pub fn parse_uci(&self, uci: &str) -> Result<Option<Move>, EngineError> {
        if uci == "0000" {
            return Ok(None);
        }
        Ok(Some(Move::from_str(uci)?))
    }

    /// The function `parse_move` parses a move text input and returns a `Move` or an
    /// `EngineError`.
    ///
    /// Arguments:
    ///
    /// * `move_text`: The `move_text` parameter is a reference to a string slice (`&str`) that
    /// represents the text input containing a chess move in either UCI (Universal Chess Interface) or
    /// SAN (Standard Algebraic Notation) format.
    ///
    /// Returns:
    ///
    /// The `parse_move` function returns a `Result` containing either `Some(Move)` if the move text can
    /// be parsed successfully, or `None` if the move text cannot be parsed. If an error occurs during
    /// parsing, an `EngineError` is returned.
    #[inline(always)]
    pub fn parse_move(&self, move_text: &str) -> Result<Option<Move>, EngineError> {
        self.parse_uci(move_text).or(self.parse_san(move_text))
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
    /// The `push_san` function returns a `Result` containing an `Option` of `Move` or an `EngineError`.
    pub fn push_san(&mut self, san: &str) -> Result<Option<Move>, EngineError> {
        let move_ = self.parse_san(san)?;
        self.push(move_);
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
    /// `EngineError`.
    #[inline(always)]
    pub fn push_sans(&mut self, sans: &str) -> Result<Vec<Option<Move>>, EngineError> {
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
    /// The function `push_uci` returns a `Result` containing an `Option` of `Move` or an `EngineError`.
    pub fn push_uci(&mut self, uci: &str) -> Result<Option<Move>, EngineError> {
        let move_ = self.parse_uci(uci)?;
        self.push(move_);
        Ok(move_)
    }

    /// The `push_str` function pushes a string to a data structure using the UCI protocol.
    ///
    /// Arguments:
    ///
    /// * `s`: The parameter `s` in the `push_str` function is a reference to a string slice (`&str`).
    #[inline(always)]
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
    /// `EngineError`.
    #[inline(always)]
    pub fn push_uci_moves(&mut self, uci_moves: &str) -> Result<Vec<Option<Move>>, EngineError> {
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
    /// The function `algebraic_and_push` returns a `Result<String, BoardError>`. The result can either
    /// be an `Ok` containing a `String` value representing the algebraic notation of a move with
    /// optional suffixes like "#" for checkmate or "+" for check, or an `Err` containing a `BoardError`
    /// if an error occurs during the execution of the function.
    pub fn algebraic_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
        long: bool,
    ) -> Result<String, BoardError> {
        let optional_move = optional_move.into();
        if optional_move.is_none() {
            return Ok("--".to_string());
        }
        let move_ = optional_move.unwrap();
        let san = move_.algebraic_without_suffix(self.get_sub_board(), long)?;

        // Look ahead for check or checkmate.
        self.push(move_);
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
    /// The `san_and_push` function is returning a `Result<String, BoardError>`.
    #[inline(always)]
    pub fn san_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
    ) -> Result<String, BoardError> {
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
    /// The `lan_and_push` function returns a `Result<String, BoardError>`.
    #[inline(always)]
    pub fn lan_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
    ) -> Result<String, BoardError> {
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
    fn variation_san(&self, board: &Board, variation: Vec<Option<Move>>) -> String {
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
        pgn += &self.variation_san(
            &Self::from_fen(&self.starting_fen).unwrap(),
            Vec::from_iter(
                self.stack
                    .clone()
                    .into_iter()
                    .map(|(_, optional_m)| optional_m),
            ),
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
            self.push(move_);
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
    #[inline(always)]
    pub fn perft(&mut self, depth: Depth) -> usize {
        self.perft_helper(depth, true)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (Piece, Square)> + 'a {
        self.sub_board.iter()
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
    type Err = EngineError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
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
            sub_board,
            stack: Vec::new(),
            starting_fen: STARTING_POSITION_FEN.to_string(),
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

macro_rules! copy_from_sub_board {
    ($($visibility:vis fn $function:ident(&self $(, $argument:ident: $argument_type:ty)* $(,)?) -> $return_type:ty),* $(,)?) => {
        /// All the functions are copied from `SubBoard` struct.
        impl Board {
            $(
                #[inline(always)]
                $visibility fn $function(&self, $($argument: $argument_type),*) -> $return_type {
                    self.sub_board.$function($($argument),*)
                }
            )*
        }
    };
}

copy_from_sub_board!(
    pub fn generate_legal_moves(&self) -> MoveGenerator,
    pub fn generate_masked_legal_moves(&self, to_bitboard: BitBoard) -> MoveGenerator,
    pub fn generate_legal_captures(&self) -> MoveGenerator,
    pub fn get_hash(&self) -> u64,
    pub fn get_pawn_hash(&self) -> u64,
    pub fn get_material_score(&self) -> Score,
    pub fn get_non_pawn_material_score_abs(&self) -> Score,
    pub fn get_winning_side(&self) -> Option<Color>,
    pub fn get_material_score_flipped(&self) -> Score,
    pub fn get_material_score_abs(&self) -> Score,
    pub fn is_legal(&self, move_: Move) -> bool,
    pub fn score_flipped(&self, score: Score) -> Score,
    pub fn get_masked_material_score_abs(&self, mask: BitBoard) -> Score,
    pub fn get_fen(&self) -> String,
    pub fn color_at(&self, square: Square) -> Option<Color>,
    pub fn piece_type_at(&self, square: Square) -> Option<PieceType>,
    pub fn piece_at(&self, square: Square) -> Option<Piece>,
    pub fn get_checkers(&self) -> BitBoard,
    pub fn get_king_square(&self, color: Color) -> Square,
    pub fn turn(&self) -> Color,
    pub fn occupied(&self) -> BitBoard,
    pub fn occupied_co(&self, color: Color) -> BitBoard,
    pub fn get_black_occupied(&self) -> BitBoard,
    pub fn get_white_occupied(&self) -> BitBoard,
    pub fn is_check(&self) -> bool,
    pub fn is_checkmate(&self) -> bool,
    pub fn status(&self) -> BoardStatus,
    pub fn get_halfmove_clock(&self) -> u8,
    pub fn get_fullmove_number(&self) -> NumMoves,
    pub fn has_non_pawn_material(&self) -> bool,
    pub fn get_non_king_pieces_mask(&self) -> BitBoard,
    pub fn has_only_same_colored_bishop(&self) -> bool,
    pub fn is_insufficient_material(&self) -> bool,
    pub fn is_en_passant(&self, move_: Move) -> bool,
    pub fn is_passed_pawn(&self, square: Square) -> bool,
    pub fn is_capture(&self, move_: Move) -> bool,
    pub fn is_zeroing(&self, move_: Move) -> bool,
    pub fn get_piece_mask(&self, piece: PieceType) -> BitBoard,
    pub fn ep_square(&self) -> Option<Square>,
    pub fn is_castling(&self, move_: Move) -> bool,
    pub fn get_num_pieces(&self) -> u32,
    pub fn has_insufficient_material(&self, color: Color) -> bool,
    pub fn gives_check(&self, move_: Move) -> bool,
    pub fn gives_checkmate(&self, move_: Move) -> bool,
);

#[cfg(feature = "nnue")]
copy_from_sub_board!(
    pub fn evaluate(&self) -> Score,
    pub fn evaluate_flipped(&self) -> Score,
);
