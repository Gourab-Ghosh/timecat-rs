use super::*;
pub use Color::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn to_my_backrank(self) -> Rank {
        match self {
            White => Rank::First,
            Black => Rank::Eighth,
        }
    }

    #[inline(always)]
    pub const fn to_their_backrank(self) -> Rank {
        match self {
            White => Rank::Eighth,
            Black => Rank::First,
        }
    }

    #[inline(always)]
    pub const fn to_second_rank(self) -> Rank {
        match self {
            White => Rank::Second,
            Black => Rank::Seventh,
        }
    }

    #[inline(always)]
    pub const fn to_third_rank(self) -> Rank {
        match self {
            White => Rank::Third,
            Black => Rank::Sixth,
        }
    }

    #[inline(always)]
    pub const fn to_fourth_rank(self) -> Rank {
        match self {
            White => Rank::Fourth,
            Black => Rank::Fifth,
        }
    }

    #[inline(always)]
    pub const fn to_seventh_rank(self) -> Rank {
        match self {
            White => Rank::Seventh,
            Black => Rank::Second,
        }
    }
}

impl Not for Color {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        match self {
            White => Black,
            Black => White,
        }
    }
}
