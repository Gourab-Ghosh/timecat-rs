use super::*;

pub enum CastleMoveType {
    KingSide,
    QueenSide,
}

pub enum MoveType {
    Capture,
    Castle(CastleMoveType),
    DoublePawnPush,
    Other,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct Move {
    source: Square,
    dest: Square,
    promotion: Option<PieceType>,
}

impl Move {
    #[inline(always)]
    pub fn new(source: Square, dest: Square, promotion: Option<PieceType>) -> Self {
        Self {
            source,
            dest,
            promotion,
        }
    }

    #[inline(always)]
    pub fn get_source(&self) -> Square {
        self.source
    }

    #[inline(always)]
    pub fn get_dest(&self) -> Square {
        self.dest
    }

    #[inline(always)]
    pub fn get_promotion(&self) -> Option<PieceType> {
        self.promotion
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.promotion {
            None => write!(f, "{}{}", self.source, self.dest),
            Some(x) => write!(f, "{}{}{}", self.source, self.dest, x),
        }
    }
}

impl FromStr for Move {
    type Err = ChessError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = Square::from_str(s.get(0..2).ok_or(ChessError::InvalidUciMove)?)?;
        let dest = Square::from_str(s.get(2..4).ok_or(ChessError::InvalidUciMove)?)?;

        let mut promo = None;
        if s.len() == 5 {
            promo = Some(match s.chars().last().ok_or(ChessError::InvalidUciMove)? {
                'q' => PieceType::Queen,
                'r' => PieceType::Rook,
                'n' => PieceType::Knight,
                'b' => PieceType::Bishop,
                _ => return Err(ChessError::InvalidUciMove),
            });
        }

        Ok(Self::new(source, dest, promo))
    }
}

pub struct MoveWithInfo {
    move_: Move,
    type_: MoveType,
    is_check: bool,
}