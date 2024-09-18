use super::*;
pub use Square::*;

const PAWN_MOVES_AND_ATTACKS: [[[BitBoard; 64]; 2]; 2] = {
    let mut moves_array = [[BB_EMPTY; 64]; 2];
    let mut attacks_array = [[BB_EMPTY; 64]; 2];
    let mut i = 0;
    while i < NUM_COLORS {
        let mut j = 0;
        while j < NUM_SQUARES {
            moves_array[i][j] = if i == 0 {
                BB_SQUARES[j].shift_up()
            } else {
                BB_SQUARES[j].shift_down()
            };
            attacks_array[i][j] = bb_xor(
                moves_array[i][j].shift_left(),
                moves_array[i][j].shift_right(),
            );
            j += 1;
        }
        i += 1;
    }
    i = 0;
    while i < 8 {
        moves_array[0][i + 8] = bb_xor(moves_array[0][i + 8], moves_array[0][i + 8].shift_up());
        moves_array[1][i + 48] =
            bb_xor(moves_array[1][i + 48], moves_array[1][i + 48].shift_down());
        i += 1;
    }
    [moves_array, attacks_array]
};

const KING_MOVES: [BitBoard; 64] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        let square_bb = BB_SQUARES[index];
        array[index] = BitBoard::new(
            square_bb.shift_up().into_inner()
                ^ square_bb.shift_down().into_inner()
                ^ square_bb.shift_left().into_inner()
                ^ square_bb.shift_right().into_inner()
                ^ square_bb.shift_up().shift_left().into_inner()
                ^ square_bb.shift_up().shift_right().into_inner()
                ^ square_bb.shift_down().shift_left().into_inner()
                ^ square_bb.shift_down().shift_right().into_inner(),
        );
        index += 1;
    }
    array
};

const KNIGHT_MOVES: [BitBoard; 64] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        let square_bb = BB_SQUARES[index];
        array[index] = BitBoard::new(
            square_bb.shift_up_n_times(2).shift_left().into_inner()
                ^ square_bb.shift_up_n_times(2).shift_right().into_inner()
                ^ square_bb.shift_down_n_times(2).shift_left().into_inner()
                ^ square_bb.shift_down_n_times(2).shift_right().into_inner()
                ^ square_bb.shift_left_n_times(2).shift_up().into_inner()
                ^ square_bb.shift_left_n_times(2).shift_down().into_inner()
                ^ square_bb.shift_right_n_times(2).shift_up().into_inner()
                ^ square_bb.shift_right_n_times(2).shift_down().into_inner(),
        );
        index += 1;
    }
    array
};

const BISHOP_DIAGONAL_RAYS: [BitBoard; NUM_SQUARES] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        let square = Square::from_index(index);
        let square_rank_index = square.get_rank().to_int();
        let square_file_index = square.get_file().to_int();
        let rank_file_diff = square_rank_index.abs_diff(square_file_index);
        array[index] = if square_rank_index < square_file_index {
            DIAGONAL_RAY.shift_right_n_times(rank_file_diff)
        } else if square_rank_index > square_file_index {
            DIAGONAL_RAY.shift_left_n_times(rank_file_diff)
        } else {
            DIAGONAL_RAY
        };
        index += 1;
    }
    array
};

const BISHOP_ANTI_DIAGONAL_RAYS: [BitBoard; NUM_SQUARES] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        array[SQUARES_VERTICAL_MIRROR[index].to_index()] =
            BISHOP_DIAGONAL_RAYS[index].flip_horizontal();
        index += 1;
    }
    array
};

const BISHOP_RAYS: [BitBoard; NUM_SQUARES] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        array[index] = bb_xor(
            BISHOP_DIAGONAL_RAYS[index],
            BISHOP_ANTI_DIAGONAL_RAYS[index],
        );
        index += 1;
    }
    array
};

const ROOK_RAYS: [BitBoard; NUM_SQUARES] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        let square = Square::from_index(index);
        array[index] = bb_xor(
            BB_RANKS[square.get_rank().to_index()],
            BB_FILES[square.get_file().to_index()],
        );
        index += 1;
    }
    array
};

const ALL_DIRECTION_RAYS: [BitBoard; NUM_SQUARES] = {
    let mut array = [BB_EMPTY; NUM_SQUARES];
    let mut index = 0;
    while index < NUM_SQUARES {
        array[index] = bb_xor(ROOK_RAYS[index], BISHOP_RAYS[index]);
        index += 1;
    }
    array
};

