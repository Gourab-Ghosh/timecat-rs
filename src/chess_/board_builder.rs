use super::*;

#[derive(Clone, Copy)]
pub struct BoardBuilder {
    pieces: [Option<Piece>; 64],
    side_to_move: Color,
    castle_rights: [CastleRights; 2],
    en_passant: Option<File>,
    halfmove_number: u8,
    fullmove_count: NumMoves,
}

impl BoardBuilder {
    /// Returns empty board builder with white to move
    pub fn new() -> Self {
        Self {
            pieces: [None; 64],
            side_to_move: Color::White,
            castle_rights: [CastleRights::None, CastleRights::None],
            en_passant: None,
            halfmove_number: 0,
            fullmove_count: 1,
        }
    }

    // pub fn setup<'a>(
    //     pieces: impl IntoIterator<Item = &'a (Square, Piece, Color)>,
    //     side_to_move: Color,
    //     white_castle_rights: CastleRights,
    //     black_castle_rights: CastleRights,
    //     en_passant: Option<File>,
    // ) -> BoardBuilder {
    //     let mut result = BoardBuilder {
    //         pieces: [None; 64],
    //         side_to_move: side_to_move,
    //         castle_rights: [white_castle_rights, black_castle_rights],
    //         en_passant: en_passant,
    //     };

    //     for piece in pieces.into_iter() {
    //         result.pieces[piece.get_mask().to_index()] = Some((piece.1, piece.2));
    //     }

    //     result
    // }

    pub fn get_side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn get_castle_rights(&self, color: Color) -> CastleRights {
        self.castle_rights[color.to_index()]
    }

    pub fn get_en_passant(&self) -> Option<Square> {
        self.en_passant
            .map(|f| Square::from_rank_and_file((!self.get_side_to_move()).to_fourth_rank(), f))
    }

    pub fn side_to_move(&mut self, color: Color) -> &mut Self {
        self.side_to_move = color;
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

        if self.side_to_move == Color::White {
            write!(f, "w ")?;
        } else {
            write!(f, "b ")?;
        }

        write!(
            f,
            "{}",
            self.castle_rights[Color::White.to_index()].to_string(Color::White)
        )?;
        write!(
            f,
            "{}",
            self.castle_rights[Color::Black.to_index()].to_string(Color::Black)
        )?;
        if self.castle_rights[0] == CastleRights::None
            && self.castle_rights[1] == CastleRights::None
        {
            write!(f, "-")?;
        }

        write!(f, " ")?;
        if let Some(sq) = self.get_en_passant() {
            write!(f, "{}", sq)?;
        } else {
            write!(f, "-")?;
        }

        write!(f, " 0 1")
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
        let mut fen = BoardBuilder::new();

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
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Rook, Color::Black));
                    cur_file = cur_file.right();
                }
                'R' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Rook, Color::White));
                    cur_file = cur_file.right();
                }
                'n' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Knight, Color::Black));
                    cur_file = cur_file.right();
                }
                'N' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Knight, Color::White));
                    cur_file = cur_file.right();
                }
                'b' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Bishop, Color::Black));
                    cur_file = cur_file.right();
                }
                'B' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Bishop, Color::White));
                    cur_file = cur_file.right();
                }
                'p' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Pawn, Color::Black));
                    cur_file = cur_file.right();
                }
                'P' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Pawn, Color::White));
                    cur_file = cur_file.right();
                }
                'q' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Queen, Color::Black));
                    cur_file = cur_file.right();
                }
                'Q' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::Queen, Color::White));
                    cur_file = cur_file.right();
                }
                'k' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::King, Color::Black));
                    cur_file = cur_file.right();
                }
                'K' => {
                    fen[Square::from_rank_and_file(cur_rank, cur_file)] =
                        Some(Piece::new(PieceType::King, Color::White));
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
            "w" | "W" => _ = fen.side_to_move(Color::White),
            "b" | "B" => _ = fen.side_to_move(Color::Black),
            _ => {
                return Err(EngineError::BadFen {
                    fen: value.to_string(),
                })
            }
        }

        if castles.contains('K') && castles.contains('Q') {
            fen.castle_rights[Color::White.to_index()] = CastleRights::Both;
        } else if castles.contains('K') {
            fen.castle_rights[Color::White.to_index()] = CastleRights::KingSide;
        } else if castles.contains('Q') {
            fen.castle_rights[Color::White.to_index()] = CastleRights::QueenSide;
        } else {
            fen.castle_rights[Color::White.to_index()] = CastleRights::None;
        }

        if castles.contains('k') && castles.contains('q') {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::Both;
        } else if castles.contains('k') {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::KingSide;
        } else if castles.contains('q') {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::QueenSide;
        } else {
            fen.castle_rights[Color::Black.to_index()] = CastleRights::None;
        }

        if let Ok(sq) = Square::from_str(ep) {
            fen.en_passant(Some(sq.get_file()));
        }

        Ok(fen)
    }
}
