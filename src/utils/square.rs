use super::*;
pub use Square::*;

#[rustfmt::skip]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[inline]
    pub const fn to_int(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    #[rustfmt::skip]
    #[inline]
    pub const fn from_int(int: u8) -> Self {
        match int {
             0 => A1,  1 => B1,  2 => C1,  3 => D1,  4 => E1,  5 => F1,  6 => G1,  7 => H1,
             8 => A2,  9 => B2, 10 => C2, 11 => D2, 12 => E2, 13 => F2, 14 => G2, 15 => H2,
            16 => A3, 17 => B3, 18 => C3, 19 => D3, 20 => E3, 21 => F3, 22 => G3, 23 => H3,
            24 => A4, 25 => B4, 26 => C4, 27 => D4, 28 => E4, 29 => F4, 30 => G4, 31 => H4,
            32 => A5, 33 => B5, 34 => C5, 35 => D5, 36 => E5, 37 => F5, 38 => G5, 39 => H5,
            40 => A6, 41 => B6, 42 => C6, 43 => D6, 44 => E6, 45 => F6, 46 => G6, 47 => H6,
            48 => A7, 49 => B7, 50 => C7, 51 => D7, 52 => E7, 53 => F7, 54 => G7, 55 => H7,
            56 => A8, 57 => B8, 58 => C8, 59 => D8, 60 => E8, 61 => F8, 62 => G8, 63 => H8,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub const fn from_index(index: usize) -> Self {
        Self::from_int(index as u8)
    }

    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Self::from_int((rank.to_int() << 3) ^ file.to_int())
    }

    #[inline]
    pub fn get_rank(self) -> Rank {
        Rank::from_index(self.to_index() >> 3)
    }

    #[inline]
    pub fn get_file(self) -> File {
        File::from_index(self.to_index() & 7)
    }

    #[inline]
    pub fn up(self) -> Option<Square> {
        Some(Square::from_rank_and_file(
            self.get_rank().up()?,
            self.get_file(),
        ))
    }

    #[inline]
    pub fn down(self) -> Option<Square> {
        Some(Square::from_rank_and_file(
            self.get_rank().down()?,
            self.get_file(),
        ))
    }

    #[inline]
    pub fn left(self) -> Option<Square> {
        Some(Square::from_rank_and_file(
            self.get_rank(),
            self.get_file().left()?,
        ))
    }

    #[inline]
    pub fn right(self) -> Option<Square> {
        Some(Square::from_rank_and_file(
            self.get_rank(),
            self.get_file().right()?,
        ))
    }

    #[inline]
    pub fn forward(self, color: Color) -> Option<Square> {
        match color {
            White => self.up(),
            Black => self.down(),
        }
    }

    #[inline]
    pub fn backward(self, color: Color) -> Option<Square> {
        match color {
            White => self.down(),
            Black => self.up(),
        }
    }

    #[inline]
    pub fn wrapping_up(self) -> Square {
        Square::from_rank_and_file(self.get_rank().wrapping_up(), self.get_file())
    }

    #[inline]
    pub fn wrapping_down(self) -> Square {
        Square::from_rank_and_file(self.get_rank().wrapping_down(), self.get_file())
    }

    #[inline]
    pub fn wrapping_left(self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().wrapping_left())
    }

    #[inline]
    pub fn wrapping_right(self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().wrapping_right())
    }

    #[inline]
    pub fn wrapping_forward(self, color: Color) -> Square {
        match color {
            White => self.wrapping_up(),
            Black => self.wrapping_down(),
        }
    }

    #[inline]
    pub fn wrapping_backward(self, color: Color) -> Square {
        match color {
            White => self.wrapping_down(),
            Black => self.wrapping_up(),
        }
    }

    #[inline]
    pub fn to_bitboard(self) -> BitBoard {
        *get_item_unchecked!(BB_SQUARES, self.to_index())
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

    #[rustfmt::skip]
    #[inline]
    pub const fn vertical_mirror(self) -> Self {
        match self {
            A1 => H1, B1 => G1, C1 => F1, D1 => E1, E1 => D1, F1 => C1, G1 => B1, H1 => A1,
            A2 => H2, B2 => G2, C2 => F2, D2 => E2, E2 => D2, F2 => C2, G2 => B2, H2 => A2,
            A3 => H3, B3 => G3, C3 => F3, D3 => E3, E3 => D3, F3 => C3, G3 => B3, H3 => A3,
            A4 => H4, B4 => G4, C4 => F4, D4 => E4, E4 => D4, F4 => C4, G4 => B4, H4 => A4,
            A5 => H5, B5 => G5, C5 => F5, D5 => E5, E5 => D5, F5 => C5, G5 => B5, H5 => A5,
            A6 => H6, B6 => G6, C6 => F6, D6 => E6, E6 => D6, F6 => C6, G6 => B6, H6 => A6,
            A7 => H7, B7 => G7, C7 => F7, D7 => E7, E7 => D7, F7 => C7, G7 => B7, H7 => A7,
            A8 => H8, B8 => G8, C8 => F8, D8 => E8, E8 => D8, F8 => C8, G8 => B8, H8 => A8,
        }
    }

    #[rustfmt::skip]
    #[inline]
    pub const fn horizontal_mirror(self) -> Self {
        match self {
            A1 => A8, B1 => B8, C1 => C8, D1 => D8, E1 => E8, F1 => F8, G1 => G8, H1 => H8,
            A2 => A7, B2 => B7, C2 => C7, D2 => D7, E2 => E7, F2 => F7, G2 => G7, H2 => H7,
            A3 => A6, B3 => B6, C3 => C6, D3 => D6, E3 => E6, F3 => F6, G3 => G6, H3 => H6,
            A4 => A5, B4 => B5, C4 => C5, D4 => D5, E4 => E5, F4 => F5, G4 => G5, H4 => H5,
            A5 => A4, B5 => B4, C5 => C4, D5 => D4, E5 => E4, F5 => F4, G5 => G4, H5 => H4,
            A6 => A3, B6 => B3, C6 => C3, D6 => D3, E6 => E3, F6 => F3, G6 => G3, H6 => H3,
            A7 => A2, B7 => B2, C7 => C2, D7 => D2, E7 => E2, F7 => F2, G7 => G2, H7 => H2,
            A8 => A1, B8 => B1, C8 => C1, D8 => D1, E8 => E1, F8 => F1, G8 => G1, H8 => H1,
        }
    }

    #[rustfmt::skip]
    #[inline]
    pub const fn rotate(self) -> Self {
        self.vertical_mirror().horizontal_mirror()
    }
}

impl FromStr for Square {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() < 2 {
            return Err(TimecatError::InvalidSquareString { s: s.to_string() });
        }
        let ch = s.to_lowercase().chars().collect_vec();
        match ch[0] {
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' => {}
            _ => {
                return Err(TimecatError::InvalidSquareString { s: s.to_string() });
            }
        }
        match ch[1] {
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {}
            _ => {
                return Err(TimecatError::InvalidSquareString { s: s.to_string() });
            }
        }
        Ok(Square::from_rank_and_file(
            Rank::from_index(((ch[1] as usize) - ('1' as usize)) & 7),
            File::from_index(((ch[0] as usize) - ('a' as usize)) & 7),
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
