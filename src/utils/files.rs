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
    pub const unsafe fn from_int(i: u8) -> Self {
        std::mem::transmute(i)
    }

    #[inline]
    pub const unsafe fn from_index(i: usize) -> Self {
        Self::from_int(i as u8)
    }

    #[inline]
    pub fn left(self) -> Option<Self> {
        (self != Self::A).then(|| unsafe { Self::from_int(self.to_int() - 1) })
    }

    #[inline]
    pub fn right(self) -> Option<Self> {
        (self != Self::H).then(|| unsafe { Self::from_int(self.to_int() + 1) })
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
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard::new(0x0101_0101_0101_0101 << self.to_int())
    }

    #[inline]
    pub fn get_adjacent_files_bb(self) -> BitBoard {
        *get_item_unchecked!(BB_ADJACENT_FILES, self.to_index())
    }
}

impl fmt::Display for File {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (b'a' + self.to_int()) as char)
    }
}

impl TryFrom<char> for File {
    type Error = TimecatError;

    #[inline]
    fn try_from(c: char) -> Result<Self> {
        match c {
            'a' | 'A' => Ok(Self::A),
            'b' | 'B' => Ok(Self::B),
            'c' | 'C' => Ok(Self::C),
            'd' | 'D' => Ok(Self::D),
            'e' | 'E' => Ok(Self::E),
            'f' | 'F' => Ok(Self::F),
            'g' | 'G' => Ok(Self::G),
            'h' | 'H' => Ok(Self::H),
            _ => Err(TimecatError::InvalidFileString {
                s: c.to_string().into(),
            }),
        }
    }
}

impl FromStr for File {
    type Err = TimecatError;

    #[inline]
    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "a" | "A" => Ok(Self::A),
            "b" | "B" => Ok(Self::B),
            "c" | "C" => Ok(Self::C),
            "d" | "D" => Ok(Self::D),
            "e" | "E" => Ok(Self::E),
            "f" | "F" => Ok(Self::F),
            "g" | "G" => Ok(Self::G),
            "h" | "H" => Ok(Self::H),
            _ => Err(TimecatError::InvalidFileString {
                s: s.to_string().into(),
            }),
        }
    }
}
