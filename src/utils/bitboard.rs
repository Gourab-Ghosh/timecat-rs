use super::*;

#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct BitBoard(u64);

macro_rules! generate_shift_functions {
    ($func: ident, $operator: tt, $empty_side: ident) => {
        #[inline]
        pub fn $func(self, n: u8) -> Self {
            if n > 7 {
                BitBoard::EMPTY
            } else {
                (self $operator n)
                    & get_item_unchecked!(
                        const {
                            let mut array = [BitBoard::ALL; 8];
                            let mut i = 1;
                            while i < 8 {
                                array[i].0 = array[i - 1].0 & !($empty_side.0 $operator (i - 1));
                                i += 1;
                            }
                            array
                        },
                        n as usize,
                    )
            }
        }
    };
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self::new(0xFFFFFFFFFFFFFFFF);

    #[inline]
    pub const fn new(bb: u64) -> Self {
        unsafe { std::mem::transmute(bb) }
    }

    #[inline]
    pub const fn set_mask(&mut self, mask: u64) {
        self.0 = mask;
    }

    /// Generates the bitboard of the square at the intersection of the given rank and file.
    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Self::new(1 << ((rank.to_int() << 3) ^ file.to_int()))
    }

    #[inline]
    pub const fn popcnt(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub const fn reverse_colors(self) -> Self {
        Self::new(self.0.swap_bytes())
    }

    #[inline]
    pub const unsafe fn to_square_index_unchecked(self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline]
    pub const unsafe fn to_square_unchecked(self) -> Square {
        Square::from_index(self.to_square_index_unchecked())
    }

    #[inline]
    pub const fn to_square_index(self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.to_square_index_unchecked() })
        }
    }

    #[inline]
    pub const fn to_square(self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.to_square_unchecked() })
        }
    }

    #[inline]
    pub const fn xor_square(&mut self, square: Square) {
        self.0 ^= square.to_bitboard().0;
    }

    #[inline]
    pub const fn remove_square(&mut self, square: Square) {
        self.0 &= !square.to_bitboard().0;
    }

    pub const unsafe fn pop_square_unchecked(&mut self) -> Square {
        let square = self.to_square_unchecked();
        self.xor_square(square);
        square
    }

    #[inline]
    pub const fn pop_square(&mut self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.pop_square_unchecked() })
        }
    }

    #[inline]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_mul(rhs.0))
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == const { Self::EMPTY.0 }
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#FlipVertically>
    pub const fn flip_vertical(self) -> Self {
        let mut bb = self.0;
        bb = ((bb >> 8) & 0x00FF_00FF_00FF_00FF) | ((bb & 0x00FF_00FF_00FF_00FF) << 8);
        bb = ((bb >> 16) & 0x0000_FFFF_0000_FFFF) | ((bb & 0x0000_FFFF_0000_FFFF) << 16);
        bb = (bb >> 32) | ((bb & 0x0000_0000_FFFF_FFFF) << 32);
        Self::new(bb)
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#MirrorHorizontally>
    pub const fn flip_horizontal(self) -> Self {
        let mut bb = self.0;
        bb = ((bb >> 1) & 0x5555_5555_5555_5555) | ((bb & 0x5555_5555_5555_5555) << 1);
        bb = ((bb >> 2) & 0x3333_3333_3333_3333) | ((bb & 0x3333_3333_3333_3333) << 2);
        bb = ((bb >> 4) & 0x0F0F_0F0F_0F0F_0F0F) | ((bb & 0x0F0F_0F0F_0F0F_0F0F) << 4);
        Self::new(bb)
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
        Self::new(bb)
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
        Self::new(bb)
    }

    #[inline]
    pub const fn shift_up_n_times(self, n: u8) -> Self {
        if n > 7 {
            Self::EMPTY
        } else {
            Self::new(self.0 << (n << 3))
        }
    }

    #[inline]
    pub const fn shift_down_n_times(self, n: u8) -> Self {
        if n > 7 {
            Self::EMPTY
        } else {
            Self::new(self.0 >> (n << 3))
        }
    }

    generate_shift_functions!(shift_left_n_times, >>, BB_FILE_H);
    generate_shift_functions!(shift_right_n_times, <<, BB_FILE_A);

    #[inline]
    pub const fn shift_up(self) -> Self {
        self.shift_up_n_times(1)
    }

    #[inline]
    pub const fn shift_down(self) -> Self {
        self.shift_down_n_times(1)
    }

    #[inline]
    pub const fn shift_left(self) -> Self {
        Self::new((self.0 & const { !BB_FILE_A.0 }) >> 1)
    }

    #[inline]
    pub const fn shift_right(self) -> Self {
        Self::new((self.0 & const { !BB_FILE_H.0 }) << 1)
    }

    #[inline]
    pub const fn shift_forward_n_times(self, color: Color, n: u8) -> Self {
        match color {
            White => self.shift_up_n_times(n),
            Black => self.shift_down_n_times(n),
        }
    }

    #[inline]
    pub const fn shift_backward_n_times(self, color: Color, n: u8) -> Self {
        match color {
            White => self.shift_down_n_times(n),
            Black => self.shift_up_n_times(n),
        }
    }

    #[inline]
    pub const fn shift_forward(self, color: Color) -> Self {
        self.shift_forward_n_times(color, 1)
    }

    #[inline]
    pub const fn shift_backward(self, color: Color) -> Self {
        self.shift_backward_n_times(color, 1)
    }

    #[inline]
    pub const fn contains(self, square: Square) -> bool {
        !Self::new(self.0 & square.to_bitboard().0).is_empty()
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

impl From<&BitBoard> for u64 {
    #[inline]
    fn from(value: &BitBoard) -> Self {
        value.0
    }
}

impl From<BitBoard> for u64 {
    #[inline]
    fn from(value: BitBoard) -> Self {
        (&value).into()
    }
}

impl From<u64> for BitBoard {
    #[inline]
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<&u64> for BitBoard {
    #[inline]
    fn from(value: &u64) -> Self {
        (*value).into()
    }
}

macro_rules! implement_u64_methods {
    ($($visibility:vis const fn $function:ident(self $(, $argument:ident: $argument_type:ty)* $(,)?) -> $return_type:ty),* $(,)?) => {
        impl BitBoard {
            $(
                #[inline]
                $visibility const fn $function(&self, $($argument: $argument_type),*) -> $return_type {
                    Self::new(self.0.$function($($argument),*))
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
    (@bit_shifting $direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident) => {
        impl<T> $assign_trait<T> for BitBoard where u64: $assign_trait<T> {

            #[inline]
            fn $assign_func(&mut self, rhs: T) {
                self.0.$assign_func(rhs)
            }
        }

        impl<T> $direct_trait<T> for BitBoard where Self: $assign_trait<T> {
            type Output = Self;

            #[inline]
            fn $direct_func(mut self, rhs: T) -> Self::Output {
                self.$assign_func(rhs);
                self
            }
        }

        impl<T> $direct_trait<T> for &BitBoard where BitBoard: $direct_trait<T> {
            type Output = <BitBoard as $direct_trait<T>>::Output;

            #[inline]
            fn $direct_func(self, rhs: T) -> Self::Output {
                (*self).$direct_func(rhs)
            }
        }
    };

    ($direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident) => {
        implement_bitwise_operations!(@bigger_integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u128);
        implement_bitwise_operations!(@bigger_integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u64);
        implement_bitwise_operations!(@bigger_integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i128);

        impl<T> $assign_trait<T> for BitBoard where u64: From<T> {

            #[inline]
            fn $assign_func(&mut self, rhs: T) {
                self.0.$assign_func(u64::from(rhs))
            }
        }

        impl<T> $direct_trait<T> for BitBoard where u64: From<T> {
            type Output = Self;

            #[inline]
            fn $direct_func(mut self, rhs: T) -> Self::Output {
                self.$assign_func(rhs);
                self
            }
        }

        impl<T> $direct_trait<T> for &BitBoard where u64: From<T> {
            type Output = BitBoard;

            #[inline]
            fn $direct_func(self, rhs: T) -> Self::Output {
                (*self).$direct_func(rhs)
            }
        }
    };

    (@bigger_integer_implementation $direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident, $int_type: ident) => {
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
implement_bitwise_operations!(@bit_shifting Shl, ShlAssign, shl, shl_assign);
implement_bitwise_operations!(@bit_shifting Shr, ShrAssign, shr, shr_assign);

impl Not for &BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard::new(!self.0)
    }
}

impl Not for BitBoard {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        !&self
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        self.pop_square()
    }
}

impl fmt::Display for BitBoard {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let occupied_symbol = "X".colorize(BITBOARD_OCCUPIED_SQUARE_STYLE);
        write!(
            f,
            "{}",
            get_board_string(true, |square| if self.contains(square) {
                occupied_symbol.as_str().into()
            } else {
                Cow::Borrowed(" ")
            })
        )
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for BitBoard {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(int) = ob.extract::<u64>() {
            return Ok(Self::new(int));
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
