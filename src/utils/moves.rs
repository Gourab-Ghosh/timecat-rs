use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Move {
    source: Square,
    dest: Square,
    promotion: Option<PieceType>,
}

impl Move {
    pub const NULL_MOVE: Self = Self {
        source: A1,
        dest: A1,
        promotion: None,
    };

    #[inline]
    pub fn new(source: Square, dest: Square, promotion: Option<PieceType>) -> Self {
        #[cfg(any(test, not(feature = "binary")))]
        assert_ne!(source, dest);
        Self {
            source,
            dest,
            promotion,
        }
    }

    pub fn is_null(&self) -> bool {
        *self == Self::NULL_MOVE
    }

    pub fn from_san(sub_board: &SubBoard, san: &str) -> Result<Option<Self>> {
        // TODO: Make the logic better
        let san = san.trim().replace('0', "O");
        if san == "--" {
            return Ok(None);
        }
        let san = san.replace('0', "O");
        for move_ in sub_board.generate_legal_moves() {
            if move_.san(sub_board).unwrap() == san {
                return Ok(Some(move_));
            }
        }
        Err(TimecatError::InvalidSanMoveString { s: san.to_string() })
    }

    pub fn from_lan(sub_board: &SubBoard, lan: &str) -> Result<Option<Self>> {
        // TODO: Make the logic better
        let lan = lan.trim().replace('0', "O");
        if lan == "--" {
            return Ok(None);
        }
        let lan = lan.replace('0', "O");
        for move_ in sub_board.generate_legal_moves() {
            if move_.lan(sub_board).unwrap() == lan {
                return Ok(Some(move_));
            }
        }
        Err(TimecatError::InvalidLanMoveString { s: lan.to_string() })
    }

    #[inline]
    pub const fn get_source(&self) -> Square {
        self.source
    }

    #[inline]
    pub const fn get_dest(&self) -> Square {
        self.dest
    }

    #[inline]
    pub const fn get_promotion(&self) -> Option<PieceType> {
        self.promotion
    }

    pub fn algebraic_without_suffix(self, sub_board: &SubBoard, long: bool) -> Result<String> {
        // Castling.
        if sub_board.is_castling(self) {
            return if self.get_dest().get_file() < self.get_source().get_file() {
                Ok("O-O-O".to_string())
            } else {
                Ok("O-O".to_string())
            };
        }

        let piece = sub_board.piece_type_at(self.get_source()).ok_or(
            TimecatError::InvalidSanOrLanMove {
                move_: self,
                fen: sub_board.get_fen(),
            },
        )?;
        let capture = sub_board.is_capture(self);
        let mut san = if piece == Pawn {
            String::new()
        } else {
            piece.to_string(White)
        };

        if long {
            san += &self.get_source().to_string();
        } else if piece != Pawn {
            // Get ambiguous move candidates.
            // Relevant candidates: not exactly the current move,
            // but to the same square.
            let mut others = BB_EMPTY;
            let from_mask = sub_board.get_piece_mask(piece)
                & sub_board.occupied_co(sub_board.turn())
                & !self.get_source().to_bitboard();
            let to_mask = self.get_dest().to_bitboard();
            for candidate in sub_board
                .generate_masked_legal_moves(to_mask)
                .filter(|m| !(m.get_source().to_bitboard() & from_mask).is_empty())
            {
                others |= candidate.get_source().to_bitboard();
            }

            // Disambiguate.
            if !others.is_empty() {
                let (mut row, mut column) = (false, false);
                if !(others & get_rank_bb(self.get_source().get_rank())).is_empty() {
                    column = true;
                }
                if !(others & get_file_bb(self.get_source().get_file())).is_empty() {
                    row = true;
                } else {
                    column = true;
                }
                if column {
                    san.push(
                        "abcdefgh"
                            .chars()
                            .nth(self.get_source().get_file().to_index())
                            .unwrap(),
                    );
                }
                if row {
                    san += &(self.get_source().get_rank().to_index() + 1).to_string();
                }
            }
        } else if capture {
            san.push(
                "abcdefgh"
                    .chars()
                    .nth(self.get_source().get_file().to_index())
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
        san += &self.get_dest().to_string();

        // Promotion.
        if let Some(promotion) = self.get_promotion() {
            san += &format!("={}", promotion.to_string(White))
        }

        Ok(san)
    }

    pub fn algebraic_and_new_sub_board(
        self,
        sub_board: &SubBoard,
        long: bool,
    ) -> Result<(String, SubBoard)> {
        let san = self.algebraic_without_suffix(sub_board, long)?;

        // Look ahead for check or checkmate.
        let new_sub_board = sub_board.make_move_new(self);
        let is_checkmate = new_sub_board.is_checkmate();

        // Add check or checkmate suffix.
        let san = if is_checkmate {
            san + "#"
        } else if new_sub_board.is_check() {
            san + "+"
        } else {
            san
        };
        Ok((san, new_sub_board))
    }
}

impl Default for Move {
    fn default() -> Self {
        Self::NULL_MOVE
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.promotion {
            Some(piece) => write!(f, "{}{}{}", self.source, self.dest, piece),
            None => write!(f, "{}{}", self.source, self.dest),
        }
    }
}

impl FromStr for Move {
    type Err = TimecatError;

    fn from_str(mut s: &str) -> Result<Self> {
        let error = TimecatError::InvalidUciMoveString { s: s.to_string() };
        s = s.trim();
        if s.len() > 6 {
            return Err(error.clone());
        }
        let source = Square::from_str(s.get(0..2).ok_or(error.clone())?)?;
        let dest = Square::from_str(s.get(2..4).ok_or(error.clone())?)?;

        let mut promotion = None;
        if s.len() == 5 {
            promotion = Some(match s.chars().last().ok_or(error.clone())? {
                'q' => Queen,
                'r' => Rook,
                'n' => Knight,
                'b' => Bishop,
                _ => return Err(error.clone()),
            });
        }

        Ok(Self::new(source, dest, promotion))
    }
}

pub enum CastleMoveType {
    KingSide,
    QueenSide,
}

pub enum MoveType {
    Capture { is_en_passant: bool },
    Castle(CastleMoveType),
    DoublePawnPush,
    Other,
}

pub struct MoveWithInfo {
    move_: Move,
    type_: MoveType,
    is_check: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct WeightedMove {
    pub weight: MoveWeight,
    pub move_: Move,
}

impl PartialOrd for WeightedMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeightedMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl WeightedMove {
    pub fn new(move_: Move, weight: MoveWeight) -> Self {
        Self { move_, weight }
    }
}
