use super::*;

#[derive(Clone)]
pub struct SubBoardBuilder {
    pieces: [Option<Piece>; 64],
    turn: Color,
    castle_rights: [CastleRights; 2],
    ep_file: Option<File>,
    halfmove_clock: u8,
    fullmove_number: NumMoves,
}

impl SubBoardBuilder {
    /// Returns empty board builder with white to move
    pub fn new() -> Self {
        Self {
            pieces: [None; 64],
            turn: White,
            castle_rights: [CastleRights::None, CastleRights::None],
            ep_file: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn setup<'a>(
        pieces: impl IntoIterator<Item = &'a (Square, Piece)>,
        turn: Color,
        white_castle_rights: CastleRights,
        black_castle_rights: CastleRights,
        ep_file: Option<File>,
        halfmove_clock: u8,
        fullmove_number: u16,
    ) -> SubBoardBuilder {
        let mut result = SubBoardBuilder {
            pieces: [None; 64],
            turn,
            castle_rights: [white_castle_rights, black_castle_rights],
            ep_file,
            halfmove_clock,
            fullmove_number,
        };

        for piece in pieces {
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
        self.ep_file
            .map(|f| Square::from_rank_and_file((!self.get_turn()).to_third_rank(), f))
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

    pub fn add_piece(&mut self, square: Square, piece: Piece) -> &mut Self {
        self[square] = Some(piece);
        self
    }

    pub fn clear_square(&mut self, square: Square) -> &mut Self {
        self[square] = None;
        self
    }

    pub fn ep_file(&mut self, file: Option<File>) -> &mut Self {
        self.ep_file = file;
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

impl Index<Square> for SubBoardBuilder {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.pieces[index.to_index()]
    }
}

impl IndexMut<Square> for SubBoardBuilder {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.pieces[index.to_index()]
    }
}

impl fmt::Display for SubBoardBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut count = 0;
        for &rank in ALL_RANKS.iter().rev() {
            for file in ALL_FILES {
                let square = Square::from_rank_and_file(rank, file).to_index();

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

            if rank != Rank::First {
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
        if let Some(square) = self.get_en_passant() {
            write!(f, "{}", square)?;
        } else {
            write!(f, "-")?;
        }

        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)
    }
}

impl Default for SubBoardBuilder {
    fn default() -> SubBoardBuilder {
        SubBoardBuilder::from_str(STARTING_POSITION_FEN).unwrap()
    }
}

impl FromStr for SubBoardBuilder {
    type Err = TimecatError;

    fn from_str(value: &str) -> Result<Self> {
        let mut cur_rank = Rank::Eighth;
        let mut cur_file = File::A;
        let mut sub_board_builder = SubBoardBuilder::new();

        let tokens: Vec<&str> = value.split(' ').collect();
        if tokens.len() < 4 {
            return Err(TimecatError::BadFen {
                fen: value.to_string(),
            });
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castles = tokens[2];
        let ep = tokens[3];
        let halfmove_clock = tokens.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
        let fullmove_number = tokens.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);

        for x in pieces.chars() {
            match x {
                '/' => {
                    cur_rank = cur_rank.wrapping_down();
                    cur_file = File::A;
                }
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    cur_file =
                        File::from_index((cur_file.to_index() + (x as usize) - ('0' as usize)) & 7);
                }
                'r' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(BlackRook);
                    cur_file = cur_file.wrapping_right();
                }
                'R' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(WhiteRook);
                    cur_file = cur_file.wrapping_right();
                }
                'n' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(BlackKnight);
                    cur_file = cur_file.wrapping_right();
                }
                'N' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(WhiteKnight);
                    cur_file = cur_file.wrapping_right();
                }
                'b' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(BlackBishop);
                    cur_file = cur_file.wrapping_right();
                }
                'B' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(WhiteBishop);
                    cur_file = cur_file.wrapping_right();
                }
                'p' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(BlackPawn);
                    cur_file = cur_file.wrapping_right();
                }
                'P' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(WhitePawn);
                    cur_file = cur_file.wrapping_right();
                }
                'q' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(BlackQueen);
                    cur_file = cur_file.wrapping_right();
                }
                'Q' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(WhiteQueen);
                    cur_file = cur_file.wrapping_right();
                }
                'k' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(BlackKing);
                    cur_file = cur_file.wrapping_right();
                }
                'K' => {
                    sub_board_builder[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(WhiteKing);
                    cur_file = cur_file.wrapping_right();
                }
                _ => {
                    return Err(TimecatError::BadFen {
                        fen: value.to_string(),
                    });
                }
            }
        }
        match side {
            "w" | "W" => _ = sub_board_builder.turn(White),
            "b" | "B" => _ = sub_board_builder.turn(Black),
            _ => {
                return Err(TimecatError::BadFen {
                    fen: value.to_string(),
                })
            }
        }

        if castles.contains('K') && castles.contains('Q') {
            sub_board_builder.castle_rights[White.to_index()] = CastleRights::Both;
        } else if castles.contains('K') {
            sub_board_builder.castle_rights[White.to_index()] = CastleRights::KingSide;
        } else if castles.contains('Q') {
            sub_board_builder.castle_rights[White.to_index()] = CastleRights::QueenSide;
        } else {
            sub_board_builder.castle_rights[White.to_index()] = CastleRights::None;
        }

        if castles.contains('k') && castles.contains('q') {
            sub_board_builder.castle_rights[Black.to_index()] = CastleRights::Both;
        } else if castles.contains('k') {
            sub_board_builder.castle_rights[Black.to_index()] = CastleRights::KingSide;
        } else if castles.contains('q') {
            sub_board_builder.castle_rights[Black.to_index()] = CastleRights::QueenSide;
        } else {
            sub_board_builder.castle_rights[Black.to_index()] = CastleRights::None;
        }

        if let Ok(square) = Square::from_str(ep) {
            sub_board_builder.ep_file(Some(square.get_file()));
        }

        sub_board_builder
            .halfmove_clock(halfmove_clock)
            .fullmove_number(fullmove_number);

        Ok(sub_board_builder)
    }
}

impl From<&SubBoard> for SubBoardBuilder {
    fn from(board: &SubBoard) -> Self {
        let mut pieces = vec![];
        for square in ALL_SQUARES {
            if let Some(piece) = board.piece_at(square) {
                pieces.push((square, piece));
            }
        }

        SubBoardBuilder::setup(
            &pieces,
            board.turn(),
            board.castle_rights(White),
            board.castle_rights(Black),
            board.ep_square().map(|square| square.get_file()),
            board.get_halfmove_clock(),
            board.get_fullmove_number(),
        )
    }
}

impl From<SubBoard> for SubBoardBuilder {
    fn from(board: SubBoard) -> Self {
        (&board).into()
    }
}
