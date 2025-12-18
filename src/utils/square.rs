use super::*;
pub use Square::*;
#[cfg(target_feature = "bmi2")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};

include!(concat!(env!("OUT_DIR"), "/magic.rs"));

#[cfg(target_feature = "bmi2")]
/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
fn get_bishop_moves_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic =
        *get_item_unchecked!(const { BISHOP_AND_ROOK_BMI_MASKS[0] }, square.to_index());
    let index = unsafe { _pext_u64(blockers.into_inner(), bmi2_magic.blockers_mask.into_inner()) }
        as usize
        + bmi2_magic.offset;
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_bishop_rays_bb().into_inner(),
        )
    };
    BitBoard::new(result)
}

#[cfg(target_feature = "bmi2")]
/// Get the moves for a rook on a particular square, given blockers blocking my movement.
fn get_rook_moves_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic =
        *get_item_unchecked!(const { BISHOP_AND_ROOK_BMI_MASKS[1] }, square.to_index());
    let index = unsafe { _pext_u64(blockers.into_inner(), bmi2_magic.blockers_mask.into_inner()) }
        as usize
        + bmi2_magic.offset;
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_rook_rays_bb().into_inner(),
        )
    };
    BitBoard::new(result)
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[cfg(not(target_feature = "bmi2"))]
fn get_bishop_moves_non_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic = *get_item_unchecked!(
        const { BISHOP_AND_ROOK_MAGIC_NUMBERS[0] },
        square.to_index()
    );
    *get_item_unchecked!(
        MOVES,
        magic.offset
            + ((blockers & magic.mask)
                .into_inner()
                .wrapping_mul(magic.magic_number)
                >> magic.right_shift) as usize,
    ) & square.get_bishop_rays_bb()
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[cfg(not(target_feature = "bmi2"))]
fn get_rook_moves_non_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic = *get_item_unchecked!(
        const { BISHOP_AND_ROOK_MAGIC_NUMBERS[1] },
        square.to_index()
    );
    *get_item_unchecked!(
        MOVES,
        magic.offset
            + ((blockers & magic.mask)
                .into_inner()
                .wrapping_mul(magic.magic_number)
                >> magic.right_shift) as usize,
    ) & square.get_rook_rays_bb()
}

/// Get the legal destination castle squares for both players
#[inline]
pub const fn get_castle_moves() -> BitBoard {
    const { BitBoard::new(0x5400000000000054) }
}

#[inline]
pub const fn get_pawn_source_double_moves() -> BitBoard {
    const { BitBoard::new(BB_RANK_2.into_inner() ^ BB_RANK_7.into_inner()) }
}

#[inline]
pub const fn get_pawn_dest_double_moves() -> BitBoard {
    const { BitBoard::new(BB_RANK_4.into_inner() ^ BB_RANK_5.into_inner()) }
}

