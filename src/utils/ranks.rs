use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[inline]
    pub const fn from_index(i: usize) -> Self {
        match i {
            0 => Self::First,
            1 => Self::Second,
            2 => Self::Third,
            3 => Self::Fourth,
            4 => Self::Fifth,
            5 => Self::Sixth,
            6 => Self::Seventh,
            7 => Self::Eighth,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub const fn up(self) -> Option<Self> {
        match self {
            Self::First => Some(Self::Second),
            Self::Second => Some(Self::Third),
            Self::Third => Some(Self::Fourth),
            Self::Fourth => Some(Self::Fifth),
            Self::Fifth => Some(Self::Sixth),
            Self::Sixth => Some(Self::Seventh),
            Self::Seventh => Some(Self::Eighth),
            Self::Eighth => None,
        }
    }

    #[inline]
    pub const fn down(self) -> Option<Self> {
        match self {
            Self::First => None,
            Self::Second => Some(Self::First),
            Self::Third => Some(Self::Second),
            Self::Fourth => Some(Self::Third),
            Self::Fifth => Some(Self::Fourth),
            Self::Sixth => Some(Self::Fifth),
            Self::Seventh => Some(Self::Sixth),
            Self::Eighth => Some(Self::Seventh),
        }
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
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
            Self::Fourth => 3,
            Self::Fifth => 4,
            Self::Sixth => 5,
            Self::Seventh => 6,
            Self::Eighth => 7,
        }
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self.to_int() as usize
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
