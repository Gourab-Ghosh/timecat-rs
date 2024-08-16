use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    #[inline]
    pub const fn from_index(i: usize) -> Self {
        unsafe { std::mem::transmute((i & 7) as u8) }
    }

    #[inline]
    pub const fn left(self) -> Option<Self> {
        match self {
            Self::A => None,
            Self::B => Some(Self::A),
            Self::C => Some(Self::B),
            Self::D => Some(Self::C),
            Self::E => Some(Self::D),
            Self::F => Some(Self::E),
            Self::G => Some(Self::F),
            Self::H => Some(Self::G),
        }
    }

    #[inline]
    pub const fn right(self) -> Option<Self> {
        match self {
            Self::A => Some(Self::B),
            Self::B => Some(Self::C),
            Self::C => Some(Self::D),
            Self::D => Some(Self::E),
            Self::E => Some(Self::F),
            Self::F => Some(Self::G),
            Self::G => Some(Self::H),
            Self::H => None,
        }
    }

    #[inline]
    pub fn wrapping_left(self) -> Self {
        self.left().unwrap_or(Self::H)
    }

    #[inline]
    pub fn wrapping_right(self) -> Self {
        self.right().unwrap_or(Self::A)
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline]
    pub fn to_bitboard(self) -> BitBoard {
        *get_item_unchecked!(BB_FILES, self.to_index())
    }

    #[inline]
    pub fn get_adjacent_files_bb(self) -> BitBoard {
        *get_item_unchecked!(BB_ADJACENT_FILES, self.to_index())
    }
}

impl FromStr for File {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().trim() {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            "d" => Ok(Self::D),
            "e" => Ok(Self::E),
            "f" => Ok(Self::F),
            "g" => Ok(Self::G),
            "h" => Ok(Self::H),
            _ => Err(TimecatError::InvalidFileString { s: s.to_string() }),
        }
    }
}
