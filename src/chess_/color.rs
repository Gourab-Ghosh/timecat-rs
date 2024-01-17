use super::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

pub const ALL_COLORS: [Color; 2] = [Color::White, Color::Black];

impl Color {
    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn to_my_backrank(self) -> Rank {
        match self {
            Self::White => Rank::First,
            Self::Black => Rank::Eighth,
        }
    }

    #[inline(always)]
    pub const fn to_their_backrank(self) -> Rank {
        match self {
            Self::White => Rank::Eighth,
            Self::Black => Rank::First,
        }
    }

    #[inline(always)]
    pub const fn to_second_rank(self) -> Rank {
        match self {
            Self::White => Rank::Second,
            Self::Black => Rank::Seventh,
        }
    }

    #[inline(always)]
    pub const fn to_fourth_rank(self) -> Rank {
        match self {
            Self::White => Rank::Fourth,
            Self::Black => Rank::Fifth,
        }
    }

    #[inline(always)]
    pub const fn to_seventh_rank(self) -> Rank {
        match self {
            Self::White => Rank::Seventh,
            Self::Black => Rank::Second,
        }
    }
}

impl Not for Color {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        if self == Self::White {
            Self::Black
        } else {
            Self::White
        }
    }
}
