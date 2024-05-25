use super::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Fail)]
pub enum BoardError {
    #[fail(
        display = "san() and lan() expect move to be legal or null, but got {} in {}",
        move_, fen
    )]
    InvalidSanMove { move_: Move, fen: String },

    #[fail(display = "{}", err_msg)]
    CustomError { err_msg: String },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct Board {
    sub_board: SubBoard,
    stack: Vec<(SubBoard, Option<Move>)>,
    starting_fen: String,
    repetition_table: RepetitionTable,
}

impl Board {
    pub fn new() -> Self {
        SubBoard::from_str(STARTING_POSITION_FEN).unwrap().into()
    }

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
        self.repetition_table.insert(self.hash());
        self.starting_fen = self.get_fen();
        self.stack.clear();
        Ok(())
    }

    pub fn from_fen(fen: &str) -> Result<Self, EngineError> {
        let mut board = Self::new();
        board.set_fen(fen)?;
        Ok(board)
    }

    pub fn get_sub_board(&self) -> &SubBoard {
        &self.sub_board
    }

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

    pub fn empty() -> Self {
        Self::from_fen(EMPTY_FEN).unwrap()
    }

    pub fn reset(&mut self) {
        self.set_fen(STARTING_POSITION_FEN).unwrap();
    }

    pub fn clear(&mut self) {
        self.set_fen(EMPTY_FEN).unwrap();
    }

    pub fn flip_vertical(&mut self) {
        self.sub_board.flip_vertical();
        self.stack.clear();
        self.repetition_table.clear();
        self.starting_fen = self.get_fen();
    }

    pub fn piece_symbol_at(&self, square: Square) -> String {
        match self.piece_at(square) {
            Some(piece) => piece.to_string(),
            None => EMPTY_SPACE_SYMBOL.to_string(),
        }
    }

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

    fn to_board_string(&self, use_unicode: bool) -> String {
        let mut skeleton = get_board_skeleton();
        let checkers = self.get_checkers();
        let king_square = self.get_king_square(self.sub_board.turn());
        let last_move = self.stack.last().and_then(|(_, m)| *m);
        for square in SQUARES_180 {
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
                format_info("Transposition Key", self.hash().stringify(), true),
                format_info(
                    "Checkers",
                    checkers.stringify().colorize(CHECKERS_STYLE),
                    true,
                ),
            ]
            .join("\n"),
        );
        #[cfg(feature = "nnue")]
        skeleton.push_str(&format!("\n{}", format_info("Current Evaluation", self.evaluate().stringify(), true)));
        skeleton
    }

    #[inline(always)]
    pub fn to_unicode_string(&self) -> String {
        self.to_board_string(true)
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

    #[inline(always)]
    pub fn get_num_moves(&self) -> NumMoves {
        self.stack.len() as NumMoves
    }

    #[inline(always)]
    pub fn get_num_repetitions(&self) -> u8 {
        self.repetition_table.get_repetition(self.hash())
    }

    #[inline(always)]
    pub fn is_repetition(&self, n_times: usize) -> bool {
        self.get_num_repetitions() as usize >= n_times
    }

    #[inline(always)]
    pub fn gives_repetition(&self, move_: Move) -> bool {
        self.repetition_table
            .get_repetition(self.sub_board.make_move_new(move_).hash())
            != 0
    }

    #[inline(always)]
    pub fn gives_threefold_repetition(&self, move_: Move) -> bool {
        self.repetition_table
            .get_repetition(self.sub_board.make_move_new(move_).hash())
            == 2
    }

    pub fn gives_claimable_threefold_repetition(&self, move_: Move) -> bool {
        //TODO: check if this is correct
        let new_board = self.sub_board.make_move_new(move_);
        MoveGenerator::new_legal(&new_board).any(|m| {
            let hash = new_board.make_move_new(m).hash();
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

    #[inline(always)]
    pub fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    #[inline(always)]
    fn is_halfmoves(&self, n: u8) -> bool {
        self.get_halfmove_clock() >= n
    }

    #[inline(always)]
    pub fn is_fifty_moves(&self) -> bool {
        self.is_halfmoves(100)
    }

    #[inline(always)]
    pub fn is_stalemate(&self) -> bool {
        self.status() == BoardStatus::Stalemate
    }

    #[inline(always)]
    pub fn is_other_draw(&self) -> bool {
        self.is_fifty_moves() || self.is_threefold_repetition() || self.is_insufficient_material()
    }

    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        self.is_other_draw() || self.is_stalemate()
    }

    #[inline(always)]
    pub fn is_game_over(&self) -> bool {
        self.is_other_draw() || self.status() != BoardStatus::Ongoing
    }

    // pub fn is_double_pawn_push(&self, move_: Move) -> bool {
    //     let source = move_.get_source();
    //     let dest = move_.get_dest();
    //     let pawn_mask = self.get_piece_mask(Pawn);
    // }

    #[inline(always)]
    pub fn is_quiet(&self, move_: Move) -> bool {
        !(self.is_capture(move_) || self.gives_check(move_))
    }

    #[inline(always)]
    pub fn has_legal_en_passant(&self) -> bool {
        self.ep_square().is_some()
    }

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

    #[inline(always)]
    pub fn is_irreversible(&self, move_: Move) -> bool {
        self.has_legal_en_passant() || self.is_zeroing(move_) || self.reduces_castling_rights(move_)
    }

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
        self.repetition_table.insert(self.hash());
        self.stack.push((sub_board_copy, optional_move));
    }

    pub fn pop(&mut self) -> Option<Move> {
        let (sub_board, optional_move) = self.stack.pop().unwrap();
        self.repetition_table.remove(self.hash());
        self.sub_board = sub_board;
        optional_move
    }

    #[inline(always)]
    pub fn get_all_moves(&self) -> Vec<&Option<Move>> {
        self.stack.iter().map(|(_, m)| m).collect_vec()
    }

    #[inline(always)]
    pub fn get_last_move(&self) -> Option<Option<Move>> {
        self.stack.last().map(|(_, m)| *m)
    }

    #[inline(always)]
    pub fn contains_null_move(&self) -> bool {
        self.stack.iter().any(|(_, m)| m.is_none())
    }

    #[inline(always)]
    pub fn get_ply(&self) -> usize {
        self.stack.len()
    }

    #[inline(always)]
    pub fn has_empty_stack(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn parse_san(&self, mut san: &str) -> Result<Option<Move>, EngineError> {
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

    #[inline(always)]
    pub fn parse_uci(&self, uci: &str) -> Result<Option<Move>, EngineError> {
        if uci == "0000" {
            return Ok(None);
        }
        Ok(Some(Move::from_str(uci)?))
    }

    #[inline(always)]
    pub fn parse_move(&self, move_text: &str) -> Result<Option<Move>, EngineError> {
        self.parse_uci(move_text).or(self.parse_san(move_text))
    }

    pub fn push_san(&mut self, san: &str) -> Result<Option<Move>, EngineError> {
        let move_ = self.parse_san(san)?;
        self.push(move_);
        Ok(move_)
    }

    #[inline(always)]
    pub fn push_sans(&mut self, sans: &str) -> Result<Vec<Option<Move>>, EngineError> {
        remove_double_spaces_and_trim(sans)
            .split(' ')
            .map(|san| self.push_san(san))
            .collect()
    }

    pub fn push_uci(&mut self, uci: &str) -> Result<Option<Move>, EngineError> {
        let move_ = self.parse_uci(uci)?;
        self.push(move_);
        Ok(move_)
    }

    #[inline(always)]
    pub fn push_str(&mut self, s: &str) {
        self.push_uci(s).unwrap();
    }

    #[inline(always)]
    pub fn push_uci_moves(&mut self, uci_moves: &str) -> Result<Vec<Option<Move>>, EngineError> {
        remove_double_spaces_and_trim(uci_moves)
            .split(' ')
            .map(|san| self.push_uci(san))
            .collect()
    }

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

    #[inline(always)]
    pub fn san_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
    ) -> Result<String, BoardError> {
        self.algebraic_and_push(optional_move.into(), false)
    }

    #[inline(always)]
    pub fn lan_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
    ) -> Result<String, BoardError> {
        self.algebraic_and_push(optional_move.into(), true)
    }

    /// Given a sequence of moves, returns a string representing the sequence
    /// in standard algebraic notation (e.g., ``1. e4 e5 2. Nf3 Nc6`` or
    /// ``37...Bg6 38. fxg6``).

    /// The board will not be modified as a result of calling this.

    /// panics if any moves in the sequence are illegal.
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

    /// Returns a string representing the game in Portable Game Notation (PGN).
    /// The result of the game is included in the tags.
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

    fn mini_perft(&mut self, depth: Depth, print_move: bool) -> usize {
        let moves = self.generate_legal_moves();
        if depth == 1 {
            return moves.len();
        }
        let mut count: usize = 0;
        for move_ in moves {
            self.push(move_);
            let c_count = self.mini_perft(depth - 1, false);
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

    #[inline(always)]
    pub fn perft(&mut self, depth: Depth) -> usize {
        self.mini_perft(depth, true)
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
        board.repetition_table.insert(board.hash());
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
    pub fn hash(&self) -> u64,
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
