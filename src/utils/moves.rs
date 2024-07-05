use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Move {
    source: Square,
    dest: Square,
    promotion: Option<PieceType>,
}

impl Move {
    #[inline]
    pub const fn new_unchecked(source: Square, dest: Square, promotion: Option<PieceType>) -> Self {
        Self {
            source,
            dest,
            promotion,
        }
    }

    #[inline]
    pub const fn new(source: Square, dest: Square, promotion: Option<PieceType>) -> Result<Self> {
        if source.to_int() == dest.to_int() {
            return Err(TimecatError::InvalidMoveStructGeneration);
        }
        Ok(Self::new_unchecked(source, dest, promotion))
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

    pub fn from_san(sub_board: &SubBoard, san: &str) -> Result<Self> {
        // TODO: Make the logic better
        let san = san.trim().replace('0', "O");
        for move_ in sub_board.generate_legal_moves() {
            if move_.san(sub_board).unwrap() == san {
                return Ok(move_);
            }
        }
        Err(TimecatError::InvalidSanMoveString { s: san.to_string() })
    }

    pub fn from_lan(sub_board: &SubBoard, lan: &str) -> Result<Self> {
        // TODO: Make the logic better
        let lan = lan.trim().replace('0', "O");
        let lan = lan.replace('0', "O");
        for valid_or_null_move in sub_board.generate_legal_moves() {
            if valid_or_null_move.lan(sub_board).unwrap() == lan {
                return Ok(valid_or_null_move);
            }
        }
        Err(TimecatError::InvalidLanMoveString { s: lan.to_string() })
    }

    pub fn algebraic_without_suffix(self, sub_board: &SubBoard, long: bool) -> Result<String> {
        let source = self.get_source();
        let dest = self.get_dest();

        // Castling.
        if sub_board.is_castling(self) {
            return if dest.get_file() < source.get_file() {
                Ok("O-O-O".to_string())
            } else {
                Ok("O-O".to_string())
            };
        }

        let piece = sub_board
            .piece_type_at(source)
            .ok_or(TimecatError::InvalidSanOrLanMove {
                valid_or_null_move: self.into(),
                fen: sub_board.get_fen(),
            })?;
        let capture = sub_board.is_capture(self);
        let mut san = if piece == Pawn {
            String::new()
        } else {
            piece.to_string(White)
        };

        if long {
            san += &source.to_string();
        } else if piece != Pawn {
            // Get ambiguous move candidates.
            // Relevant candidates: not exactly the current move,
            // but to the same square.
            let mut others = BB_EMPTY;
            let from_mask = sub_board.get_piece_mask(piece)
                & sub_board.occupied_co(sub_board.turn())
                & !source.to_bitboard();
            let to_mask = dest.to_bitboard();
            for candidate in sub_board.generate_masked_legal_moves(from_mask, to_mask) {
                others |= candidate.get_source().to_bitboard();
            }

            // Disambiguate.
            if !others.is_empty() {
                let (mut row, mut column) = (false, false);
                if !(others & get_rank_bb(source.get_rank())).is_empty() {
                    column = true;
                }
                if !(others & get_file_bb(source.get_file())).is_empty() {
                    row = true;
                } else {
                    column = true;
                }
                if column {
                    san.push(
                        "abcdefgh"
                            .chars()
                            .nth(source.get_file().to_index())
                            .unwrap(),
                    );
                }
                if row {
                    san += &(source.get_rank().to_index() + 1).to_string();
                }
            }
        } else if capture {
            san.push(
                "abcdefgh"
                    .chars()
                    .nth(source.get_file().to_index())
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
        san += &dest.to_string();

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

        Self::new(source, dest, promotion)
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[repr(transparent)]
pub struct ValidOrNullMove(Option<Move>);

impl ValidOrNullMove {
    #[allow(non_upper_case_globals)]
    pub const NullMove: Self = Self(None);

    #[inline]
    pub const fn new_unchecked(source: Square, dest: Square, promotion: Option<PieceType>) -> Self {
        Self(Some(Move::new_unchecked(source, dest, promotion)))
    }

    #[inline]
    pub fn new(source: Square, dest: Square, promotion: Option<PieceType>) -> Result<Self> {
        Ok(Self(Some(Move::new(source, dest, promotion)?)))
    }

    #[inline]
    pub fn into_inner(&self) -> Option<&Move> {
        self.0.as_ref()
    }

    #[inline]
    pub fn into_inner_mut(&mut self) -> Option<&mut Move> {
        self.0.as_mut()
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.into_inner().is_none()
    }

    #[inline]
    pub fn get_source(&self) -> Option<Square> {
        self.into_inner().map(|move_| move_.source)
    }

    #[inline]
    pub fn get_dest(&self) -> Option<Square> {
        self.into_inner().map(|move_| move_.dest)
    }

    #[inline]
    pub fn get_promotion(&self) -> Option<PieceType> {
        self.into_inner()?.promotion
    }

    pub fn from_san(sub_board: &SubBoard, san: &str) -> Result<Self> {
        // TODO: Make the logic better
        let san = san.trim();
        if san == "--" || san == "0000" {
            return Ok(ValidOrNullMove::NullMove);
        }
        Ok(Move::from_san(sub_board, san)?.into())
    }

    pub fn from_lan(sub_board: &SubBoard, lan: &str) -> Result<Self> {
        // TODO: Make the logic better
        let lan = lan.trim();
        if lan == "--" || lan == "0000" {
            return Ok(ValidOrNullMove::NullMove);
        }
        Ok(Move::from_lan(sub_board, lan)?.into())
    }

    pub fn algebraic_without_suffix(self, sub_board: &SubBoard, long: bool) -> Result<String> {
        self.map(|move_| move_.algebraic_without_suffix(sub_board, long))
            .unwrap_or(Ok("--".to_string()))
    }

    pub fn algebraic_and_new_sub_board(
        self,
        sub_board: &SubBoard,
        long: bool,
    ) -> Result<(String, SubBoard)> {
        self.map(|move_| move_.algebraic_and_new_sub_board(sub_board, long))
            .unwrap_or(Ok(("--".to_string(), sub_board.null_move()?)))
    }
}

#[allow(non_upper_case_globals)]
pub const NullMove: ValidOrNullMove = ValidOrNullMove::NullMove;

impl Default for ValidOrNullMove {
    fn default() -> Self {
        Self::NullMove
    }
}

impl fmt::Display for ValidOrNullMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(move_) = self.into_inner() {
            write!(f, "{}", move_)
        } else {
            write!(f, "--")
        }
    }
}

impl FromStr for ValidOrNullMove {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        if s == "--" || s == "0000" {
            return Ok(Self::NullMove);
        }
        Ok(Move::from_str(s)?.into())
    }
}

impl From<Move> for ValidOrNullMove {
    #[inline]
    fn from(value: Move) -> Self {
        Some(value).into()
    }
}

impl From<Option<Move>> for ValidOrNullMove {
    #[inline]
    fn from(value: Option<Move>) -> Self {
        Self(value)
    }
}

impl Deref for ValidOrNullMove {
    type Target = Option<Move>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ValidOrNullMove {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CastleMoveType {
    KingSide,
    QueenSide,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub enum MoveType {
    Capture {
        is_en_passant: bool,
    },
    Castle(CastleMoveType),
    DoublePawnPush,
    Promotion(PieceType),
    #[default]
    Other,
}

pub struct MoveWithInfo {
    valid_or_null_move: ValidOrNullMove,
    type_: MoveType,
    is_check: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
