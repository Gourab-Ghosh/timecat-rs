use super::*;
use Square::*;

#[rustfmt::skip]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

pub const NUM_SQUARES: usize = 64;

#[rustfmt::skip]
pub const ALL_SQUARES: [Square; NUM_SQUARES] = [
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
];

impl Square {
    #[inline(always)]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        unsafe { transmute(8 * rank.to_int() + file.to_int()) }
    }

    #[inline(always)]
    pub fn get_rank(&self) -> Rank {
        Rank::from_index(self.to_index() >> 3)
    }

    #[inline(always)]
    pub fn get_file(&self) -> File {
        File::from_index(self.to_index() & 7)
    }

    #[inline(always)]
    pub fn up(&self) -> Option<Square> {
        if self.get_rank() == Rank::Eighth {
            None
        } else {
            Some(Square::from_rank_and_file(
                self.get_rank().up(),
                self.get_file(),
            ))
        }
    }

    #[inline(always)]
    pub fn down(&self) -> Option<Square> {
        if self.get_rank() == Rank::First {
            None
        } else {
            Some(Square::from_rank_and_file(
                self.get_rank().down(),
                self.get_file(),
            ))
        }
    }

    #[inline(always)]
    pub fn left(&self) -> Option<Square> {
        if self.get_file() == File::A {
            None
        } else {
            Some(Square::from_rank_and_file(
                self.get_rank(),
                self.get_file().left(),
            ))
        }
    }

    #[inline(always)]
    pub fn right(&self) -> Option<Square> {
        if self.get_file() == File::H {
            None
        } else {
            Some(Square::from_rank_and_file(
                self.get_rank(),
                self.get_file().right(),
            ))
        }
    }

    #[inline(always)]
    pub fn forward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down(),
        }
    }

    #[inline(always)]
    pub fn backward(&self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up(),
        }
    }

    #[inline(always)]
    pub fn wrapping_up(&self) -> Square {
        Square::from_rank_and_file(self.get_rank().up(), self.get_file())
    }

    #[inline(always)]
    pub fn wrapping_down(&self) -> Square {
        Square::from_rank_and_file(self.get_rank().down(), self.get_file())
    }

    #[inline(always)]
    pub fn wrapping_left(&self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().left())
    }

    #[inline(always)]
    pub fn wrapping_right(&self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().right())
    }

    #[inline(always)]
    pub fn wrapping_forward(&self, color: Color) -> Square {
        match color {
            Color::White => self.wrapping_up(),
            Color::Black => self.wrapping_down(),
        }
    }

    #[inline(always)]
    pub fn wrapping_backward(&self, color: Color) -> Square {
        match color {
            Color::White => self.wrapping_down(),
            Color::Black => self.wrapping_up(),
        }
    }
}

impl FromStr for Square {
    type Err = EngineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(EngineError::InvalidSquareString { s: s.to_string() });
        }
        let ch: Vec<char> = s.chars().collect();
        match ch[0] {
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => {}
            _ => {
                return Err(EngineError::InvalidSquareString { s: s.to_string() });
            }
        }
        match ch[1] {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {}
            _ => {
                return Err(EngineError::InvalidSquareString { s: s.to_string() });
            }
        }
        Ok(Square::from_rank_and_file(
            Rank::from_index((ch[1] as usize) - ('1' as usize)),
            File::from_index((ch[0] as usize) - ('a' as usize)),
        ))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            (b'a' + ((self.to_index() & 7) as u8)) as char,
            (b'1' + ((self.to_index() >> 3) as u8)) as char
        )
    }
}
