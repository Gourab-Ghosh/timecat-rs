use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Rank {
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
    Sixth = 5,
    Seventh = 6,
    Eighth = 7,
}

impl Rank {
    #[inline]
    pub const unsafe fn from_int(i: u8) -> Self {
        std::mem::transmute(i)
    }

    #[inline]
    pub const unsafe fn from_index(i: usize) -> Self {
        Self::from_int(i as u8)
    }

    #[inline]
    pub fn up(self) -> Option<Self> {
        (self != Self::Eighth).then(|| unsafe { Self::from_int(self.to_int() + 1) })
    }

    #[inline]
    pub fn down(self) -> Option<Self> {
        (self != Self::First).then(|| unsafe { Self::from_int(self.to_int() - 1) })
    }

    #[inline]
    pub fn wrapping_up(self) -> Self {
        self.up().unwrap_or(Self::First)
    }

    #[inline]
    pub fn wrapping_down(self) -> Self {
        self.down().unwrap_or(Self::Eighth)
    }

    #[inline]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard::new(0xff << (self.to_int() << 3))
    }

    #[inline]
    pub fn get_upper_board_mask(self, color: Color) -> BitBoard {
        *get_item_unchecked!(UPPER_BOARD_MASK, color.to_index(), self.to_index())
    }

    #[inline]
    pub fn get_lower_board_mask(self, color: Color) -> BitBoard {
        self.get_upper_board_mask(!color)
    }
}

impl FromStr for Rank {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "1" => Ok(Self::First),
            "2" => Ok(Self::Second),
            "3" => Ok(Self::Third),
            "4" => Ok(Self::Fourth),
            "5" => Ok(Self::Fifth),
            "6" => Ok(Self::Sixth),
            "7" => Ok(Self::Seventh),
            "8" => Ok(Self::Eighth),
            _ => Err(TimecatError::InvalidRankString { s: s.to_string() }),
        }
    }
}
