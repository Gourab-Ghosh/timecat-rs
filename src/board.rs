use super::*;

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

#[derive(Clone, Debug)]
struct BoardState {
    board: chess::Board,
    ep_square: Option<Square>,
    halfmove_clock: u8,
    fullmove_number: NumMoves,
    num_repetitions: u8,
}

#[derive(Clone)]
pub struct Board {
    board: chess::Board,
    evaluator: Arc<Mutex<Evaluator>>,
    stack: Vec<(BoardState, Option<Move>)>,
    ep_square: Option<Square>,
    halfmove_clock: u8,
    fullmove_number: NumMoves,
    num_repetitions: u8,
    starting_fen: String,
    repetition_table: RepetitionTable,
}

impl Board {
    pub fn new(evaluator: Arc<Mutex<Evaluator>>) -> Self {
        let mut board = Self {
            board: chess::Board::from_str(STARTING_FEN).unwrap(),
            evaluator,
            stack: Vec::new(),
            ep_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            num_repetitions: 0,
            starting_fen: STARTING_FEN.to_string(),
            repetition_table: RepetitionTable::new(),
        };
        board.num_repetitions = board
            .repetition_table
            .insert_and_get_repetition(board.hash());
        board
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<(), chess::Error> {
        if !Self::is_good_fen(fen) {
            return Err(chess::Error::InvalidFen {
                fen: fen.to_string(),
            });
        }
        let fen = simplify_fen(fen);
        if fen == self.get_fen() {
            self.starting_fen = self.get_fen();
            return Ok(());
        }
        self.board = chess::Board::from_str(&fen).expect("FEN not parsed properly!");
        let mut splitted_fen = fen.split(' ');
        self.ep_square = match splitted_fen.nth(3).unwrap_or("-") {
            "-" => None,
            square => Some(Square::from_str(square).expect("Invalid en_passant square!")),
        };
        self.halfmove_clock = splitted_fen.next().unwrap_or("0").parse().unwrap();
        self.fullmove_number = splitted_fen.next().unwrap_or("1").parse().unwrap();
        self.repetition_table.clear();
        self.num_repetitions = self.repetition_table.insert_and_get_repetition(self.hash());
        self.starting_fen = self.get_fen();
        Ok(())
    }

    pub fn from_fen(fen: &str) -> Result<Self, chess::Error> {
        let fen = simplify_fen(fen);
        let mut board = Self::new(Default::default());
        board.set_fen(&fen)?;
        Ok(board)
    }

    pub fn get_evaluator(&self) -> Arc<Mutex<Evaluator>> {
        self.evaluator.clone()
    }

    pub fn set_evaluator(&mut self, evaluator: Arc<Mutex<Evaluator>>) {
        self.evaluator = evaluator;
    }

    pub fn get_fen(&self) -> String {
        // TODO: check later
        let parent_class_fen = self.board.to_string();
        let splitted_parent_class_fen = parent_class_fen.split(' ');
        let mut fen = String::new();
        for (i, part) in splitted_parent_class_fen.enumerate() {
            fen.push_str(part);
            fen.push(' ');
            if i == 3 {
                break;
            }
        }
        fen.push_str(&format!("{} {}", self.halfmove_clock, self.fullmove_number));
        fen
    }

    pub fn get_sub_board(&self) -> chess::Board {
        self.board
    }

    pub fn is_good_fen(fen: &str) -> bool {
        let fen = simplify_fen(fen);
        if chess::Board::from_str(&fen).is_err() {
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
        self.set_fen(STARTING_FEN).unwrap();
    }

    pub fn clear(&mut self) {
        self.set_fen(EMPTY_FEN).unwrap();
    }

    pub fn piece_type_at(&self, square: Square) -> usize {
        match self.board.piece_on(square) {
            None => 0,
            Some(p) => p.to_index() + 1,
        }
    }

    pub fn color_at(&self, square: Square) -> Option<Color> {
        self.board.color_on(square)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board.piece_on(square)
    }

    pub fn piece_symbol_at(&self, square: Square) -> String {
        let symbol = get_item_unchecked!(PIECE_SYMBOLS, self.piece_type_at(square)).to_string();
        if let Some(color) = self.color_at(square) {
            return match color {
                White => symbol.to_uppercase(),
                Black => symbol.to_lowercase(),
            };
        }
        symbol
    }

    pub fn piece_unicode_symbol_at(&self, square: Square, flip_color: bool) -> String {
        if let Some(color) = self.color_at(square) {
            let piece_index = self.piece_at(square).unwrap().to_index();
            let (white_pieces, black_pieces) = match flip_color {
                true => (BLACK_PIECE_UNICODE_SYMBOLS, WHITE_PIECE_UNICODE_SYMBOLS),
                false => (WHITE_PIECE_UNICODE_SYMBOLS, BLACK_PIECE_UNICODE_SYMBOLS),
            };
            return match color {
                White => get_item_unchecked!(white_pieces, piece_index),
                Black => get_item_unchecked!(black_pieces, piece_index),
            }
            .to_string();
        }
        EMPTY_SPACE_UNICODE_SYMBOL.to_string()
    }

    fn get_skeleton(&self) -> String {
        let skeleton = String::from(BOARD_SKELETON.trim_matches('\n'));
        let mut colored_skeleton = String::new();
        fn get_colored_char(c: char) -> String {
            let mut _char = c.to_string();
            let style = if "+-|".contains(c) {
                BOARD_SKELETON_STYLE
            } else if "abcdefghABCDEFGH12345678".contains(c) {
                BOARD_LABEL_STYLE
            } else {
                ""
            };
            colorize(_char, style)
        }
        for c in skeleton.chars() {
            colored_skeleton.push_str(&get_colored_char(c));
        }
        colored_skeleton
    }

    pub fn get_checkers(&self) -> BitBoard {
        return *self.board.checkers();
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        self.board.king_square(color)
    }

    fn to_board_string(&self, use_unicode: bool) -> String {
        let mut skeleton = self.get_skeleton();
        let checkers = self.get_checkers();
        let king_square = self.get_king_square(self.board.side_to_move());
        let last_move = self.stack.last().and_then(|(_, m)| *m);
        for square in SQUARES_180 {
            let symbol = if use_unicode {
                self.piece_unicode_symbol_at(square, false)
            } else {
                self.piece_symbol_at(square)
            };
            let mut style = String::new();
            if symbol != " " {
                style += match self.color_at(square).unwrap() {
                    White => WHITE_PIECES_STYLE,
                    Black => BLACK_PIECES_STYLE,
                };
                if square == king_square && checkers != BB_EMPTY {
                    style += " ";
                    style += CHECK_STYLE;
                }
            }
            if last_move.is_some()
                && [
                    last_move.unwrap().get_source(),
                    last_move.unwrap().get_dest(),
                ]
                .contains(&square)
            {
                style += " ";
                style += LAST_MOVE_HIGHLIGHT_STYLE;
            }
            skeleton = skeleton.replacen('O', &colorize(symbol, &style), 1);
        }
        skeleton.push('\n');
        let mut checkers_string = String::new();
        for square in checkers {
            checkers_string += &square.to_string();
            checkers_string += " ";
        }
        skeleton.push_str(
            &[
                String::new(),
                format_info("Fen", self.get_fen()),
                format_info("Transposition Key", hash_to_string(self.hash())),
                format_info(
                    "Checkers",
                    colorize(checkers_string.trim().to_uppercase(), CHECKERS_STYLE),
                ),
                format_info("Current Evaluation", score_to_string(self.evaluate())),
            ]
            .join("\n"),
        );
        skeleton
    }

    #[inline(always)]
    pub fn to_unicode_string(&self) -> String {
        self.to_board_string(true)
    }

    fn get_board_state(&self) -> BoardState {
        BoardState {
            board: self.board,
            ep_square: self.ep_square,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            num_repetitions: self.num_repetitions,
        }
    }

    #[inline(always)]
    pub fn turn(&self) -> Color {
        self.board.side_to_move()
    }

    #[inline(always)]
    pub fn occupied(&self) -> &BitBoard {
        return self.board.combined();
    }

    #[inline(always)]
    pub fn occupied_co(&self, color: Color) -> &BitBoard {
        return self.board.color_combined(color);
    }

    #[inline(always)]
    pub fn black_occupied(&self) -> &BitBoard {
        return self.board.color_combined(Black);
    }

    #[inline(always)]
    pub fn white_occupied(&self) -> &BitBoard {
        return self.board.color_combined(White);
    }

    #[inline(always)]
    pub fn is_check(&self) -> bool {
        self.get_checkers() != BB_EMPTY
    }

    #[inline(always)]
    pub fn is_checkmate(&self) -> bool {
        self.status() == BoardStatus::Checkmate
    }

    pub fn gives_check(&self, move_: Move) -> bool {
        let mut temp_board = self.board;
        self.board.make_move(move_, &mut temp_board);
        return temp_board.checkers() != &BB_EMPTY;
    }

    pub fn gives_checkmate(&self, move_: Move) -> bool {
        let mut temp_board = self.board;
        self.board.make_move(move_, &mut temp_board);
        temp_board.status() == BoardStatus::Checkmate
    }

    #[inline(always)]
    pub fn status(&self) -> BoardStatus {
        self.board.status()
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
    pub fn get_halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    #[inline(always)]
    pub fn get_fullmove_number(&self) -> NumMoves {
        self.fullmove_number
    }

    #[inline(always)]
    pub fn get_num_repetitions(&self) -> u8 {
        self.num_repetitions
    }

    #[inline(always)]
    pub fn is_repetition(&self, n_times: usize) -> bool {
        self.get_num_repetitions() as usize >= n_times
    }

    pub fn gives_repetition(&self, move_: Move) -> bool {
        let mut new_board = self.get_sub_board();
        new_board.clone().make_move(move_, &mut new_board);
        self.repetition_table.get_repetition(new_board.get_hash()) != 0
    }

    pub fn gives_threefold_repetition(&self, move_: Move) -> bool {
        let mut new_board = self.get_sub_board();
        new_board.clone().make_move(move_, &mut new_board);
        self.repetition_table.get_repetition(new_board.get_hash()) == 2
    }

    pub fn gives_claimable_threefold_repetition(&mut self, move_: Move) -> bool {
        self.push(Some(move_));
        if self.is_threefold_repetition() {
            self.pop();
            return true;
        }
        if self
            .generate_legal_moves()
            .any(|m| self.gives_threefold_repetition(m))
        {
            self.pop();
            return true;
        }
        self.pop();
        false
    }

    #[inline(always)]
    pub fn is_threefold_repetition(&self) -> bool {
        self.is_repetition(3)
    }

    #[inline(always)]
    fn is_halfmoves(&self, n: u8) -> bool {
        self.halfmove_clock >= n
    }

    #[inline(always)]
    pub fn is_fifty_moves(&self) -> bool {
        self.is_halfmoves(100)
    }

    pub fn has_insufficient_material(&self, color: Color) -> bool {
        let occupied = self.occupied_co(color);
        return match occupied.popcnt() {
            1 => true,
            2 => {
                (self.get_piece_mask(Rook) | self.get_piece_mask(Queen) | self.get_piece_mask(Pawn))
                    & occupied
                    == BB_EMPTY
            }
            _ => false,
        };
    }

    #[inline(always)]
    pub fn has_non_pawn_material(&self) -> bool {
        self.get_piece_mask(Pawn) | self.get_piece_mask(King) != *self.occupied()
    }

    pub fn is_insufficient_material(&self) -> bool {
        match self.occupied().popcnt() {
            2 => true,
            3 => {
                self.get_piece_mask(Rook) == &BB_EMPTY
                    && self.get_piece_mask(Queen) == &BB_EMPTY
                    && self.get_piece_mask(Pawn) == &BB_EMPTY
            }
            4 => {
                self.get_piece_mask(Rook) == &BB_EMPTY
                    && self.get_piece_mask(Knight) == &BB_EMPTY
                    && self.get_piece_mask(Queen) == &BB_EMPTY
                    && self.get_piece_mask(Pawn) == &BB_EMPTY
                    && [0, 2].contains(&(BB_LIGHT_SQUARES & self.get_piece_mask(Bishop)).popcnt())
            }
            _ => false,
        }
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

    pub fn is_en_passant(&self, move_: Move) -> bool {
        match self.ep_square() {
            Some(ep_square) => {
                let source = move_.get_source();
                let dest = move_.get_dest();
                ep_square == dest
                    && self.get_piece_mask(Pawn) & get_square_bb(source) != BB_EMPTY
                    && [7, 9].contains(&dest.to_int().abs_diff(source.to_int()))
                    && self.occupied() & get_square_bb(dest) == BB_EMPTY
            }
            None => false,
        }
    }

    pub fn is_passed_pawn(&self, square: Square) -> bool {
        let pawn_mask = self.get_piece_mask(Pawn);
        let self_color = self.turn();
        if pawn_mask & self.occupied_co(self_color) & get_square_bb(square) == BB_EMPTY {
            return false;
        }
        let file = square.get_file();
        pawn_mask
            & self.occupied_co(!self_color)
            & (get_adjacent_files(file) | get_file_bb(file))
            & get_upper_board_mask(square.get_rank(), self_color)
            == BB_EMPTY
    }

    pub fn is_capture(&self, move_: Move) -> bool {
        let touched = get_square_bb(move_.get_source()) ^ get_square_bb(move_.get_dest());
        (touched & self.occupied_co(!self.turn())) != BB_EMPTY || self.is_en_passant(move_)
    }

    #[inline(always)]
    pub fn is_quiet(&self, move_: Move) -> bool {
        !(self.is_capture(move_) || self.gives_check(move_))
    }

    pub fn is_zeroing(&self, move_: Move) -> bool {
        let touched = get_square_bb(move_.get_source()) ^ get_square_bb(move_.get_dest());
        return touched & self.get_piece_mask(Pawn) != BB_EMPTY
            || (touched & self.occupied_co(!self.turn())) != BB_EMPTY;
    }

    #[inline(always)]
    pub fn get_en_passant_square(&self) -> Option<Square> {
        self.board.en_passant()
    }

    #[inline(always)]
    pub fn has_legal_en_passant(&self) -> bool {
        self.get_en_passant_square().is_some()
    }

    pub fn clean_castling_rights(&self) -> BitBoard {
        let white_castling_rights = match self.board.castle_rights(White) {
            chess::CastleRights::Both => BB_A1 | BB_H1,
            chess::CastleRights::KingSide => BB_H1,
            chess::CastleRights::QueenSide => BB_A1,
            chess::CastleRights::NoRights => BB_EMPTY,
        };
        let black_castling_rights = match self.board.castle_rights(Black) {
            chess::CastleRights::Both => BB_A8 | BB_H8,
            chess::CastleRights::KingSide => BB_H8,
            chess::CastleRights::QueenSide => BB_A8,
            chess::CastleRights::NoRights => BB_EMPTY,
        };
        white_castling_rights | black_castling_rights
    }

    #[inline(always)]
    pub fn get_piece_mask(&self, piece: Piece) -> &BitBoard {
        self.board.pieces(piece)
    }

    fn reduces_castling_rights(&self, move_: Move) -> bool {
        let cr = self.clean_castling_rights();
        let touched = get_square_bb(move_.get_source()) ^ get_square_bb(move_.get_dest());
        let touched_cr = touched & cr;
        let kings = self.get_piece_mask(King);
        let touched_kings_cr = touched_cr & kings;
        touched_cr != BB_EMPTY
            || BB_RANK_1 & touched_kings_cr & self.occupied_co(White) != BB_EMPTY
            || BB_RANK_8 & touched_kings_cr & self.occupied_co(Black) != BB_EMPTY
    }

    #[inline(always)]
    pub fn is_irreversible(&self, move_: Move) -> bool {
        self.has_legal_en_passant() || self.is_zeroing(move_) || self.reduces_castling_rights(move_)
    }

    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    pub fn is_castling(&self, move_: Move) -> bool {
        if (self.get_piece_mask(King) & get_square_bb(move_.get_source())) != BB_EMPTY {
            let rank_diff = move_
                .get_source()
                .get_file()
                .to_index()
                .abs_diff(move_.get_dest().get_file().to_index());
            return rank_diff > 1
                || self.get_piece_mask(Rook)
                    & self.occupied_co(self.turn())
                    & get_square_bb(move_.get_dest())
                    != BB_EMPTY;
        }
        false
    }

    #[inline(always)]
    pub fn get_num_pieces(&self) -> u32 {
        self.occupied().popcnt()
    }

    #[inline(always)]
    pub fn is_endgame(&self) -> bool {
        if self.get_num_pieces() <= ENDGAME_PIECE_THRESHOLD {
            return true;
        }
        match self.get_piece_mask(Queen).popcnt() {
            0 => {
                (self.get_piece_mask(Rook)
                    | self.get_piece_mask(Bishop)
                    | self.get_piece_mask(Knight))
                .popcnt()
                    <= 4
            }
            1 => {
                self.get_piece_mask(Rook).popcnt() <= 2
                    && self.get_piece_mask(Bishop) | self.get_piece_mask(Knight) == BB_EMPTY
            }
            2 => {
                self.get_piece_mask(Rook)
                    | self.get_piece_mask(Bishop)
                    | self.get_piece_mask(Knight)
                    == BB_EMPTY
            }
            _ => false,
        }
    }

    pub fn push(&mut self, optional_move: impl Into<Option<Move>>) {
        let optional_move = optional_move.into();
        let board_state = self.get_board_state();
        if self.turn() == Black {
            self.fullmove_number += 1;
        }
        if let Some(move_) = optional_move {
            if self.is_zeroing(move_) {
                self.halfmove_clock = 0;
            } else {
                self.halfmove_clock += 1;
            }
            self.board = self.board.make_move_new(move_);
        } else {
            self.halfmove_clock += 1;
            self.board = self
                .board
                .null_move()
                .expect("Trying to push null move while in check!");
        }
        self.ep_square = self
            .board
            .en_passant()
            .map(|ep_square| ep_square.forward(self.turn()).unwrap());
        self.num_repetitions = self.repetition_table.insert_and_get_repetition(self.hash());
        self.stack.push((board_state, optional_move));
    }

    fn restore(&mut self, board_state: BoardState) {
        self.board = board_state.board;
        self.halfmove_clock = board_state.halfmove_clock;
        self.fullmove_number = board_state.fullmove_number;
        self.num_repetitions = board_state.num_repetitions;
        self.ep_square = board_state.ep_square;
    }

    pub fn pop(&mut self) -> Option<Move> {
        let (board_state, optional_move) = self.stack.pop().unwrap();
        self.repetition_table.remove(self.hash());
        self.restore(board_state);
        optional_move
    }

    pub fn get_all_moves(&self) -> Vec<Option<Move>> {
        self.stack.iter().map(|(_, m)| *m).collect_vec()
    }

    pub fn get_last_move(&self) -> Option<Move> {
        self.stack.last().unwrap().1
    }

    pub fn contains_null_move(&self) -> bool {
        self.stack.iter().any(|(_, m)| m.is_none())
    }

    pub fn get_ply(&self) -> usize {
        self.stack.len()
    }

    #[inline(always)]
    pub fn has_empty_stack(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn parse_san(&self, san: &str) -> Result<Option<Move>, chess::Error> {
        if san == "--" {
            return Ok(None);
        }
        let san = san.replace('0', "O");
        for move_ in self.generate_legal_moves() {
            if self.san(move_).unwrap() == san {
                return Ok(Some(move_));
            }
        }
        Err(chess::Error::InvalidSanMove)
        // Move::from_san(&self.board, &san.replace('0', "O"))
    }

    #[inline(always)]
    pub fn parse_uci(&self, uci: &str) -> Result<Option<Move>, chess::Error> {
        if uci == "0000" {
            return Ok(None);
        }
        Ok(Some(Move::from_str(uci)?))
    }

    pub fn parse_move(&self, move_text: &str) -> Result<Option<Move>, chess::Error> {
        let possible_move = self.parse_san(move_text);
        if possible_move.is_err() {
            return self.parse_uci(move_text);
        }
        possible_move
    }

    pub fn push_san(&mut self, san: &str) -> Option<Move> {
        let move_ = self.parse_san(san).unwrap_or_else(|err| panic!("{}", err));
        self.push(move_);
        move_
    }

    pub fn push_sans(&mut self, sans: &str) {
        for san in remove_double_spaces(sans).split(' ') {
            self.push_san(san);
        }
    }

    pub fn push_uci(&mut self, uci: &str) -> Option<Move> {
        let move_ = self
            .parse_uci(uci)
            .unwrap_or_else(|_| panic!("Bad uci: {uci}"));
        self.push(move_);
        move_
    }

    #[inline(always)]
    pub fn push_str(&mut self, s: &str) {
        self.push_uci(s);
    }

    pub fn push_uci_moves(&mut self, uci_moves: &str) {
        for uci in remove_double_spaces(uci_moves).split(' ') {
            self.push_uci(uci);
        }
    }

    fn algebraic_without_suffix(
        &self,
        optional_move: Option<Move>,
        long: bool,
    ) -> Result<String, BoardError> {
        // Null move.
        if optional_move.is_none() {
            return Ok("--".to_string());
        }

        let move_ = optional_move.unwrap();

        // Castling.
        if self.is_castling(move_) {
            return if move_.get_dest().get_file() < move_.get_source().get_file() {
                Ok("O-O-O".to_string())
            } else {
                Ok("O-O".to_string())
            };
        }

        let piece = self
            .piece_at(move_.get_source())
            .ok_or(BoardError::InvalidSanMove {
                move_,
                fen: self.get_fen(),
            })?;
        let capture = self.is_capture(move_);
        let mut san = if piece == Pawn {
            String::new()
        } else {
            piece.to_string(White)
        };

        if long {
            san += &move_.get_source().to_string();
        } else if piece != Pawn {
            // Get ambiguous move candidates.
            // Relevant candidates: not exactly the current move,
            // but to the same square.
            let mut others = BB_EMPTY;
            let from_mask = self.get_piece_mask(piece)
                & self.occupied_co(self.turn())
                & !get_square_bb(move_.get_source());
            let to_mask = get_square_bb(move_.get_dest());
            for candidate in self
                .generate_masked_legal_moves(to_mask)
                .filter(|m| get_square_bb(m.get_source()) & from_mask != BB_EMPTY)
            {
                others |= get_square_bb(candidate.get_source());
            }

            // Disambiguate.
            if others != BB_EMPTY {
                let (mut row, mut column) = (false, false);
                if others & get_rank_bb(move_.get_source().get_rank()) != BB_EMPTY {
                    column = true;
                }
                if others & get_file_bb(move_.get_source().get_file()) != BB_EMPTY {
                    row = true;
                } else {
                    column = true;
                }
                if column {
                    san.push(
                        "abcdefgh"
                            .chars()
                            .nth(move_.get_source().get_file().to_index())
                            .unwrap(),
                    );
                }
                if row {
                    san += &(move_.get_source().get_rank().to_index() + 1).to_string();
                }
            }
        } else if capture {
            san.push(
                "abcdefgh"
                    .chars()
                    .nth(move_.get_source().get_file().to_index())
                    .unwrap(),
            );
        }

        // Captures.
        if capture {
            san += "x";
        } else if long {
            san += "-";
        }

        // Destination square.
        san += &move_.get_dest().to_string();

        // Promotion.
        if let Some(promotion) = move_.get_promotion() {
            san += &format!("={}", promotion.to_string(White))
        }

        Ok(san)
    }

    pub fn algebraic_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
        long: bool,
    ) -> Result<String, BoardError> {
        let optional_move = optional_move.into();
        let san = self.algebraic_without_suffix(optional_move, long)?;

        // Look ahead for check or checkmate.
        self.push(optional_move);
        let is_check = self.is_check();
        let is_checkmate = is_check && self.is_checkmate();

        // Add check or checkmate suffix.
        if is_checkmate && optional_move.is_some() {
            Ok(san + "#")
        } else if is_check && optional_move.is_some() {
            Ok(san + "+")
        } else {
            Ok(san)
        }
    }

    #[inline(always)]
    fn algebraic(&self, optional_move: Option<Move>, long: bool) -> Result<String, BoardError> {
        self.clone().algebraic_and_push(optional_move, long)
    }

    /// Gets the standard algebraic notation of the given move in the context
    /// of the current position.
    #[inline(always)]
    pub fn san(&self, optional_move: impl Into<Option<Move>>) -> Result<String, BoardError> {
        self.algebraic(optional_move.into(), false)
    }

    pub fn uci(optional_move: impl Into<Option<Move>>) -> Result<String, BoardError> {
        if let Some(move_) = optional_move.into() {
            Ok(move_.to_string())
        } else {
            Ok("0000".to_string())
        }
    }

    pub fn stringify_move(
        &self,
        optional_move: impl Into<Option<Move>>,
    ) -> Result<String, BoardError> {
        let optional_move = optional_move.into();
        if is_in_uci_mode() {
            Self::uci(optional_move)
        } else {
            self.san(optional_move)
        }
    }

    #[inline(always)]
    pub fn san_and_push(
        &mut self,
        optional_move: impl Into<Option<Move>>,
    ) -> Result<String, BoardError> {
        self.algebraic_and_push(optional_move.into(), false)
    }

    /// Gets the long algebraic notation of the given move in the context of
    /// the current position.
    #[inline(always)]
    pub fn lan(&self, optional_move: impl Into<Option<Move>>) -> Result<String, BoardError> {
        self.algebraic(optional_move.into(), true)
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
                san.push(format!("{}. {}", board.fullmove_number, san_str.unwrap()));
            } else if san.is_empty() {
                let san_str = board.san_and_push(optional_move);
                san.push(format!("{}...{}", board.fullmove_number, san_str.unwrap()));
            } else {
                san.push(board.san_and_push(optional_move).unwrap().to_string());
            }
        }
        let mut san_string = String::new();
        for s in san {
            san_string += &(s + " ");
        }
        return san_string.trim().to_string();
    }

    /// Returns a string representing the game in Portable Game Notation (PGN).
    /// The result of the game is included in the tags.
    pub fn get_pgn(&self) -> String {
        let mut pgn = String::new();
        if self.starting_fen != STARTING_FEN {
            pgn += &format!("[FEN \"{}\"]", self.starting_fen);
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

    #[inline(always)]
    pub fn is_legal(&self, move_: Move) -> bool {
        self.board.legal(move_)
    }

    pub fn generate_masked_legal_moves(&self, to_bitboard: BitBoard) -> chess::MoveGen {
        let mut moves = chess::MoveGen::new_legal(&self.board);
        moves.set_iterator_mask(to_bitboard);
        moves
    }

    #[inline(always)]
    pub fn generate_legal_moves(&self) -> chess::MoveGen {
        chess::MoveGen::new_legal(&self.board)
    }

    pub fn generate_legal_captures(&self) -> chess::MoveGen {
        let targets = self.occupied_co(!self.turn());
        self.generate_masked_legal_moves(*targets)
    }

    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.board.get_hash()
    }

    #[inline(always)]
    pub fn get_pawn_hash(&self) -> u64 {
        self.board.get_pawn_hash()
    }

    #[inline(always)]
    pub fn score_flipped(&self, score: Score) -> Score {
        if self.turn() == White {
            score
        } else {
            -score
        }
    }

    pub fn get_material_score(&self) -> Score {
        let mut score = 0;
        let black_occupied = self.black_occupied();
        for &piece in chess::ALL_PIECES[..5].iter() {
            let piece_mask = self.get_piece_mask(piece);
            if piece_mask == &BB_EMPTY {
                continue;
            }
            score += (piece_mask.popcnt() as Score
                - 2 * (piece_mask & black_occupied).popcnt() as Score)
                * evaluate_piece(piece);
        }
        score
    }

    #[inline(always)]
    pub fn get_winning_side(&self) -> Option<Color> {
        let material_score = self.get_material_score();
        if material_score.is_positive() {
            Some(White)
        } else if material_score.is_negative() {
            Some(Black)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_material_score_flipped(&self) -> Score {
        self.score_flipped(self.get_material_score())
    }

    #[inline(always)]
    pub fn get_masked_material_score_abs(&self, mask: &BitBoard) -> Score {
        chess::ALL_PIECES[..5]
            .iter()
            .map(|&piece| {
                evaluate_piece(piece) * (self.get_piece_mask(piece) & mask).popcnt() as Score
            })
            .sum()
    }

    #[inline(always)]
    pub fn get_material_score_abs(&self) -> Score {
        chess::ALL_PIECES[..5]
            .iter()
            .map(|&piece| evaluate_piece(piece) * self.get_piece_mask(piece).popcnt() as Score)
            .sum()
    }

    #[inline(always)]
    pub fn get_non_pawn_material_score_abs(&self) -> Score {
        chess::ALL_PIECES[1..5]
            .iter()
            .map(|&piece| evaluate_piece(piece) * self.get_piece_mask(piece).popcnt() as Score)
            .sum()
    }

    #[inline(always)]
    pub fn evaluate(&self) -> Score {
        self.evaluator.lock().unwrap().evaluate(self)
    }

    #[inline(always)]
    pub fn evaluate_flipped(&self) -> Score {
        self.score_flipped(self.evaluate())
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
                    colorize(move_, PERFT_MOVE_STYLE),
                    colorize(c_count, PERFT_COUNT_STYLE),
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

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_board_string(false))
    }
}

impl Default for Board {
    fn default() -> Self {
        STARTING_FEN.into()
    }
}

impl FromStr for Board {
    type Err = chess::Error;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        Self::from_fen(fen)
    }
}

impl From<&str> for Board {
    fn from(fen: &str) -> Self {
        Self::from_fen(fen).unwrap()
    }
}