pub(crate) const BETWEEN: [[BitBoard; NUM_SQUARES]; NUM_SQUARES] = {
    const fn calculate_between(square1: Square, square2: Square) -> BitBoard {
        if !bb_contains(ALL_DIRECTION_RAYS[square1.to_index()], square2) {
            return BB_EMPTY;
        }

        let square1_rank = square1.get_rank();
        let square1_file = square1.get_file();
        let square2_rank = square2.get_rank();
        let square2_file = square2.get_file();

        let rank_ordering = u8_cmp(square1_rank.to_int(), square2_rank.to_int());
        let file_ordering = u8_cmp(square1_file.to_int(), square2_file.to_int());

        let mut bb = BB_EMPTY;
        let mut current_square = square1;
        loop {
            let mut next_square = match rank_ordering {
                Ordering::Less => current_square.wrapping_up(),
                Ordering::Equal => current_square,
                Ordering::Greater => current_square.wrapping_down(),
            };
            next_square = match file_ordering {
                Ordering::Less => next_square.wrapping_right(),
                Ordering::Equal => next_square,
                Ordering::Greater => next_square.wrapping_left(),
            };
            if square_eq(next_square, square2) {
                return bb;
            }
            bb = bb_xor(bb, square_to_bitboard(next_square));
            current_square = next_square;
        }
    }

    let mut array = [[BB_EMPTY; NUM_SQUARES]; NUM_SQUARES];
    let mut i = 0;
    while i < NUM_SQUARES {
        let mut j = 0;
        while j < i {
            array[i][j] = calculate_between(Square::from_index(i), Square::from_index(j));
            array[j][i] = array[i][j];
            j += 1;
        }
        i += 1;
    }
    array
};

const LINE: [[BitBoard; NUM_SQUARES]; NUM_SQUARES] = {
    const fn calculate_line(square1: Square, square2: Square) -> BitBoard {
        if !bb_contains(ALL_DIRECTION_RAYS[square1.to_index()], square2) {
            return BB_EMPTY;
        }

        let square2_bb = BB_SQUARES[square2.to_index()];
        let mut possible_line = BB_RANKS[square1.get_rank().to_index()];
        if !bb_and(possible_line, square2_bb).is_empty() {
            return possible_line;
        }
        possible_line = BB_FILES[square1.get_file().to_index()];
        if !bb_and(possible_line, square2_bb).is_empty() {
            return possible_line;
        }
        possible_line = BISHOP_DIAGONAL_RAYS[square1.to_index()];
        if !bb_and(possible_line, square2_bb).is_empty() {
            return possible_line;
        }
        possible_line = BISHOP_ANTI_DIAGONAL_RAYS[square1.to_index()];
        if !bb_and(possible_line, square2_bb).is_empty() {
            return possible_line;
        }

        unreachable!();
    }

    let mut array = [[BB_EMPTY; NUM_SQUARES]; NUM_SQUARES];
    let mut i = 0;
    while i < NUM_SQUARES {
        let mut j = 0;
        while j < i {
            array[i][j] = calculate_line(Square::from_index(i), Square::from_index(j));
            array[j][i] = array[i][j];
            j += 1;
        }
        i += 1;
    }
    array
};

// const MOVES: [BitBoard; 0] = {
//     const fn generate_rook_mask(square: Square) -> BitBoard {
//         let mut to_remove = BB_CORNERS;
//         match square.get_rank() {
//             Rank::First => to_remove = bb_xor(to_remove, BB_RANK_1),
//             Rank::Eighth => to_remove = bb_xor(to_remove, BB_RANK_8),
//             _ => (),
//         }
//         // TODO: Check of xor works here!
//         match square.get_file() {
//             File::A => to_remove = bb_xor(to_remove, BB_FILE_A),
//             File::H => to_remove = bb_xor(to_remove, BB_FILE_H),
//             _ => (),
//         }
//         bb_and(ROOK_RAYS[square.to_index()], bb_not(to_remove))
//     }

//     let mut bishop_masks = [BB_EMPTY; 64];
//     let mut rook_masks = [BB_EMPTY; 64];
//     {
//         let mut i = 0;
//         while i < 64 {
//             bishop_masks[i] = bb_and(BISHOP_RAYS[i], bb_not(BB_CORNERS));
//             rook_masks[i] = generate_rook_mask(Square::from_index(i));
//             i += 1;
//         }
//     }

//     []
// };

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
            BB_EMPTY
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
