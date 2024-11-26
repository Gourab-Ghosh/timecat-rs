use super::*;
use std::arch::x86_64::{_pdep_u64, _pext_u64};
pub use Square::*;

include!(concat!(env!("OUT_DIR"), "/magic.rs"));

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
fn get_bishop_moves_non_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic =
        *get_item_unchecked!(const BISHOP_AND_ROOK_MAGIC_NUMBERS[0], square.to_index());
    *get_item_unchecked!(
        MOVES,
        magic.offset
            + ((blockers & magic.mask)
                .into_inner()
                .wrapping_mul(magic.magic_number)
                >> magic.right_shift) as usize,
    ) & square.get_bishop_rays_bb()
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
fn get_bishop_moves_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic = *get_item_unchecked!(const BISHOP_AND_ROOK_BMI_MASKS[0], square.to_index());
    let index = unsafe { _pext_u64(blockers.into_inner(), bmi2_magic.blockers_mask.into_inner()) }
        as usize
        + bmi2_magic.offset;
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_bishop_rays_bb().into_inner(),
        )
    };
    return BitBoard::new(result);
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
fn get_rook_moves_non_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: Magic =
        *get_item_unchecked!(const BISHOP_AND_ROOK_MAGIC_NUMBERS[1], square.to_index());
    *get_item_unchecked!(
        MOVES,
        magic.offset
            + ((blockers & magic.mask)
                .into_inner()
                .wrapping_mul(magic.magic_number)
                >> magic.right_shift) as usize,
    ) & square.get_rook_rays_bb()
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
fn get_rook_moves_bmi(square: Square, blockers: BitBoard) -> BitBoard {
    let bmi2_magic = *get_item_unchecked!(const BISHOP_AND_ROOK_BMI_MASKS[1], square.to_index());
    let index = unsafe { _pext_u64(blockers.into_inner(), bmi2_magic.blockers_mask.into_inner()) }
        as usize
        + bmi2_magic.offset;
    let result = unsafe {
        _pdep_u64(
            *BMI_MOVES.get_unchecked(index) as u64,
            square.get_rook_rays_bb().into_inner(),
        )
    };
    return BitBoard::new(result);
}

/// Get the moves for a bishop on a particular square, given blockers blocking my movement.
#[inline]
pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    #[cfg(target_feature = "bmi2")]
    {
        get_bishop_moves_bmi(square, blockers)
    }
    #[cfg(not(target_feature = "bmi2"))]
    {
        get_bishop_moves_non_bmi(square, blockers)
    }
}

/// Get the moves for a rook on a particular square, given blockers blocking my movement.
#[inline]
pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    #[cfg(target_feature = "bmi2")]
    {
        get_rook_moves_bmi(square, blockers)
    }
    #[cfg(not(target_feature = "bmi2"))]
    {
        get_rook_moves_non_bmi(square, blockers)
    }
}

#[inline]
pub fn get_queen_moves(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_moves(square, blockers) ^ get_bishop_moves(square, blockers)
}

