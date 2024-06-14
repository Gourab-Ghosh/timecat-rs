use super::*;
pub use Color::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn to_my_backrank(self) -> Rank {
        match self {
            White => Rank::First,
            Black => Rank::Eighth,
        }
    }

    #[inline]
    pub const fn to_their_backrank(self) -> Rank {
        match self {
            White => Rank::Eighth,
            Black => Rank::First,
        }
    }

    #[inline]
    pub const fn to_second_rank(self) -> Rank {
        match self {
            White => Rank::Second,
            Black => Rank::Seventh,
        }
    }

    #[inline]
    pub const fn to_third_rank(self) -> Rank {
        match self {
            White => Rank::Third,
            Black => Rank::Sixth,
        }
    }

    #[inline]
    pub const fn to_fourth_rank(self) -> Rank {
        match self {
            White => Rank::Fourth,
            Black => Rank::Fifth,
        }
    }

    #[inline]
    pub const fn to_seventh_rank(self) -> Rank {
        match self {
            White => Rank::Seventh,
            Black => Rank::Second,
        }
    }
}

impl Not for Color {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        match self {
            White => Black,
            Black => White,
        }
    }
}
