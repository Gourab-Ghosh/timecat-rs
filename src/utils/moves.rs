use super::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Move {
    source: Square,
    dest: Square,
    promotion: Option<PieceType>,
}

impl Move {
    #[inline(always)]
    pub const fn new(source: Square, dest: Square, promotion: Option<PieceType>) -> Self {
        Self {
            source,
            dest,
            promotion,
        }
    }

    #[inline(always)]
    pub const fn get_source(&self) -> Square {
        self.source
    }

    #[inline(always)]
    pub const fn get_dest(&self) -> Square {
        self.dest
    }

    #[inline(always)]
    pub const fn get_promotion(&self) -> Option<PieceType> {
        self.promotion
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
    type Err = EngineError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let error = EngineError::InvalidUciMoveString { s: s.to_string() };
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
