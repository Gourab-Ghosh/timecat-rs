use super::*;

#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct BitBoard(u64);

impl BitBoard {
    #[inline]
    pub const fn new(bb: u64) -> Self {
        Self(bb)
    }

    #[inline]
    pub fn from_array(array: [u64; 8]) -> Self {
        Self(
            array
                .into_iter()
                .rev()
                .enumerate()
                .map(|(i, bb)| bb << (i << 3))
                .fold(0, |acc, bb| acc ^ bb),
        )
    }

    #[inline]
    pub fn set_mask(&mut self, mask: u64) {
        self.0 = mask;
    }

    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Self(1 << ((rank.to_int() << 3) | file.to_int()))
    }

    #[inline]
    pub const fn popcnt(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub const fn reverse_colors(self) -> Self {
        Self(self.0.swap_bytes())
    }

    #[inline]
    pub const fn to_square_index_unchecked(self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline]
    pub fn to_square_unchecked(self) -> Square {
        *get_item_unchecked!(ALL_SQUARES, self.to_square_index_unchecked())
    }

    #[inline]
    pub const fn to_square_index(self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.to_square_index_unchecked())
        }
    }

    #[inline]
    pub fn to_square(self) -> Option<Square> {
        Some(*get_item_unchecked!(ALL_SQUARES, self.to_square_index()?))
    }

    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#FlipVertically>
    pub const fn flip_vertical(self) -> Self {
        let mut bb = self.0;
        bb = ((bb >> 8) & 0x00FF_00FF_00FF_00FF) | ((bb & 0x00FF_00FF_00FF_00FF) << 8);
        bb = ((bb >> 16) & 0x0000_FFFF_0000_FFFF) | ((bb & 0x0000_FFFF_0000_FFFF) << 16);
        bb = (bb >> 32) | ((bb & 0x0000_0000_FFFF_FFFF) << 32);
        Self(bb)
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#MirrorHorizontally>
    pub const fn flip_horizontal(self) -> Self {
        let mut bb = self.0;
        bb = ((bb >> 1) & 0x5555_5555_5555_5555) | ((bb & 0x5555_5555_5555_5555) << 1);
        bb = ((bb >> 2) & 0x3333_3333_3333_3333) | ((bb & 0x3333_3333_3333_3333) << 2);
        bb = ((bb >> 4) & 0x0F0F_0F0F_0F0F_0F0F) | ((bb & 0x0F0F_0F0F_0F0F_0F0F) << 4);
        Self(bb)
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#FlipabouttheDiagonal>
    pub const fn flip_diagonal(self) -> Self {
        let mut bb = self.0;
        let mut t = (bb ^ (bb << 28)) & 0x0F0F_0F0F_0000_0000;
        bb = bb ^ t ^ (t >> 28);
        t = (bb ^ (bb << 14)) & 0x3333_0000_3333_0000;
        bb = bb ^ t ^ (t >> 14);
        t = (bb ^ (bb << 7)) & 0x5500_5500_5500_5500;
        bb = bb ^ t ^ (t >> 7);
        Self(bb)
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#FlipabouttheAntidiagonal>
    pub const fn flip_anti_diagonal(self) -> Self {
        let mut bb = self.0;
        let mut t = bb ^ (bb << 36);
        bb = bb ^ ((t ^ (bb >> 36)) & 0xF0F0_F0F0_0F0F_0F0F);
        t = (bb ^ (bb << 18)) & 0xCCCC_0000_CCCC_0000;
        bb = bb ^ t ^ (t >> 18);
        t = (bb ^ (bb << 9)) & 0xAA00_AA00_AA00_AA00;
        bb = bb ^ t ^ (t >> 9);
        Self(bb)
    }

    #[inline]
    pub const fn shift_up(self) -> Self {
        Self((self.0 & !BB_RANK_8.0) << 8)
    }

    #[inline]
    pub const fn shift_down(self) -> Self {
        Self(self.0 >> 8)
    }

    #[inline]
    pub const fn shift_left(self) -> Self {
        Self((self.0 & !BB_FILE_A.0) >> 1)
    }

    #[inline]
    pub const fn shift_right(self) -> Self {
        Self((self.0 & !BB_FILE_H.0) << 1)
    }

    pub const fn shift_up_n_times(self, n: usize) -> Self {
        if n > 7 {
            return BB_EMPTY;
        }
        let mut bb = self.0;
        let mut i = 0;
        while i < n {
            bb = (bb & !BB_RANK_8.0) << 8;
            i += 1;
        }
        Self(bb)
    }

    pub const fn shift_down_n_times(self, n: usize) -> Self {
        if n > 7 {
            return BB_EMPTY;
        }
        let mut bb = self.0;
        let mut i = 0;
        while i < n {
            bb = bb >> 8;
            i += 1;
        }
        Self(bb)
    }

    pub const fn shift_left_n_times(self, n: usize) -> Self {
        if n > 7 {
            return BB_EMPTY;
        }
        let mut bb = self.0;
        let mut i = 0;
        while i < n {
            bb = (bb & !BB_FILE_A.0) >> 1;
            i += 1;
        }
        Self(bb)
    }

    pub const fn shift_right_n_times(self, n: usize) -> Self {
        if n > 7 {
            return BB_EMPTY;
        }
        let mut bb = self.0;
        let mut i = 0;
        while i < n {
            bb = (bb & !BB_FILE_H.0) << 1;
            i += 1;
        }
        Self(bb)
    }

    #[inline]
    pub fn contains(self, square: Square) -> bool {
        !(self & square.to_bitboard()).is_empty()
    }

    #[inline]
    pub const fn into_inner(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

macro_rules! implement_u64_methods {
    ($($visibility:vis const fn $function:ident(self $(, $argument:ident: $argument_type:ty)* $(,)?) -> $return_type:ty),* $(,)?) => {
        impl BitBoard {
            $(
                #[inline]
                $visibility const fn $function(&self, $($argument: $argument_type),*) -> $return_type {
                    BitBoard(self.0.$function($($argument),*))
                }
            )*
        }
    };
}

implement_u64_methods!(
    pub const fn wrapping_shl(self, rhs: u32) -> Self,
    pub const fn wrapping_shr(self, rhs: u32) -> Self,
);

macro_rules! implement_bitwise_operations {
    ($direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident) => {
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u128);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, usize);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u64);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i128);

        impl $assign_trait<&BitBoard> for BitBoard {
            #[inline]
            fn $assign_func(&mut self, rhs: &Self) {
                self.$assign_func(rhs.0)
            }
        }

        impl $assign_trait for BitBoard {
            #[inline]
            fn $assign_func(&mut self, rhs: Self) {
                self.$assign_func(&rhs)
            }
        }

        impl $direct_trait for &BitBoard {
            type Output = BitBoard;

            #[inline]
            fn $direct_func(self, rhs: Self) -> Self::Output {
                self.$direct_func(rhs.0)
            }
        }

        impl $direct_trait for BitBoard {
            type Output = Self;

            #[inline]
            fn $direct_func(self, rhs: Self) -> Self::Output {
                (&self).$direct_func(&rhs)
            }
        }

        impl $direct_trait<BitBoard> for &BitBoard {
            type Output = BitBoard;

            #[inline]
            fn $direct_func(self, rhs: BitBoard) -> Self::Output {
                self.$direct_func(&rhs)
            }
        }

        impl $direct_trait<&BitBoard> for BitBoard {
            type Output = Self;

            #[inline]
            fn $direct_func(self, rhs: &Self) -> Self::Output {
                (&self).$direct_func(rhs)
            }
        }
    };

    (@bit_shifting $direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident) => {
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u32);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u16);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u8);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, isize);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i64);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i32);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i16);
        implement_bitwise_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i8);
    };

    (@integer_implementation $direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident, $int_type: ident) => {
        impl $assign_trait<$int_type> for BitBoard {
            #[inline]
            fn $assign_func(&mut self, rhs: $int_type) {
                self.0 = self.0.$direct_func(rhs as u64)
            }
        }

        impl $assign_trait<&$int_type> for BitBoard {
            #[inline]
            fn $assign_func(&mut self, rhs: &$int_type) {
                self.$assign_func(*rhs)
            }
        }

        impl $direct_trait<$int_type> for BitBoard {
            type Output = Self;

            #[inline]
            fn $direct_func(self, rhs: $int_type) -> Self::Output {
                Self(self.0.$direct_func(rhs as u64))
            }
        }

        impl $direct_trait<&$int_type> for BitBoard {
            type Output = Self;

            #[inline]
            fn $direct_func(self, rhs: &$int_type) -> Self::Output {
                self.$direct_func(*rhs)
            }
        }

        impl $direct_trait<&$int_type> for &BitBoard {
            type Output = BitBoard;

            #[inline]
            fn $direct_func(self, rhs: &$int_type) -> Self::Output {
                (*self).$direct_func(rhs)
            }
        }

        impl $direct_trait<$int_type> for &BitBoard {
            type Output = BitBoard;

            #[inline]
            fn $direct_func(self, rhs: $int_type) -> Self::Output {
                (*self).$direct_func(rhs)
            }
        }

        impl $assign_trait<&BitBoard> for $int_type {
            #[inline]
            fn $assign_func(&mut self, rhs: &BitBoard) {
                self.$assign_func(rhs.0 as $int_type)
            }
        }

        impl $assign_trait<BitBoard> for $int_type {
            #[inline]
            fn $assign_func(&mut self, rhs: BitBoard) {
                self.$assign_func(&rhs)
            }
        }

        impl $direct_trait<&BitBoard> for $int_type {
            type Output = $int_type;

            #[inline]
            fn $direct_func(mut self, rhs: &BitBoard) -> Self::Output {
                self.$assign_func(rhs);
                self
            }
        }

        impl $direct_trait<BitBoard> for $int_type {
            type Output = $int_type;

            #[inline]
            fn $direct_func(self, rhs: BitBoard) -> Self::Output {
                self.$direct_func(&rhs)
            }
        }

        impl $direct_trait<&BitBoard> for &$int_type {
            type Output = $int_type;

            #[inline]
            fn $direct_func(self, rhs: &BitBoard) -> Self::Output {
                (*self).$direct_func(rhs)
            }
        }

        impl $direct_trait<BitBoard> for &$int_type {
            type Output = $int_type;

            #[inline]
            fn $direct_func(self, rhs: BitBoard) -> Self::Output {
                self.$direct_func(&rhs)
            }
        }
    };
}