/// Get the legal destination castle squares for both players
#[inline]
pub fn get_castle_moves() -> BitBoard {
    const { BitBoard::new(6052837899185946708) }
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
    pub const fn from_int(int: u8) -> Self {
        // TODO: Maybe consider getting it from ALL_SQUARES? Is it faster?
        unsafe { std::mem::transmute(int & 63) }
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
    pub const fn get_rank(self) -> Rank {
        Rank::from_index(self.to_index() >> 3)
    }

    #[inline]
    pub fn get_rank_bb(self) -> BitBoard {
        *get_item_unchecked!(BB_RANKS, self.to_index() >> 3)
    }

    #[inline]
    pub const fn get_file(self) -> File {
        File::from_index(self.to_index() & 7)
    }

    #[inline]
    pub fn get_file_bb(self) -> BitBoard {
        *get_item_unchecked!(BB_FILES, self.to_index() & 7)
    }

    #[inline]
    pub const fn up(self) -> Option<Square> {
        if let Some(rank) = self.get_rank().up() {
            Some(Square::from_rank_and_file(rank, self.get_file()))
        } else {
            None
        }
    }

    #[inline]
    pub const fn up_left(self) -> Option<Square> {
        if let Some(square) = self.up() {
            square.left()
        } else {
            None
        }
    }

    #[inline]
    pub const fn up_right(self) -> Option<Square> {
        if let Some(square) = self.up() {
            square.right()
        } else {
            None
        }
    }

    #[inline]
    pub const fn down(self) -> Option<Square> {
        if let Some(rank) = self.get_rank().down() {
            Some(Square::from_rank_and_file(rank, self.get_file()))
        } else {
            None
        }
    }

    #[inline]
    pub const fn down_left(self) -> Option<Square> {
        if let Some(square) = self.down() {
            square.left()
        } else {
            None
        }
    }

    #[inline]
    pub const fn down_right(self) -> Option<Square> {
        if let Some(square) = self.down() {
            square.right()
        } else {
            None
        }
    }

    #[inline]
    pub const fn left(self) -> Option<Square> {
        if let Some(file) = self.get_file().left() {
            Some(Square::from_rank_and_file(self.get_rank(), file))
        } else {
            None
        }
    }

    #[inline]
    pub const fn right(self) -> Option<Square> {
        if let Some(file) = self.get_file().right() {
            Some(Square::from_rank_and_file(self.get_rank(), file))
        } else {
            None
        }
    }

    #[inline]
    pub const fn forward(self, color: Color) -> Option<Square> {
        match color {
            White => self.up(),
            Black => self.down(),
        }
    }

    #[inline]
    pub const fn backward(self, color: Color) -> Option<Square> {
        match color {
            White => self.down(),
            Black => self.up(),
        }
    }

    #[inline]
    pub const fn wrapping_up(self) -> Square {
        Square::from_rank_and_file(self.get_rank().wrapping_up(), self.get_file())
    }

    #[inline]
    pub const fn wrapping_down(self) -> Square {
        Square::from_rank_and_file(self.get_rank().wrapping_down(), self.get_file())
    }

    #[inline]
    pub const fn wrapping_left(self) -> Square {
        Square::from_rank_and_file(self.get_rank(), self.get_file().wrapping_left())
    }

    #[inline]
    pub const fn wrapping_right(self) -> Square {
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

    pub const fn manhattan_distance(self, other: Square) -> u8 {
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

    #[inline]
    pub fn vertical_mirror(self) -> Self {
        *get_item_unchecked!(SQUARES_VERTICAL_MIRROR, self.to_index())
    }

    #[inline]
    pub fn horizontal_mirror(self) -> Self {
        *get_item_unchecked!(SQUARES_HORIZONTAL_MIRROR, self.to_index())
    }

    #[inline]
    pub fn rotate(self) -> Self {
        *get_item_unchecked!(SQUARES_ROTATED, self.to_index())
    }

    /// Get a line (extending to infinity, which in chess is 8 squares), given two squares.
    /// This line does extend past the squares.
    #[inline]
    pub fn line(self, other: Square) -> BitBoard {
        *get_item_unchecked!(LINE, self.to_index(), other.to_index())
    }

    /// Get a line between these two squares, not including the squares themselves.
    #[inline]
    pub fn between(self, other: Square) -> BitBoard {
        *get_item_unchecked!(BETWEEN, self.to_index(), other.to_index())
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

    /// Get the king moves for a particular square.
    #[inline]
    pub fn get_king_moves(self) -> BitBoard {
        *get_item_unchecked!(KING_MOVES, self.to_index())
    }

    /// Get the knight moves for a particular square.
    #[inline]
    pub fn get_knight_moves(self) -> BitBoard {
        *get_item_unchecked!(KNIGHT_MOVES, self.to_index())
    }

    /// Get the pawn capture move for a particular square, given the pawn's color and the potential
    /// victims
    #[inline]
    pub fn get_pawn_attacks(self, color: Color, blockers: BitBoard) -> BitBoard {
        *get_item_unchecked!(const PAWN_MOVES_AND_ATTACKS[1], color.to_index(), self.to_index())
            & blockers
    }

    /// Get the quiet pawn moves (non-captures) for a particular square, given the pawn's color and
    /// the potential blocking pieces.
    #[inline]
    pub fn get_pawn_quiets(self, color: Color, blockers: BitBoard) -> BitBoard {
        // TODO: Maybe optimization possible?
        if !(self.to_bitboard().shift_forward(color) & blockers).is_empty() {
            BitBoard::EMPTY
        } else {
            *get_item_unchecked!(const PAWN_MOVES_AND_ATTACKS[0], color.to_index(), self.to_index())
                & !blockers
        }
    }

    /// Get all the pawn moves for a particular square, given the pawn's color and the potential
    /// blocking pieces and victims.
    #[inline]
    pub fn get_pawn_moves(self, color: Color, blockers: BitBoard) -> BitBoard {
        self.get_pawn_attacks(color, blockers) ^ self.get_pawn_quiets(color, blockers)
    }
}

impl FromStr for Square {
    type Err = TimecatError;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() < 2 {
            return Err(TimecatError::InvalidSquareString { s: s.to_string() });
        }
        let ch = s.to_lowercase().chars().collect_vec();
        if !(('a'..='h').contains(&ch[0]) && ('1'..='8').contains(&ch[1])) {
            return Err(TimecatError::InvalidSquareString { s: s.to_string() });
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

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for Square {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(int) = ob.extract::<u8>() {
            return Ok(Self::from_int(int));
        }
        if let Ok(fen) = ob.extract::<&str>() {
            if let Ok(position) = Self::from_str(fen) {
                return Ok(position);
            }
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
