use super::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

pub const NUM_FILES: usize = 8;

pub const ALL_FILES: [File; NUM_FILES] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

impl File {
    #[inline(always)]
    pub const fn from_index(i: usize) -> File {
        unsafe { transmute((i as u8) & 7) }
    }

    #[inline(always)]
    pub const fn left(self) -> File {
        File::from_index(self.to_index().wrapping_sub(1))
    }

    #[inline(always)]
    pub const fn right(self) -> File {
        File::from_index(self.to_index() + 1)
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

impl FromStr for File {
    type Err = EngineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "a" => Ok(File::A),
            "b" => Ok(File::B),
            "c" => Ok(File::C),
            "d" => Ok(File::D),
            "e" => Ok(File::E),
            "f" => Ok(File::F),
            "g" => Ok(File::G),
            "h" => Ok(File::H),
            _ => Err(EngineError::InvalidFileString { s: s.to_string() }),
        }
    }
}
