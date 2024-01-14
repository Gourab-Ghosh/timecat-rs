use super::*;

#[rustfmt::skip]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Square {
    A1, A2, A3, A4, A5, A6, A7, A8,
    B1, B2, B3, B4, B5, B6, B7, B8,
    C1, C2, C3, C4, C5, C6, C7, C8,
    D1, D2, D3, D4, D5, D6, D7, D8,
    E1, E2, E3, E4, E5, E6, E7, E8,
    F1, F2, F3, F4, F5, F6, F7, F8,
    G1, G2, G3, G4, G5, G6, G7, G8,
    H1, H2, H3, H4, H5, H6, H7, H8,
}

pub const NUM_SQUARES: usize = 64;

#[rustfmt::skip]
pub const ALL_SQUARES: [Square; NUM_SQUARES] = [
    A1, A2, A3, A4, A5, A6, A7, A8,
    B1, B2, B3, B4, B5, B6, B7, B8,
    C1, C2, C3, C4, C5, C6, C7, C8,
    D1, D2, D3, D4, D5, D6, D7, D8,
    E1, E2, E3, E4, E5, E6, E7, E8,
    F1, F2, F3, F4, F5, F6, F7, F8,
    G1, G2, G3, G4, G5, G6, G7, G8,
    H1, H2, H3, H4, H5, H6, H7, H8,
];

impl Square {
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    pub const fn to_index(self) -> usize {
        self as usize
    }

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
    type Err = ChessError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(ChessError::InvalidSquare);
        }
        let ch: Vec<char> = s.chars().collect();
        match ch[0] {
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => {}
            _ => {
                return Err(ChessError::InvalidSquare);
            }
        }
        match ch[1] {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {}
            _ => {
                return Err(ChessError::InvalidSquare);
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
