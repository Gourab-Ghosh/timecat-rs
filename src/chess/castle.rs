use super::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CastleRights {
    None,
    KingSide,
    QueenSide,
    Both,
}

const CASTLES_PER_SQUARE: [[u8; 64]; 2] = [
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
    pub fn has_kingside(self) -> bool {
        self.to_index() & 1 == 1
    }

    /// Can I castle queenside?
    pub fn has_queenside(self) -> bool {
        self.to_index() & 2 == 2
    }

    pub fn square_to_castle_rights(color: Color, sq: Square) -> Self {
        Self::from_index(unsafe {
            *CASTLES_PER_SQUARE
                .get_unchecked(color.to_index())
                .get_unchecked(sq.to_index())
        } as usize)
    }

    /// What squares need to be empty to castle kingside?
    pub fn kingside_squares(self, color: Color) -> BitBoard {
        unsafe { *KINGSIDE_CASTLE_SQUARES.get_unchecked(color.to_index()) }
    }

    /// What squares need to be empty to castle queenside?
    pub fn queenside_squares(self, color: Color) -> BitBoard {
        unsafe { *QUEENSIDE_CASTLE_SQUARES.get_unchecked(color.to_index()) }
    }

    /// Remove castle rights, and return a new `CastleRights`.
    pub fn remove(self, remove: Self) -> Self {
        Self::from_index(self.to_index() & !remove.to_index())
    }

    /// Convert `CastleRights` to `usize` for table lookups
    pub fn to_index(self) -> usize {
        self as usize
    }

    /// Convert `usize` to `CastleRights`.  Panic if invalid number.
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::None,
            1 => Self::KingSide,
            2 => Self::QueenSide,
            3 => Self::Both,
            _ => unreachable!(),
        }
    }

    /// Which rooks can we "guarantee" we haven't moved yet?
    pub fn unmoved_rooks(self, color: Color) -> BitBoard {
        match self {
            Self::None => BB_EMPTY,
            Self::KingSide => BitBoard::from_rank_and_file(color.to_my_backrank(), File::H),
            Self::QueenSide => {
                BitBoard::from_rank_and_file(color.to_my_backrank(), File::A)
            }
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

        if color == Color::White {
            result.to_uppercase()
        } else {
            result.to_string()
        }
    }

    /// Given a square of a rook, which side is it on?
    /// Note: It is invalid to pass in a non-rook square.  The code may panic.
    pub fn rook_square_to_castle_rights(square: Square) -> Self {
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
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_index(self.to_index() | rhs.to_index())
    }
}

impl AddAssign for CastleRights {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for CastleRights {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        self.remove(rhs)
    }
}

impl SubAssign for CastleRights {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}