#[rustfmt::skip]
#[repr(u8)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Square {
    A1 =  0, B1 =  1, C1 =  2, D1 =  3, E1 =  4, F1 =  5, G1 =  6, H1 =  7,
    A2 =  8, B2 =  9, C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
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

    #[inline]
    pub const unsafe fn from_int(int: u8) -> Self {
        unsafe { std::mem::transmute(int) }
    }

    #[inline]
    pub const unsafe fn from_index(index: usize) -> Self {
        Self::from_int(index as u8)
    }

    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        unsafe { Self::from_int((rank.to_int() << 3) ^ file.to_int()) }
    }

    #[inline]
    pub const fn get_rank(self) -> Rank {
        unsafe { Rank::from_int(self.to_int() >> 3) }
    }

    #[inline]
    pub const fn get_rank_bb(self) -> BitBoard {
        self.get_rank().to_bitboard()
    }

    #[inline]
    pub const fn get_file(self) -> File {
        unsafe { File::from_int(self.to_int() & 7) }
    }

    #[inline]
    pub const fn get_file_bb(self) -> BitBoard {
        self.get_file().to_bitboard()
    }

    #[inline]
    pub fn up(self) -> Option<Self> {
        Some(Self::from_rank_and_file(
            self.get_rank().up()?,
            self.get_file(),
        ))
    }

    #[inline]
    pub fn down(self) -> Option<Self> {
        Some(Self::from_rank_and_file(
            self.get_rank().down()?,
            self.get_file(),
        ))
    }

    #[inline]
    pub fn left(self) -> Option<Self> {
        Some(Self::from_rank_and_file(
            self.get_rank(),
            self.get_file().left()?,
        ))
    }

    #[inline]
    pub fn right(self) -> Option<Self> {
        Some(Self::from_rank_and_file(
            self.get_rank(),
            self.get_file().right()?,
        ))
    }

    #[inline]
    pub fn up_left(self) -> Option<Self> {
        self.up()?.left()
    }

    #[inline]
    pub fn up_right(self) -> Option<Self> {
        self.up()?.right()
    }

    #[inline]
    pub fn down_left(self) -> Option<Self> {
        self.down()?.left()
    }

    #[inline]
    pub fn down_right(self) -> Option<Self> {
        self.down()?.right()
    }

    #[inline]
    pub fn forward(self, color: Color) -> Option<Self> {
        match color {
            White => self.up(),
            Black => self.down(),
        }
    }

    #[inline]
    pub fn backward(self, color: Color) -> Option<Self> {
        match color {
            White => self.down(),
            Black => self.up(),
        }
    }

    #[inline]
    pub fn wrapping_up(self) -> Self {
        Self::from_rank_and_file(self.get_rank().wrapping_up(), self.get_file())
    }

    #[inline]
    pub fn wrapping_down(self) -> Self {
        Self::from_rank_and_file(self.get_rank().wrapping_down(), self.get_file())
    }

    #[inline]
    pub fn wrapping_left(self) -> Self {
        Self::from_rank_and_file(self.get_rank(), self.get_file().wrapping_left())
    }

    #[inline]
    pub fn wrapping_right(self) -> Self {
        Self::from_rank_and_file(self.get_rank(), self.get_file().wrapping_right())
    }

    #[inline]
    pub fn wrapping_forward(self, color: Color) -> Self {
        match color {
            White => self.wrapping_up(),
            Black => self.wrapping_down(),
        }
    }

    #[inline]
    pub fn wrapping_backward(self, color: Color) -> Self {
        match color {
            White => self.wrapping_down(),
            Black => self.wrapping_up(),
        }
    }

    #[inline]
    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard::new(1 << self.to_int())
    }

    #[inline]
    pub fn distance(self, other: Self) -> u8 {
        self.get_file()
            .to_int()
            .abs_diff(other.get_file().to_int())
            .max(self.get_rank().to_int().abs_diff(other.get_rank().to_int()))
    }

    #[inline]
    pub const fn manhattan_distance(self, other: Self) -> u8 {
        self.get_file().to_int().abs_diff(other.get_file().to_int())
            + self.get_rank().to_int().abs_diff(other.get_rank().to_int())
    }

    pub fn knight_distance(self, other: Self) -> u8 {
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

    #[inline]
    pub const fn vertical_mirror(self) -> Self {
        unsafe { Self::from_int(self.to_int() ^ 7) }
    }

    #[inline]
    pub const fn horizontal_mirror(self) -> Self {
        unsafe { Self::from_int(self.to_int() ^ 0x38) }
    }

    #[inline]
    pub const fn rotate(self) -> Self {
        unsafe { Self::from_int(self.to_int() ^ 0x3f) }
    }

    /// Get the rays for a bishop on a particular square.
    #[inline]
    pub fn get_diagonal_bishop_rays_bb(self) -> BitBoard {
        *get_item_unchecked!(BISHOP_DIAGONAL_RAYS, self.to_index())
    }

    /// Get the rays for a bishop on a particular square.
    #[inline]
    pub fn get_anti_diagonal_bishop_rays_bb(self) -> BitBoard {
        *get_item_unchecked!(BISHOP_ANTI_DIAGONAL_RAYS, self.to_index())
    }

    /// Get the rays for a bishop on a particular square.
    #[inline]
    pub fn get_bishop_rays_bb(self) -> BitBoard {
        *get_item_unchecked!(BISHOP_RAYS, self.to_index())
    }

    /// Get the rays for a rook on a particular square.
    #[inline]
    pub fn get_rook_rays_bb(self) -> BitBoard {
        *get_item_unchecked!(ROOK_RAYS, self.to_index())
    }

    /// Get the rays for a rook and bishop together on a particular square.
    #[inline]
    pub fn get_all_direction_rays_bb(self) -> BitBoard {
        *get_item_unchecked!(ALL_DIRECTION_RAYS, self.to_index())
    }

    /// Get a line between these two squares, not including the squares themselves.
    #[inline]
    pub fn between(self, other: Self) -> BitBoard {
        *get_item_unchecked!(BETWEEN, self.to_index(), other.to_index())
    }

    /// Get a line (extending to infinity, which in chess is 8 squares), given two squares.
    /// This line does extend past the squares.
    #[inline]
    pub fn line(self, other: Self) -> BitBoard {
        *get_item_unchecked!(LINE, self.to_index(), other.to_index())
    }

    /// Get the quiet pawn moves (non-captures) for a particular square, given the pawn's color and
    /// the potential blocking pieces.
    #[inline]
    pub fn get_pawn_quiets(self, color: Color, blockers: BitBoard) -> BitBoard {
        if self.get_rank() == color.to_second_rank()
            && !(blockers & color.to_third_rank_bitboard() & self.get_file().to_bitboard())
                .is_empty()
        {
            BitBoard::EMPTY
        } else {
            *get_item_unchecked!(
                const { PAWN_MOVES_AND_ATTACKS[0] },
                color.to_index(),
                self.to_index()
            ) & !blockers
        }
    }

    /// Get the pawn capture move for a particular square, given the pawn's color and the potential
    /// victims
    #[inline]
    pub fn get_pawn_attacks(self, color: Color, blockers: BitBoard) -> BitBoard {
        *get_item_unchecked!(
            const { PAWN_MOVES_AND_ATTACKS[1] },
            color.to_index(),
            self.to_index()
        ) & blockers
    }

    /// Get all the pawn moves for a particular square, given the pawn's color and the potential
    /// blocking pieces and victims.
    #[inline]
    pub fn get_pawn_moves(self, color: Color, blockers: BitBoard) -> BitBoard {
        self.get_pawn_quiets(color, blockers) ^ self.get_pawn_attacks(color, blockers)
    }

    /// Get the knight moves for a particular square.
    #[inline]
    pub fn get_knight_moves(self) -> BitBoard {
        *get_item_unchecked!(KNIGHT_MOVES, self.to_index())
    }

    /// Get the moves for a bishop on a particular square, given blockers blocking my movement.
    #[inline]
    pub fn get_bishop_moves(self, blockers: BitBoard) -> BitBoard {
        #[cfg(target_feature = "bmi2")]
        {
            get_bishop_moves_bmi(self, blockers)
        }
        #[cfg(not(target_feature = "bmi2"))]
        {
            get_bishop_moves_non_bmi(self, blockers)
        }
    }

    /// Get the moves for a rook on a particular square, given blockers blocking my movement.
    #[inline]
    pub fn get_rook_moves(self, blockers: BitBoard) -> BitBoard {
        #[cfg(target_feature = "bmi2")]
        {
            get_rook_moves_bmi(self, blockers)
        }
        #[cfg(not(target_feature = "bmi2"))]
        {
            get_rook_moves_non_bmi(self, blockers)
        }
    }

    #[inline]
    pub fn get_queen_moves(self, blockers: BitBoard) -> BitBoard {
        self.get_bishop_moves(blockers) ^ self.get_rook_moves(blockers)
    }

    /// Get the king moves for a particular square.
    #[inline]
    pub fn get_king_moves(self) -> BitBoard {
        *get_item_unchecked!(KING_MOVES, self.to_index())
    }
}

macro_rules! generate_error {
    ($s:expr) => {
        TimecatError::InvalidSquareString {
            s: $s.to_string().into(),
        }
    };
}

impl FromStr for Square {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        let binding = s.to_lowercase();
        let mut ch = binding.trim().chars();
        let file = ch.next().ok_or_else(|| generate_error!(s))?.try_into()?;
        let rank = ch.next().ok_or_else(|| generate_error!(s))?.try_into()?;
        ch.next().map_or_else(
            || Ok(Self::from_rank_and_file(rank, file)),
            |_| Err(generate_error!(s)),
        )
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.get_file(), self.get_rank())
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for Square {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(int) = ob.extract::<u8>() {
            return Ok(unsafe { Self::from_int(int) });
        }
        if let Ok(s) = ob.extract::<&str>()
            && let Ok(square) = s.parse()
        {
            return Ok(square);
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string().into(),
            to: std::any::type_name::<Self>().into(),
        }
        .into())
    }
}
