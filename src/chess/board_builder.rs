use super::*;

#[derive(Clone)]
pub struct BoardBuilder {
    pieces: [Option<Piece>; 64],
    turn: Color,
    castle_rights: [CastleRights; 2],
    en_passant: Option<File>,
    halfmove_clock: u8,
    fullmove_number: NumMoves,
}

impl BoardBuilder {
    /// Returns empty board builder with white to move
    pub fn new() -> Self {
        Self {
            pieces: [None; 64],
            turn: White,
            castle_rights: [CastleRights::None, CastleRights::None],
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn setup<'a>(
        pieces: impl IntoIterator<Item = &'a (Square, Piece)>,
        turn: Color,
        white_castle_rights: CastleRights,
        black_castle_rights: CastleRights,
        en_passant: Option<File>,
        halfmove_clock: u8,
        fullmove_number: u16,
    ) -> BoardBuilder {
        let mut result = BoardBuilder {
            pieces: [None; 64],
            turn,
            castle_rights: [white_castle_rights, black_castle_rights],
            en_passant,
            halfmove_clock,
            fullmove_number,
        };

        for piece in pieces.into_iter() {
            result.pieces[piece.0.to_index()] = Some(piece.1);
        }

        result
    }

    pub fn get_turn(&self) -> Color {
        self.turn
    }

    pub fn get_castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color.to_index()]
    }

    pub fn get_en_passant(&self) -> Option<Square> {
        self.en_passant
            .map(|f| Square::from_rank_and_file((!self.get_turn()).to_fourth_rank(), f))
    }

    #[inline]
    pub fn get_halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    #[inline]
    pub fn get_fullmove_number(&self) -> NumMoves {
        self.fullmove_number
    }

    pub fn turn(&mut self, color: Color) -> &mut Self {
        self.turn = color;
        self
    }

    pub fn castle_rights(&mut self, color: Color, castle_rights: CastleRights) -> &mut Self {
        self.castle_rights[color.to_index()] = castle_rights;
        self
    }

    pub fn piece(&mut self, square: Square, piece: Piece) -> &mut Self {
        self[square] = Some(piece);
        self
    }

    pub fn clear_square(&mut self, square: Square) -> &mut Self {
        self[square] = None;
        self
    }

    pub fn en_passant(&mut self, file: Option<File>) -> &mut Self {
        self.en_passant = file;
        self
    }

    pub fn halfmove_clock(&mut self, halfmove_clock: u8) -> &mut Self {
        self.halfmove_clock = halfmove_clock;
        self
    }

    pub fn fullmove_number(&mut self, fullmove_number: NumMoves) -> &mut Self {
        self.fullmove_number = fullmove_number;
        self
    }
}

impl Index<Square> for BoardBuilder {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.pieces[index.to_index()]
    }
}

impl IndexMut<Square> for BoardBuilder {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.pieces[index.to_index()]
    }
}

impl fmt::Display for BoardBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut count = 0;
        for rank in ALL_RANKS.iter().rev() {
            for file in ALL_FILES.iter() {
                let square = Square::from_rank_and_file(*rank, *file).to_index();

                if self.pieces[square].is_some() && count != 0 {
                    write!(f, "{}", count)?;
                    count = 0;
                }

                if let Some(piece) = self.pieces[square] {
                    write!(f, "{piece}")?;
                } else {
                    count += 1;
                }
            }

            if count != 0 {
                write!(f, "{}", count)?;
            }

            if *rank != Rank::First {
                write!(f, "/")?;
            }
            count = 0;
        }

        write!(f, " ")?;

        if self.turn == White {
            write!(f, "w ")?;
        } else {
            write!(f, "b ")?;
        }

        write!(
            f,
            "{}",
            self.castle_rights[White.to_index()].to_string(White)
        )?;
        write!(
            f,
            "{}",
            self.castle_rights[Black.to_index()].to_string(Black)
        )?;
        if self.castle_rights[0] == CastleRights::None
            && self.castle_rights[1] == CastleRights::None
        {
            write!(f, "-")?;
        }

        write!(f, " ")?;
        if let Some(sq) = self.get_en_passant() {
            write!(f, "{}", sq.wrapping_forward(self.turn))?;
        } else {
            write!(f, "-")?;
        }

        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)
    }
}

