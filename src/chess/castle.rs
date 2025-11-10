use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CastleRights {
    None = 0,
    KingSide = 1,
    QueenSide = 2,
    Both = 3,
}

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
        *get_item_unchecked!(
            const {
                let mut array = [[Self::None; 64]; 2];
                array[0][63] = Self::KingSide;
                array[1][7] = Self::KingSide;
                array[0][56] = Self::QueenSide;
                array[1][0] = Self::QueenSide;
                array[0][60] = Self::Both;
                array[1][4] = Self::Both;
                array
            },
            color.to_index(),
            square.to_index()
        )
    }

    /// What squares need to be empty to castle kingside?
    #[inline]
    pub fn kingside_squares(self, color: Color) -> BitBoard {
        *get_item_unchecked!(
            const { [BitBoard::new(6917529027641081856), BitBoard::new(96)] },
            color.to_index(),
        )
    }

    /// What squares need to be empty to castle queenside?
    #[inline]
    pub fn queenside_squares(self, color: Color) -> BitBoard {
        *get_item_unchecked!(
            const { [BitBoard::new(1008806316530991104), BitBoard::new(14)] },
            color.to_index(),
        )
    }

    /// Remove castle rights, and return a new `CastleRights`.
    #[inline]
    pub fn remove(self, remove: Self) -> Self {
        Self::from_index(self.to_index() & !remove.to_index())
    }

    /// Convert `CastleRights` to `u8` for table lookups
    #[inline]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    /// Convert `CastleRights` to `usize` for table lookups
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Convert `usize` to `CastleRights`.  Panic if invalid number.
    #[inline]
    pub const fn from_int(i: u8) -> Self {
        unsafe { std::mem::transmute(i) }
    }

    /// Convert `usize` to `CastleRights`.  Panic if invalid number.
    #[inline]
    pub const fn from_index(i: usize) -> Self {
        Self::from_int(i as u8)
    }

    /// Which rooks can we "guarantee" we haven't moved yet?
    #[inline]
    pub fn unmoved_rooks(self, color: Color) -> BitBoard {
        // TODO: match can be removed.
        match self {
            Self::None => BitBoard::EMPTY,
            Self::KingSide => BitBoard::from_rank_and_file(color.to_my_backrank(), File::H),
            Self::QueenSide => BitBoard::from_rank_and_file(color.to_my_backrank(), File::A),
            Self::Both => {
                let my_backrank = color.to_my_backrank();
                BitBoard::from_rank_and_file(my_backrank, File::A)
                    ^ BitBoard::from_rank_and_file(my_backrank, File::H)
            }
        }
    }

    #[inline]
    pub fn to_string(self, color: Color) -> &'static str {
        get_item_unchecked!(
            const { ["", "k", "q", "kq", "", "K", "Q", "KQ"] },
            color.to_index() << 2 | self.to_index()
        )
    }
}

impl Add for CastleRights {
    type Output = Self;

    #[expect(clippy::suspicious_arithmetic_impl)]
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_index(self.to_index() | rhs.to_index())
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