implement_bitwise_operations!(BitAnd, BitAndAssign, bitand, bitand_assign);
implement_bitwise_operations!(BitOr, BitOrAssign, bitor, bitor_assign);
implement_bitwise_operations!(BitXor, BitXorAssign, bitxor, bitxor_assign);
implement_bitwise_operations!(Mul, MulAssign, mul, mul_assign);
implement_bitwise_operations!(Shl, ShlAssign, shl, shl_assign);
implement_bitwise_operations!(Shr, ShrAssign, shr, shr_assign);
implement_bitwise_operations!(@bit_shifting Shl, ShlAssign, shl, shl_assign);
implement_bitwise_operations!(@bit_shifting Shr, ShrAssign, shr, shr_assign);

impl Not for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        !&self
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        let square_index = self.to_square_index()?;
        let square = Square::from_index(square_index);
        self.0 ^= 1 << square_index;
        Some(square)
    }
}

impl fmt::Display for BitBoard {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut skeleton = get_board_skeleton();
        let occupied_symbol = "X".colorize(BITBOARD_OCCUPIED_SQUARE_STYLE);
        for square in SQUARES_HORIZONTAL_MIRROR {
            skeleton = skeleton.replacen(
                'O',
                if self.contains(square) {
                    &occupied_symbol
                } else {
                    " "
                },
                1,
            );
        }
        write!(f, "{skeleton}")
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for BitBoard {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(int) = ob.extract::<u64>() {
            return Ok(Self(int));
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