impl Default for BoardBuilder {
    fn default() -> BoardBuilder {
        BoardBuilder::from_str(STARTING_POSITION_FEN).unwrap()
    }
}

impl FromStr for BoardBuilder {
    type Err = EngineError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut cur_rank = Rank::Eighth;
        let mut cur_file = File::A;
        let mut board_builder = BoardBuilder::new();

        let tokens: Vec<&str> = value.split(' ').collect();
        if tokens.len() < 4 {
            return Err(EngineError::BadFen {
                fen: value.to_string(),
            });
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let ep = tokens[3];
        let halfmove_clock = tokens.get(4).map(|s| s.parse().ok()).flatten().unwrap_or(0);
        let fullmove_number = tokens.get(5).map(|s| s.parse().ok()).flatten().unwrap_or(1);

        for x in pieces.chars() {
            match x {
                '/' => {
                    cur_rank = cur_rank.down();
                    cur_file = File::A;
                }
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    cur_file =
                        File::from_index(cur_file.to_index() + (x as usize) - ('0' as usize));
                }
                'r' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Rook, Black));
                    cur_file = cur_file.right();
                }
                'R' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Rook, White));
                    cur_file = cur_file.right();
                }
                'n' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Knight, Black));
                    cur_file = cur_file.right();
                }
                'N' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Knight, White));
                    cur_file = cur_file.right();
                }
                'b' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Bishop, Black));
                    cur_file = cur_file.right();
                }
                'B' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Bishop, White));
                    cur_file = cur_file.right();
                }
                'p' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Pawn, Black));
                    cur_file = cur_file.right();
                }
                'P' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Pawn, White));
                    cur_file = cur_file.right();
                }
                'q' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Queen, Black));
                    cur_file = cur_file.right();
                }
                'Q' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(Queen, White));
                    cur_file = cur_file.right();
                }
                'k' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(King, Black));
                    cur_file = cur_file.right();
                }
                'K' => {
                    board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(King, White));
                    cur_file = cur_file.right();
                }
                _ => {
                    return Err(EngineError::BadFen {
                        fen: value.to_string(),
                    });
                }
            }
        }
        match side {
            "w" | "W" => _ = board_builder.turn(White),
            "b" | "B" => _ = board_builder.turn(Black),
            _ => {
                return Err(EngineError::BadFen {
                    fen: value.to_string(),
                })
            }
        }

        if castles.contains('K') && castles.contains('Q') {
            board_builder.castle_rights[White.to_index()] = CastleRights::Both;
        } else if castles.contains('K') {
            board_builder.castle_rights[White.to_index()] = CastleRights::KingSide;
        } else if castles.contains('Q') {
            board_builder.castle_rights[White.to_index()] = CastleRights::QueenSide;
        } else {
            board_builder.castle_rights[White.to_index()] = CastleRights::None;
        }

        if castles.contains('k') && castles.contains('q') {
            board_builder.castle_rights[Black.to_index()] = CastleRights::Both;
        } else if castles.contains('k') {
            board_builder.castle_rights[Black.to_index()] = CastleRights::KingSide;
        } else if castles.contains('q') {
            board_builder.castle_rights[Black.to_index()] = CastleRights::QueenSide;
        } else {
            board_builder.castle_rights[Black.to_index()] = CastleRights::None;
        }

        if let Ok(sq) = Square::from_str(ep) {
            board_builder.en_passant(Some(sq.get_file()));
        }

        board_builder
            .halfmove_clock(halfmove_clock)
            .fullmove_number(fullmove_number);

        Ok(board_builder)
    }
}

impl From<&SubBoard> for BoardBuilder {
    fn from(board: &SubBoard) -> Self {
        let mut pieces = vec![];
        for sq in ALL_SQUARES.iter() {
            if let Some(piece) = board.piece_type_at(*sq) {
                let color = board.color_at(*sq).unwrap();
                pieces.push((*sq, Piece::new(piece, color)));
            }
        }

        BoardBuilder::setup(
            &pieces,
            board.turn(),
            board.castle_rights(White),
            board.castle_rights(Black),
            board.en_passant().map(|sq| sq.get_file()),
            board.get_halfmove_clock(),
            board.get_fullmove_number(),
        )
    }
}

impl From<SubBoard> for BoardBuilder {
    fn from(board: SubBoard) -> Self {
        (&board).into()
    }
}
