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

    pub fn from_san(mini_board: &MiniBoard, san: &str) -> Result<Self> {
        // TODO: Make the logic better
        let san = san.trim().replace('0', "O");
        for move_ in mini_board.generate_legal_moves() {
            if move_.san(mini_board).unwrap() == san {
                return Ok(move_);
            }
        }
        Err(TimecatError::InvalidSanMoveString { s: san.to_string() })
    }

    pub fn from_lan(mini_board: &MiniBoard, lan: &str) -> Result<Self> {
        // TODO: Make the logic better
        let lan = lan.trim().replace('0', "O");
        let lan = lan.replace('0', "O");
        for valid_or_null_move in mini_board.generate_legal_moves() {
            if valid_or_null_move.lan(mini_board).unwrap() == lan {
                return Ok(valid_or_null_move);
            }
        }
        Err(TimecatError::InvalidLanMoveString { s: lan.to_string() })
    }

    pub fn algebraic_without_suffix(self, mini_board: &MiniBoard, long: bool) -> Result<String> {
        let source = self.get_source();
        let dest = self.get_dest();

        // Castling.
        if mini_board.is_castling(self) {
            return if dest.get_file() < source.get_file() {
                Ok("O-O-O".to_string())
            } else {
                Ok("O-O".to_string())
            };
        }

        let piece =
            mini_board
                .get_piece_type_at(source)
                .ok_or(TimecatError::InvalidSanOrLanMove {
                    valid_or_null_move: self.into(),
                    fen: mini_board.get_fen(),
                })?;
        let capture = mini_board.is_capture(self);
        let mut san = if piece == Pawn {
            String::new()
        } else {
            piece.to_colored_piece_string(White)
        };

        if long {
            san += &source.to_string();
        } else if piece != Pawn {
            // Get ambiguous move candidates.
            // Relevant candidates: not exactly the current move,
            // but to the same square.
            let mut others = BB_EMPTY;
            let from_mask = mini_board.get_piece_mask(piece)
                & mini_board.self_occupied()
                & !source.to_bitboard();
            let to_mask = dest.to_bitboard();
            for candidate in mini_board.generate_masked_legal_moves(from_mask, to_mask) {
                others |= candidate.get_source().to_bitboard();
            }

            // Disambiguate.
            if !others.is_empty() {
                let (mut row, mut column) = (false, false);
                if !(others & source.get_rank_bb()).is_empty() {
                    column = true;
                }
                if !(others & source.get_file_bb()).is_empty() {
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
            san += &format!("={}", promotion.to_colored_piece_string(White))
        }

        Ok(san)
    }

    pub fn algebraic_and_new_mini_board(
        self,
        mini_board: &MiniBoard,
        long: bool,
    ) -> Result<(String, MiniBoard)> {
        let san = self.algebraic_without_suffix(mini_board, long)?;

        // Look ahead for check or checkmate.
        let new_mini_board = mini_board.make_move_new(self);
        let is_checkmate = new_mini_board.is_checkmate();

        // Add check or checkmate suffix.
        let san = if is_checkmate {
            san + "#"
        } else if new_mini_board.is_check() {
            san + "+"
        } else {
            san
        };
        Ok((san, new_mini_board))
    }

    #[cfg(feature = "pyo3")]
    fn from_py_move(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let source = ob.getattr("from_square")?.extract()?;
        let dest = ob.getattr("to_square")?.extract()?;
        let promotion = ob.getattr("promotion")?.extract()?;
        Ok(Self::new(source, dest, promotion)?)
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

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for Move {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(move_text) = ob.extract::<&str>() {
            if let Ok(move_) = Self::from_str(move_text) {
                return Ok(move_);
            }
        }
        if let Ok(move_) = Self::from_py_move(ob) {
            return Ok(move_);
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
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

    pub fn from_san(mini_board: &MiniBoard, san: &str) -> Result<Self> {
        // TODO: Make the logic better
        let san = san.trim();
        if san == "--" || san == "0000" {
            return Ok(ValidOrNullMove::NullMove);
        }
        Ok(Move::from_san(mini_board, san)?.into())
    }

    pub fn from_lan(mini_board: &MiniBoard, lan: &str) -> Result<Self> {
        // TODO: Make the logic better
        let lan = lan.trim();
        if lan == "--" || lan == "0000" {
            return Ok(ValidOrNullMove::NullMove);
        }
        Ok(Move::from_lan(mini_board, lan)?.into())
    }

    pub fn algebraic_without_suffix(self, mini_board: &MiniBoard, long: bool) -> Result<String> {
        self.map(|move_| move_.algebraic_without_suffix(mini_board, long))
            .unwrap_or(Ok("--".to_string()))
    }

    pub fn algebraic_and_new_mini_board(
        self,
        mini_board: &MiniBoard,
        long: bool,
    ) -> Result<(String, MiniBoard)> {
        self.map(|move_| move_.algebraic_and_new_mini_board(mini_board, long))
            .unwrap_or(Ok(("--".to_string(), mini_board.null_move()?)))
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

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for ValidOrNullMove {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(move_) = ob.extract::<Move>() {
            return Ok(move_.into());
        }
        if let Ok(move_text) = ob.extract::<&str>() {
            if let Ok(valid_or_null_move) = Self::from_str(move_text) {
                return Ok(valid_or_null_move);
            }
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CastleMoveType {
    KingSide,
    QueenSide,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl fmt::Display for WeightedMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.move_, self.weight)
    }
}
