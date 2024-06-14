use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct BitBoard(u64);

impl BitBoard {
    #[inline]
    pub const fn new(bb: u64) -> Self {
        Self(bb)
    }

    #[inline]
    pub const fn get_mask(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn set_mask(&mut self, mask: u64) {
        self.0 = mask;
    }

    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Self(1 << ((rank.to_int() << 3) + file.to_int()))
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
    pub const fn to_size(self, right_shift: u8) -> usize {
        (self.0 >> right_shift) as usize
    }

    #[inline]
    pub const fn to_square_index(self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline]
    pub fn to_square(self) -> Square {
        *get_item_unchecked!(ALL_SQUARES, self.to_square_index())
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
    #[inline]
    pub const fn flip_vertical(self) -> Self {
        let mut bb = self.0;
        bb = ((bb >> 8) & 0x00ff_00ff_00ff_00ff) | ((bb & 0x00ff_00ff_00ff_00ff) << 8);
        bb = ((bb >> 16) & 0x0000_ffff_0000_ffff) | ((bb & 0x0000_ffff_0000_ffff) << 16);
        bb = (bb >> 32) | ((bb & 0x0000_0000_ffff_ffff) << 32);
        Self::new(bb)
    }

    /// <https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating#MirrorHorizontally>
    #[inline]
    pub const fn flip_horizontal(self) -> Self {
        let mut bb = self.0;
        bb = ((bb >> 1) & 0x5555_5555_5555_5555) | ((bb & 0x5555_5555_5555_5555) << 1);
        bb = ((bb >> 2) & 0x3333_3333_3333_3333) | ((bb & 0x3333_3333_3333_3333) << 2);
        bb = ((bb >> 4) & 0x0f0f_0f0f_0f0f_0f0f) | ((bb & 0x0f0f_0f0f_0f0f_0f0f) << 4);
        Self::new(bb)
    }

    #[inline]
    pub fn contains(self, square: Square) -> bool {
        !(self & square.to_bitboard()).is_empty()
    }
}

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
        if self.0 == 0 {
            None
        } else {
            let square_index = self.to_square_index();
            let square = Square::from_index(square_index);
            self.0 ^= 1 << square_index;
            Some(square)
        }
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
