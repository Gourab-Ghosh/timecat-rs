use super::*;

const UPPER_BOARD_MASK: [[BitBoard; 8]; 2] = [
    [
        BitBoard::new(0xffff_ffff_ffff_ff00),
        BitBoard::new(0xffff_ffff_ffff_0000),
        BitBoard::new(0xffff_ffff_ff00_0000),
        BitBoard::new(0xffff_ffff_0000_0000),
        BitBoard::new(0xffff_ff00_0000_0000),
        BitBoard::new(0xffff_0000_0000_0000),
        BitBoard::new(0xff00_0000_0000_0000),
        BitBoard::new(0x0000_0000_0000_0000),
    ],
    [
        BitBoard::new(0x00ff_ffff_ffff_ffff),
        BitBoard::new(0x0000_ffff_ffff_ffff),
        BitBoard::new(0x0000_00ff_ffff_ffff),
        BitBoard::new(0x0000_0000_ffff_ffff),
        BitBoard::new(0x0000_0000_00ff_ffff),
        BitBoard::new(0x0000_0000_0000_ffff),
        BitBoard::new(0x0000_0000_0000_00ff),
        BitBoard::new(0x0000_0000_0000_0000),
    ],
];

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
    pub const fn from_int(i: u8) -> Self {
        unsafe { std::mem::transmute(i) }
    }

    #[inline]
    pub const fn from_index(i: usize) -> Self {
        Self::from_int(i as u8)
    }

    #[inline]
    pub fn up(self) -> Option<Self> {
        *get_item_unchecked!(
            const [
                Some(Self::Second),
                Some(Self::Third),
                Some(Self::Fourth),
                Some(Self::Fifth),
                Some(Self::Sixth),
                Some(Self::Seventh),
                Some(Self::Eighth),
                None,
            ],
            self.to_index(),
        )
    }

    #[inline]
    pub fn down(self) -> Option<Self> {
        *get_item_unchecked!(
            const [
                None,
                Some(Self::First),
                Some(Self::Second),
                Some(Self::Third),
                Some(Self::Fourth),
                Some(Self::Fifth),
                Some(Self::Sixth),
                Some(Self::Seventh),
            ],
            self.to_index(),
        )
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
    pub fn to_bitboard(self) -> BitBoard {
        *get_item_unchecked!(BB_RANKS, self.to_index())
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
