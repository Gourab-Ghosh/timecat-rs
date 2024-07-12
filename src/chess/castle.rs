use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CastleRights {
    None,
    KingSide,
    QueenSide,
    Both,
}

const CASTLES_PER_SQUARE: [[usize; 64]; 2] = [
    [
        2, 0, 0, 0, 3, 0, 0, 1, // 1
        0, 0, 0, 0, 0, 0, 0, 0, // 2
        0, 0, 0, 0, 0, 0, 0, 0, // 3
        0, 0, 0, 0, 0, 0, 0, 0, // 4
        0, 0, 0, 0, 0, 0, 0, 0, // 5
        0, 0, 0, 0, 0, 0, 0, 0, // 6
        0, 0, 0, 0, 0, 0, 0, 0, // 7
        0, 0, 0, 0, 0, 0, 0, 0, // 8
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, // 1
        0, 0, 0, 0, 0, 0, 0, 0, // 2
        0, 0, 0, 0, 0, 0, 0, 0, // 3
        0, 0, 0, 0, 0, 0, 0, 0, // 4
        0, 0, 0, 0, 0, 0, 0, 0, // 5
        0, 0, 0, 0, 0, 0, 0, 0, // 6
        0, 0, 0, 0, 0, 0, 0, 0, // 7
        2, 0, 0, 0, 3, 0, 0, 1, // 8
    ],
];

impl CastleRights {
    /// Can I castle kingside?
    #[inline]
    pub const fn has_kingside(self) -> bool {
        self.to_index() & 1 == 1
    }

    /// Can I castle queenside?
    #[inline]
    pub const fn has_queenside(self) -> bool {
        self.to_index() & 2 == 2
    }

    #[inline]
    pub fn square_to_castle_rights(color: Color, square: Square) -> Self {
        Self::from_index(*get_item_unchecked!(
            CASTLES_PER_SQUARE,
            color.to_index(),
            square.to_index()
        ))
    }

    /// What squares need to be empty to castle kingside?
    #[inline]
    pub fn kingside_squares(self, color: Color) -> BitBoard {
        *get_item_unchecked!(KINGSIDE_CASTLE_SQUARES, color.to_index())
    }

    /// What squares need to be empty to castle queenside?
    #[inline]
    pub fn queenside_squares(self, color: Color) -> BitBoard {
        *get_item_unchecked!(QUEENSIDE_CASTLE_SQUARES, color.to_index())
    }

    /// Remove castle rights, and return a new `CastleRights`.
    #[inline]
    pub const fn remove(self, remove: Self) -> Self {
        match (self, remove) {
            (lhs, Self::None) => lhs,
            (_, Self::Both)
            | (Self::None, _)
            | (Self::KingSide, Self::KingSide)
            | (Self::QueenSide, Self::QueenSide) => Self::None,
            (Self::Both, Self::QueenSide) | (Self::KingSide, Self::QueenSide) => Self::KingSide,
            (Self::Both, Self::KingSide) | (Self::QueenSide, Self::KingSide) => Self::QueenSide,
        }
    }

    /// Convert `CastleRights` to `usize` for table lookups
    #[inline]
    pub const fn to_index(self) -> usize {
        match self {
            Self::None => 0,
            Self::KingSide => 1,
            Self::QueenSide => 2,
            Self::Both => 3,
        }
    }

    /// Convert `usize` to `CastleRights`.  Panic if invalid number.
    #[inline]
    pub const fn from_index(i: usize) -> Self {
        // TODO: Write a proper panic message.
        match i {
            0 => Self::None,
            1 => Self::KingSide,
            2 => Self::QueenSide,
            3 => Self::Both,
            _ => unreachable!(),
        }
    }

    /// Which rooks can we "guarantee" we haven't moved yet?
    #[inline]
    pub fn unmoved_rooks(self, color: Color) -> BitBoard {
        match self {
            Self::None => BB_EMPTY,
            Self::KingSide => BitBoard::from_rank_and_file(color.to_my_backrank(), File::H),
            Self::QueenSide => BitBoard::from_rank_and_file(color.to_my_backrank(), File::A),
            Self::Both => {
                BitBoard::from_rank_and_file(color.to_my_backrank(), File::A)
                    ^ BitBoard::from_rank_and_file(color.to_my_backrank(), File::H)
            }
        }
    }

    pub fn to_string(self, color: Color) -> String {
        let result = match self {
            Self::None => "",
            Self::KingSide => "k",
            Self::QueenSide => "q",
            Self::Both => "kq",
        };

        if color == White {
            result.to_uppercase()
        } else {
            result.to_string()
        }
    }

    /// Given a square of a rook, which side is it on?
    /// Note: It is invalid to pass in a non-rook square.  The code may panic.
    #[inline]
    pub fn rook_square_to_castle_rights(square: Square) -> Self {
        // TODO: Write a proper panic message.
        match square.get_file() {
            File::A => Self::QueenSide,
            File::H => Self::KingSide,
            _ => unreachable!(),
        }
    }
}

impl Add for CastleRights {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Both, _)
            | (_, Self::Both)
            | (Self::KingSide, Self::QueenSide)
            | (Self::QueenSide, Self::KingSide) => Self::Both,
            (Self::None, rhs) => rhs,
            (lhs, Self::None) => lhs,
            (Self::KingSide, Self::KingSide) => Self::KingSide,
            (Self::QueenSide, Self::QueenSide) => Self::QueenSide,
        }
    }
}

impl AddAssign for CastleRights {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for CastleRights {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.remove(rhs)
    }
}

impl SubAssign for CastleRights {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
