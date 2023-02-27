use super::*;

// #[derive(Clone, PartialEq, Debug, Eq)]
#[derive(Clone)]
struct BoardState {
    board: chess::Board,
    halfmove_clock: u8,
    fullmove_number: u8,
    num_repetitions: u8,
}

pub mod perft_test {
    use super::*;

    fn mini_perft(board: &mut Board, depth: u8, print_move: bool) -> usize {
        let _moves = board.generate_legal_moves();
        // if depth == 0 {return 1;}
        if depth == 1 {
            return _moves.len();
        }
        let mut count: usize = 0;
        for _move in _moves {
            board.push(_move);
            let c_count = mini_perft(board, depth - 1, false);
            board.pop();
            if print_move {
                println!(
                    "{}: {}",
                    colorize(_move, PERFT_MOVE_STYLE),
                    colorize(c_count, PERFT_COUNT_STYLE),
                );
            }
            count += c_count;
        }
        count
    }

    pub fn perft(board: &mut Board, depth: u8) -> usize {
        mini_perft(board, depth, true)
    }
}

pub struct Board {
    board: chess::Board,
    stack: Vec<(BoardState, Move)>,
    halfmove_clock: u8,
    fullmove_number: u8,
    num_repetitions: u8,
    pub evaluator: Option<Evaluator>,
    repetition_table: RepetitionTable,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            board: chess::Board::from_str(STARTING_BOARD_FEN).unwrap(),
            stack: Vec::new(),
            halfmove_clock: 0,
            fullmove_number: 1,
            num_repetitions: 0,
            evaluator: Some(Evaluator::new()),
            repetition_table: RepetitionTable::new(),
        };
        board.num_repetitions = board
            .repetition_table
            .insert_and_get_repetition(board.get_hash());
        for square in *board.occupied() {
            let piece = board.piece_at(square).unwrap();
            let color = board.color_at(square).unwrap();
            board
                .evaluator
                .as_mut()
                .expect("No Evaluator found!")
                .activate_nnue(piece, color, square);
        }
        board
    }

    pub fn set_fen(&mut self, fen: &str) {
        for square in *self.occupied() {
            let piece = self.piece_at(square).unwrap();
            let color = self.color_at(square).unwrap();
            self.evaluator
                .as_mut()
                .expect("No Evaluator found!")
                .deactivate_nnue(piece, color, square);
        }
        let fen = simplify_fen(fen);
        self.board = chess::Board::from_str(fen.as_str()).expect("Valid Position");
        let mut splitted_fen = fen.split(' ');
        self.halfmove_clock = splitted_fen.nth(4).unwrap_or("0").parse().unwrap();
        self.fullmove_number = splitted_fen.next().unwrap_or("1").parse().unwrap();
        self.repetition_table.clear();
        self.num_repetitions = self
            .repetition_table
            .insert_and_get_repetition(self.get_hash());
        for square in *self.occupied() {
            let piece = self.piece_at(square).unwrap();
            let color = self.color_at(square).unwrap();
            self.evaluator
                .as_mut()
                .expect("No Evaluator found!")
                .activate_nnue(piece, color, square);
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let fen = simplify_fen(fen);
        let mut board = Self::new();
        board.set_fen(&fen);
        board
    }

    pub fn from_str(s: &str) -> Self {
        Self::from_fen(s)
    }

    pub fn from(board: &Self) -> Self {
        Self::from_fen(&board.get_fen())
    }

    pub fn get_fen(&self) -> String {
        // check later
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
        fen.push_str(format!("{} {}", self.halfmove_clock, self.fullmove_number).as_str());
        fen
    }

    pub fn get_sub_board(&self) -> chess::Board {
        self.board
    }

    pub fn is_good_fen(fen: &str) -> bool {
        let fen = simplify_fen(fen);
        if chess::Board::from_str(fen.as_str()).is_err() {
            return false;
        }
        let mut splitted_fen = fen.split(' ');
        if splitted_fen.nth(4).unwrap_or("0").parse().unwrap_or(-1) < 0 {
            return false;
        };
        if splitted_fen.next().unwrap_or("1").parse().unwrap_or(-1) < 0 {
            return false;
        };
        if splitted_fen.next().is_some() {
            return false;
        };
        true
    }

    pub fn empty() -> Self {
        Self::from_fen(EMPTY_BOARD_FEN)
    }

    pub fn reset(&mut self) {
        self.set_fen(STARTING_BOARD_FEN);
    }

    pub fn clear(&mut self) {
        self.set_fen(EMPTY_BOARD_FEN);
    }

    pub fn piece_type_at(&self, square: Square) -> u8 {
        match self.board.piece_on(square) {
            None => 0,
            Some(p) => match p {
                Pawn => 1,
                Knight => 2,
                Bishop => 3,
                Rook => 4,
                Queen => 5,
                King => 6,
            },
        }
    }

    pub fn color_at(&self, square: Square) -> Option<Color> {
        self.board.color_on(square)
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board.piece_on(square)
    }

    pub fn piece_symbol_at(&self, square: Square) -> String {
        let mut symbol = match self.piece_at(square) {
            Some(p) => match p {
                Pawn => "p",
                Knight => "n",
                Bishop => "b",
                Rook => "r",
                Queen => "q",
                King => "k",
            },
            None => " ",
        }
        .to_string();
        symbol = match self.color_at(square) {
            Some(c) => match c {
                White => symbol.to_uppercase(),
                Black => symbol.to_lowercase(),
            },
            None => symbol,
        };
        symbol
    }

    pub fn piece_unicode_symbol_at(&self, square: Square, flip_color: bool) -> String {
        let piece_type = self.piece_type_at(square);
        let white_pieces: [&str; 7];
        let black_pieces: [&str; 7];
        if flip_color {
            white_pieces = BLACK_PIECE_UNICODE_SYMBOLS;
            black_pieces = WHITE_PIECE_UNICODE_SYMBOLS;
        } else {
            white_pieces = WHITE_PIECE_UNICODE_SYMBOLS;
            black_pieces = BLACK_PIECE_UNICODE_SYMBOLS;
        }
        match self.color_at(square) {
            Some(color) => match color {
                White => white_pieces[piece_type as usize],
                Black => black_pieces[piece_type as usize],
            },
            None => " ",
        }
        .to_string()
    }

    pub fn repr(&self) -> String {
        stringify!(self.board).to_string()
    }

    fn get_skeleton(&self) -> String {
        let skeleton = String::from(" +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 8\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 7\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 6\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 5\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 4\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 3\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 2\n +---+---+---+---+---+---+---+---+\n | {} | {} | {} | {} | {} | {} | {} | {} | 1\n +---+---+---+---+---+---+---+---+\n   a   b   c   d   e   f   g   h");
        let mut colored_skeleton = String::new();
        fn get_colored_char(c: char) -> String {
            let mut _char = c.to_string();
            let style = if "+-|".contains(c) {
                BOARD_SKELETON_STYLE
            } else if "abcdefgh12345678".contains(c) {
                BOARD_LABEL_STYLE
            } else {
                ""
            };
            colorize(_char, style)
        }
        for c in skeleton.chars() {
            colored_skeleton.push_str(get_colored_char(c).as_str());
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
        let last_move = self.stack.last().map(|(_, m)| m);
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
            skeleton = skeleton.replacen("{}", colorize(symbol, style.as_str()).as_str(), 1);
        }
        skeleton.push('\n');
        let mut checkers_string = String::new();
        for square in checkers {
            checkers_string += square.to_string().as_str();
            checkers_string += " ";
        }
        skeleton.push_str(
            format!(
                "\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
                colorize("Fen", INFO_STYLE),
                self.get_fen(),
                colorize("Transposition Key", INFO_STYLE),
                hash_to_string(self.get_hash()),
                colorize("Checkers", INFO_STYLE),
                colorize(checkers_string.trim(), CHECKERS_STYLE),
                colorize("Current Evaluation", INFO_STYLE),
                score_to_string(self.evaluate()),
            )
            .as_str(),
        );
        skeleton
    }

    pub fn to_unicode_string(&self) -> String {
        self.to_board_string(true)
    }

    fn get_board_state(&self) -> BoardState {
        BoardState {
            board: self.board,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            num_repetitions: self.num_repetitions,
        }
    }

    pub fn turn(&self) -> Color {
        self.board.side_to_move()
    }

    pub fn occupied(&self) -> &BitBoard {
        return self.board.combined();
    }

    pub fn occupied_co(&self, color: Color) -> &BitBoard {
        return self.board.color_combined(color);
    }

    pub fn black_occupied(&self) -> &BitBoard {
        return self.board.color_combined(Black);
    }

    pub fn white_occupied(&self) -> &BitBoard {
        return self.board.color_combined(White);
    }

    pub fn is_check(&self) -> bool {
        self.get_checkers() != BB_EMPTY
    }

    pub fn is_checkmate(&self) -> bool {
        if self.is_check() {
            return self.generate_legal_moves().len() == 0;
        }
        false
    }

    pub fn gives_check(&self, _move: Move) -> bool {
        let mut temp_board = self.board;
        self.board.make_move(_move, &mut temp_board);
        return temp_board.checkers() != &BB_EMPTY;
    }

    pub fn status(&self) -> BoardStatus {
        self.board.status()
    }

    pub fn get_current_position_repetition(&self) -> u8 {
        self.num_repetitions
    }

    pub fn is_threefold_repetition(&self) -> bool {
        self.num_repetitions >= 3
    }

    fn is_halfmoves(&self, n: u8) -> bool {
        self.halfmove_clock >= n
    }

    pub fn is_fifty_moves(&self) -> bool {
        self.is_halfmoves(100)
    }

    fn has_insufficient_material(&self, color: Color) -> bool {
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

    fn is_insufficient_material(&self) -> bool {
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
                    && [0, 2]
                        .contains(&(self.white_occupied() | self.get_piece_mask(Bishop)).popcnt())
            }
            _ => false,
        }
    }

    pub fn is_stalemate(&self) -> bool {
        self.board.status() == BoardStatus::Stalemate
    }

    pub fn is_other_draw(&self) -> bool {
        self.is_threefold_repetition() || self.is_fifty_moves() || self.is_insufficient_material()
    }

    pub fn is_draw(&self) -> bool {
        self.is_other_draw() || self.is_stalemate()
    }

    pub fn is_game_over(&self) -> bool {
        self.is_other_draw() || self.board.status() != BoardStatus::Ongoing
    }

    pub fn is_en_passant(&self, _move: Move) -> bool {
        if self.ep_square().is_none() {
            return false;
        }
        if self.ep_square().unwrap() != _move.get_dest() {
            return false;
        }
        if (self.get_piece_mask(Pawn) & get_square_bb(_move.get_source())) == BB_EMPTY {
            return false;
        }
        if 8u8.abs_diff(
            _move
                .get_dest()
                .to_int()
                .abs_diff(_move.get_source().to_int()),
        ) != 1
        {
            return false;
        }
        return (self.occupied() & get_square_bb(_move.get_dest())) == BB_EMPTY;
    }

    pub fn is_capture(&self, _move: Move) -> bool {
        let touched = get_square_bb(_move.get_source()) ^ get_square_bb(_move.get_dest());
        (touched & self.occupied_co(!self.turn())) != BB_EMPTY || self.is_en_passant(_move)
    }

    pub fn is_zeroing(&self, _move: Move) -> bool {
        let touched = get_square_bb(_move.get_source()) ^ get_square_bb(_move.get_dest());
        return touched & self.get_piece_mask(Pawn) != BB_EMPTY
            || (touched & self.occupied_co(!self.turn())) != BB_EMPTY;
    }

    pub fn get_enpassant_square(&self) -> Option<Square> {
        self.board.en_passant()
    }

    pub fn has_legal_en_passant(&self) -> bool {
        self.get_enpassant_square().is_some()
    }

    pub fn clean_castling_rights(&self) -> BitBoard {
        let white_castling_righrs = match self.board.castle_rights(White) {
            chess::CastleRights::Both => BB_A1 | BB_H1,
            chess::CastleRights::KingSide => BB_H1,
            chess::CastleRights::QueenSide => BB_A1,
            chess::CastleRights::NoRights => BB_EMPTY,
        };
        let black_castling_righrs = match self.board.castle_rights(Black) {
            chess::CastleRights::Both => BB_A8 | BB_H8,
            chess::CastleRights::KingSide => BB_H8,
            chess::CastleRights::QueenSide => BB_A8,
            chess::CastleRights::NoRights => BB_EMPTY,
        };
        white_castling_righrs | black_castling_righrs
    }

    pub fn get_piece_mask(&self, piece: Piece) -> &BitBoard {
        self.board.pieces(piece)
    }

    fn reduces_castling_rights(&self, _move: Move) -> bool {
        let cr = self.clean_castling_rights();
        let touched = get_square_bb(_move.get_source()) ^ get_square_bb(_move.get_dest());
        ((touched & cr) != BB_EMPTY)
            || ((cr & BB_RANK_1 & touched & self.get_piece_mask(King) & self.occupied_co(White))
                != BB_EMPTY)
            || ((cr & BB_RANK_8 & touched & self.get_piece_mask(King) & self.occupied_co(Black))
                != BB_EMPTY)
    }

    pub fn is_irreversible(&self, _move: Move) -> bool {
        self.is_zeroing(_move) || self.reduces_castling_rights(_move) || self.has_legal_en_passant()
    }

    pub fn ep_square(&self) -> Option<Square> {
        self.board.en_passant()
    }

    pub fn is_castling(&self, _move: Move) -> bool {
        if (self.get_piece_mask(King) & get_square_bb(_move.get_source())) != BB_EMPTY {
            let rank_diff = _move
                .get_source()
                .get_file()
                .to_index()
                .abs_diff(_move.get_dest().get_file().to_index());
            return rank_diff > 1
                || (self.get_piece_mask(Rook)
                    & self.occupied_co(self.turn())
                    & get_square_bb(_move.get_dest()))
                    != BB_EMPTY;
        }
        false
    }

    pub fn num_pieces(&self) -> u32 {
        self.occupied().popcnt()
    }

    pub fn is_endgame(&self) -> bool {
        self.num_pieces() <= 12
    }

    fn push_nnue(&mut self, _move: Move) {
        self.evaluator
            .as_mut()
            .expect("No Evaluator found!")
            .backup();
        let self_color = self.turn();
        let source = _move.get_source();
        let dest = _move.get_dest();
        let self_piece = self.piece_at(source).unwrap();
        self.evaluator
            .as_mut()
            .expect("No Evaluator found!")
            .deactivate_nnue(self_piece, self_color, source);
        if self.is_capture(_move) {
            let remove_piece_square = if self.is_en_passant(_move) {
                dest.backward(self_color).unwrap()
            } else {
                dest
            };
            let piece = self.piece_at(remove_piece_square).unwrap();
            self.evaluator
                .as_mut()
                .expect("No Evaluator found!")
                .deactivate_nnue(piece, !self_color, remove_piece_square);
        } else if self.is_castling(_move) {
            let (rook_source, rook_dest) = if _move.get_dest().get_file().to_index()
                > _move.get_source().get_file().to_index()
            {
                match self_color {
                    White => (Square::H1, Square::F1),
                    Black => (Square::H8, Square::F8),
                }
            } else {
                match self_color {
                    White => (Square::A1, Square::D1),
                    Black => (Square::A8, Square::D8),
                }
            };
            self.evaluator
                .as_mut()
                .expect("No Evaluator found!")
                .deactivate_nnue(Rook, self_color, rook_source);
            self.evaluator
                .as_mut()
                .expect("No Evaluator found!")
                .activate_nnue(Rook, self_color, rook_dest);
        }
        self.evaluator
            .as_mut()
            .expect("No Evaluator found!")
            .activate_nnue(
                _move.get_promotion().unwrap_or(self_piece),
                self_color,
                dest,
            );
    }

    pub fn push(&mut self, _move: Move) {
        let board_state = self.get_board_state();
        if self.is_zeroing(_move) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        if self.turn() == Black {
            self.fullmove_number += 1;
        }
        // self.push_nnue(_move);
        self.board.clone().make_move(_move, &mut self.board);
        self.num_repetitions = self
            .repetition_table
            .insert_and_get_repetition(self.get_hash());
        self.stack.push((board_state, _move));
    }

    pub fn push_null_move(&mut self) {
        let board_state = self.get_board_state();
        self.board = self
            .board
            .null_move()
            .expect("Trying to push null move while in check!");
        self.stack.push((board_state, Move::default()));
    }

    fn restore(&mut self, board_state: BoardState) {
        self.board = board_state.board;
        self.halfmove_clock = board_state.halfmove_clock;
        self.fullmove_number = board_state.fullmove_number;
        self.num_repetitions = board_state.num_repetitions;
    }

    fn pop_nnue(&mut self) {
        self.evaluator
            .as_mut()
            .expect("No Evaluator found!")
            .restore();
    }

    pub fn pop(&mut self) -> Move {
        let (board_state, _move) = self.stack.pop().unwrap();
        self.repetition_table.remove(self.get_hash());
        self.restore(board_state);
        // self.pop_nnue();
        _move
    }

    pub fn pop_null_move(&mut self) {
        let board_state = self.stack.pop().unwrap().0;
        self.restore(board_state);
    }

    pub fn parse_san(&self, san: &str) -> Result<Move, ChessError> {
        return Move::from_san(&self.board, san.replace('0', "O").as_str());
    }

    pub fn push_san(&mut self, san: &str) -> Move {
        let _move = self.parse_san(san).expect("Bad san: {san}");
        self.push(_move);
        return _move;
    }

    pub fn push_sans(&mut self, sans: Vec<&str>) {
        for san in sans {
            self.push_san(san);
        }
    }

    pub fn parse_uci(&self, uci: &str) -> Result<Move, ChessError> {
        Move::from_str(uci)
    }

    pub fn push_uci(&mut self, uci: &str) -> Move {
        let _move = self.parse_uci(uci).expect("Bad uci: {uci}");
        self.push(_move);
        return _move;
    }

    pub fn push_str(&mut self, s: &str) {
        self.push_uci(s);
    }

    pub fn push_ucis(&mut self, ucis: Vec<&str>) {
        for uci in ucis {
            self.push_uci(uci);
        }
    }

    fn algebraic_without_suffix(&self, _move: Move, long: bool) -> String {
        // Null move.
        if _move.get_source() == _move.get_dest() {
            return "--".to_string();
        }

        // Castling.
        if self.is_castling(_move) {
            return (if _move.get_dest().get_file() < _move.get_source().get_file() {
                "O-O-O"
            } else {
                "O-O"
            })
            .to_string();
        }

        let piece = match self.piece_at(_move.get_source()) {
            Some(piece) => piece,
            None => panic!(
                "san() and lan() expect move to be legal or null, but got {_move} in {}",
                self.get_fen()
            ),
        };
        let capture = self.is_capture(_move);
        let mut san = if piece == Pawn {
            String::new()
        } else {
            piece.to_string(White)
        };

        if long {
            san += _move.get_source().to_string().as_str();
        } else if piece != Pawn {
            // Get ambiguous move candidates.
            // Relevant candidates: not exactly the current move,
            // but to the same square.
            let mut others = BB_EMPTY;
            let from_mask = self.get_piece_mask(piece)
                & self.occupied_co(self.turn())
                & !get_square_bb(_move.get_source());
            let to_mask = get_square_bb(_move.get_dest());
            for candidate in self
                .generate_masked_legal_moves(to_mask)
                .filter(|m| get_square_bb(m.get_source()) & from_mask != BB_EMPTY)
            {
                others |= get_square_bb(candidate.get_source());
            }

            // Disambiguate.
            if others != BB_EMPTY {
                let (mut row, mut column) = (false, false);
                if others & get_rank_bb(_move.get_source().get_rank()) != BB_EMPTY {
                    column = true;
                }
                if others & get_file_bb(_move.get_source().get_file()) != BB_EMPTY {
                    row = true;
                } else {
                    column = true;
                }
                if column {
                    san.push(
                        "abcdefgh"
                            .chars()
                            .nth(_move.get_source().get_file().to_index())
                            .unwrap(),
                    );
                }
                if row {
                    san += (_move.get_source().get_rank().to_index() + 1)
                        .to_string()
                        .as_str();
                }
            }
        } else if capture {
            san.push(
                "abcdefgh"
                    .chars()
                    .nth(_move.get_source().get_file().to_index())
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
        san += _move.get_dest().to_string().as_str();

        // Promotion.
        if let Some(promotion) = _move.get_promotion() {
            san += format!("={}", promotion.to_string(White)).as_str()
        }

        san
    }

    fn algebraic_and_push(&mut self, _move: Move, long: bool) -> String {
        let san = self.algebraic_without_suffix(_move, long);

        // Look ahead for check or checkmate.
        self.push(_move);
        let is_check = self.is_check();
        let is_checkmate = is_check && self.is_checkmate();

        // Add check or checkmate suffix.
        if is_checkmate && _move.get_source() != _move.get_dest() {
            san + "#"
        } else if is_check && _move.get_source() != _move.get_dest() {
            return san + "+";
        } else {
            return san;
        }
    }

    fn algebraic(&self, _move: Move, long: bool) -> String {
        self.clone().algebraic_and_push(_move, long)
    }

    pub fn san(&self, _move: Move) -> String {
        // Gets the standard algebraic notation of the given move in the context
        // of the current position.
        self.algebraic(_move, false)
    }

    pub fn san_and_push(&mut self, _move: Move) -> String {
        self.algebraic_and_push(_move, false)
    }

    pub fn lan(&self, _move: Move) -> String {
        // Gets the long algebraic notation of the given move in the context of
        // the current position.
        self.algebraic(_move, true)
    }

    pub fn lan_and_push(&mut self, _move: Move) -> String {
        self.algebraic_and_push(_move, true)
    }

    fn variation_san(&self, board: &Board, variation: Vec<Move>) -> String {
        // Given a sequence of moves, returns a string representing the sequence
        // in standard algebraic notation (e.g., ``1. e4 e5 2. Nf3 Nc6`` or
        // ``37...Bg6 38. fxg6``).

        // The board will not be modified as a result of calling this.

        // panics if any moves in the sequence are illegal.
        let mut board = board.clone();
        let mut san = Vec::new();
        for _move in variation {
            if !board.is_legal(_move) {
                panic!("illegal move {_move} in position {}", board.get_fen());
            }

            if board.turn() == White {
                let san_str = board.san_and_push(_move);
                san.push(format!("{}. {san_str}", board.fullmove_number));
            } else if san.is_empty() {
                let san_str = board.san_and_push(_move);
                san.push(format!("{}...{san_str}", board.fullmove_number));
            } else {
                san.push(board.san_and_push(_move).to_string());
            }
        }
        let mut san_string = String::new();
        for s in san {
            san_string += s.as_str();
            san_string += " ";
        }
        return san_string.trim().to_string();
    }

    pub fn get_pgn(&self) -> String {
        // Returns a string representing the game in Portable Game Notation (PGN).
        // The result of the game is included in the tags.
        self.variation_san(
            &Self::new(),
            Vec::from_iter(self.stack.clone().into_iter().map(|(_, m)| m)),
        )
    }

    pub fn is_legal(&self, _move: Move) -> bool {
        self.board.legal(_move)
    }

    pub fn generate_masked_legal_moves(&self, to_bitboard: BitBoard) -> chess::MoveGen {
        let mut moves = chess::MoveGen::new_legal(&self.board);
        moves.set_iterator_mask(to_bitboard);
        moves
    }

    pub fn generate_legal_moves(&self) -> chess::MoveGen {
        chess::MoveGen::new_legal(&self.board)
    }

    pub fn generate_legal_captures(&self) -> chess::MoveGen {
        let targets = self.occupied_co(!self.turn());
        self.generate_masked_legal_moves(*targets)
    }

    pub fn get_hash(&self) -> u64 {
        self.board.get_hash()
    }

    pub fn get_pawn_hash(&self) -> u64 {
        self.board.get_pawn_hash()
    }

    pub fn evaluate(&self) -> Score {
        self.evaluator
            .as_ref()
            .expect("No Evaluator found!")
            .evaluate(self)
    }

    pub fn evaluate_flipped(&self) -> Score {
        let score = self.evaluate();
        if self.turn() == White {
            score
        } else {
            -score
        }
    }
}

impl fmt::Display for Board {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_board_string(false))
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Self {
            board: self.board,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            stack: self.stack.clone(),
            evaluator: None,
            repetition_table: RepetitionTable::default(),
            num_repetitions: 1,
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
