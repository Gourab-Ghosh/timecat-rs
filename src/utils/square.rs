use super::*;
pub use Square::*;

#[rustfmt::skip]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub const fn from_int(index: u8) -> Self {
        unsafe { transmute(index & 63) }
    }

    #[inline(always)]
    pub const fn from_index(index: usize) -> Self {
        Self::from_int(index as u8)
    }

    #[inline(always)]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        unsafe { transmute((rank.to_int() << 3) ^ file.to_int()) }
    }

    #[inline(always)]
    pub fn get_rank(self) -> Rank {
        Rank::from_index(self.to_index() >> 3)
    }

    #[inline(always)]
    pub fn get_file(self) -> File {
        File::from_index(self.to_index() & 7)
    }

    #[inline(always)]
    pub fn up(self) -> Option<Square> {
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
    pub fn down(self) -> Option<Square> {
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
    pub fn left(self) -> Option<Square> {
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
    pub fn right(self) -> Option<Square> {
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
    pub fn forward(self, color: Color) -> Option<Square> {
        match color {
            White => self.up(),
            Black => self.down(),
        }
    }

    #[inline(always)]
    pub fn backward(self, color: Color) -> Option<Square> {
        match color {
            White => self.down(),
            Black => self.up(),
        }
    }

    #[inline(always)]
    pub fn wrapping_up(self) -> Square {
        Square::from_rank_and_file(self.get_rank().up(), self.get_file())
    }

    #[inline(always)]
    pub fn wrapping_down(self) -> Square {
        Square::from_rank_and_file(self.get_rank().down(), self.get_file())
    }

    #[inline(always)]
    pub fn wrapping_left(self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().left())
    }

    #[inline(always)]
    pub fn wrapping_right(self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().right())
    }

    #[inline(always)]
    pub fn wrapping_forward(self, color: Color) -> Square {
        match color {
            White => self.wrapping_up(),
            Black => self.wrapping_down(),
        }
    }

    #[inline(always)]
    pub fn wrapping_backward(self, color: Color) -> Square {
        match color {
            White => self.wrapping_down(),
            Black => self.wrapping_up(),
        }
    }

    #[inline(always)]
    pub fn to_bitboard(self) -> BitBoard {
        *get_item_unchecked!(BB_SQUARES, self.to_index())
    }

    #[inline(always)]
    pub fn mirror(self) -> Square {
        *get_item_unchecked!(SQUARES_180, self.to_index())
    }

    pub fn distance(self, other: Square) -> u8 {
        let (file1, rank1) = (self.get_file(), self.get_rank());
        let (file2, rank2) = (other.get_file(), other.get_rank());
        let file_distance = file1.to_int().abs_diff(file2.to_int());
        let rank_distance = rank1.to_int().abs_diff(rank2.to_int());
        file_distance.max(rank_distance)
    }

    pub fn manhattan_distance(self, other: Square) -> u8 {
        let (file1, rank1) = (self.get_file(), self.get_rank());
        let (file2, rank2) = (other.get_file(), other.get_rank());
        let file_distance = file1.to_int().abs_diff(file2.to_int());
        let rank_distance = rank1.to_int().abs_diff(rank2.to_int());
        file_distance + rank_distance
    }

    pub fn knight_distance(self, other: Square) -> u8 {
        let dx = self.get_file().to_int().abs_diff(other.get_file().to_int());
        let dy = self.get_rank().to_int().abs_diff(other.get_rank().to_int());

        if dx + dy == 1 {
            return 3;
        }
        if dx == 2 && dy == 2 {
            return 4;
        }
        if dx == 1
            && dy == 1
            && (!(self.to_bitboard() & BB_CORNERS).is_empty()
                || !(other.to_bitboard() & BB_CORNERS).is_empty())
        {
            // Special case only for corner squares
            return 4;
        }

        let dx_f64 = dx as f64;
        let dy_f64 = dy as f64;

        let m = (dx_f64 / 2.0)
            .max(dy_f64 / 2.0)
            .max((dx_f64 + dy_f64) / 3.0)
            .ceil() as u8;
        m + ((m + dx + dy) % 2)
    }
}

impl FromStr for Square {
    type Err = EngineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(EngineError::InvalidSquareString { s: s.to_string() });
        }
        let ch = s.to_lowercase().chars().collect_vec();
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
            (b'1' + ((self.to_index() >> 3) as u8)) as char,
        )
    }
}
