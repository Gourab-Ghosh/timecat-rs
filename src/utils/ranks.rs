use super::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl Rank {
    #[inline(always)]
    pub const fn from_index(i: usize) -> Rank {
        unsafe { transmute((i as u8) & 7) }
    }

    #[inline(always)]
    pub const fn down(self) -> Rank {
        Rank::from_index(self.to_index().wrapping_sub(1))
    }

    #[inline(always)]
    pub const fn up(self) -> Rank {
        Rank::from_index(self.to_index() + 1)
    }

    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn to_int(self) -> u8 {
        self as u8
    }
}

impl FromStr for Rank {
    type Err = EngineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "1" => Ok(Rank::First),
            "2" => Ok(Rank::Second),
            "3" => Ok(Rank::Third),
            "4" => Ok(Rank::Fourth),
            "5" => Ok(Rank::Fifth),
            "6" => Ok(Rank::Sixth),
            "7" => Ok(Rank::Seventh),
            "8" => Ok(Rank::Eighth),
            _ => Err(EngineError::InvalidRankString { s: s.to_string() }),
        }
    }
}
