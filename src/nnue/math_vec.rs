use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MathVec<T, const N: usize> {
    array: [T; N],
}

impl<T: BinRead<Args = ()>, const N: usize> BinRead for MathVec<T, N> {
    type Args = ();

    fn read_options<R: Read + std::io::Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let array: [T; N] = BinRead::read_options(reader, options, ())?;
        Ok(array.into())
    }
}

impl<T, const N: usize> MathVec<T, N> {
    pub const fn new(array: [T; N]) -> Self {
        Self { array }
    }

    pub fn into_inner(self) -> [T; N] {
        self.array
    }
}

impl<T: Copy + Sum, const N: usize> MathVec<T, N> {
    pub fn sum(&self) -> T {
        self.into_iter().sum()
    }
}

impl<T: Clone, const N: usize> MathVec<T, N> {
    pub fn dot<U: From<T> + Mul + Sum<<U as Mul>::Output>>(&self, other: &Self) -> U {
        self.iter()
            .cloned()
            .map_into::<U>()
            .zip(other.iter().cloned().map_into::<U>())
            .map(|(i, j)| i * j)
            .sum()
    }
}

impl<T, const N: usize> From<[T; N]> for MathVec<T, N> {
    fn from(value: [T; N]) -> Self {
        Self::new(value)
    }
}

impl<T, const N: usize> Deref for MathVec<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<T, const N: usize> DerefMut for MathVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

macro_rules! impl_operation {
    (@ops_implementation $trait: ident, $func: ident, $trait_assign: ident, $func_assign: ident) => {
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, i8);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, i16);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, i32);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, i64);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, i128);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, isize);

        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, u8);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, u16);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, u32);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, u64);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, u128);
        impl_operation!(@inner $trait, $func, $trait_assign, $func_assign, usize);

        impl<T: $trait_assign + Clone, const N: usize> $trait_assign<&MathVec<T, N>>
            for MathVec<T, N>
        {
            fn $func_assign(&mut self, rhs: &Self) {
                self.array
                    .iter_mut()
                    .zip(rhs.array.iter().cloned())
                    .for_each(|(i, j)| i.$func_assign(j));
            }
        }

        impl<T: $trait_assign, const N: usize> $trait_assign for MathVec<T, N> {
            fn $func_assign(&mut self, rhs: Self) {
                self.array
                    .iter_mut()
                    .zip(rhs.array)
                    .for_each(|(i, j)| i.$func_assign(j));
            }
        }

        impl<T: $trait_assign + Clone, const N: usize> $trait<&MathVec<T, N>> for MathVec<T, N> {
            type Output = Self;

            fn $func(mut self, rhs: &Self) -> Self {
                self.$func_assign(rhs);
                self
            }
        }

        impl<T: $trait_assign, const N: usize> $trait for MathVec<T, N> {
            type Output = Self;

            fn $func(mut self, rhs: Self) -> Self {
                self.$func_assign(rhs);
                self
            }
        }
    };

    (@zero_implementation $int_type: ty) => {
        impl<const N: usize> MathVec<$int_type, N> {
            pub const ZERO: Self = Self::new([0; N]);
        }
    };

    (@inner $trait: ident, $func: ident, $trait_assign: ident, $func_assign: ident, $int_type: ty) => {
        impl<T: $trait_assign<$int_type>, const N: usize> $trait_assign<$int_type>
            for MathVec<T, N>
        {
            fn $func_assign(&mut self, rhs: $int_type) {
                self.array.iter_mut().for_each(|i| i.$func_assign(rhs));
            }
        }

        impl<T: $trait_assign<$int_type>, const N: usize> $trait<$int_type> for MathVec<T, N> {
            type Output = Self;

            fn $func(mut self, rhs: $int_type) -> Self {
                self.$func_assign(rhs);
                self
            }
        }

        impl<T: Clone, const N: usize> $trait<MathVec<T, N>> for $int_type
        where
            $int_type: $trait<T, Output = T>,
        {
            type Output = MathVec<T, N>;

            fn $func(self, mut rhs: MathVec<T, N>) -> MathVec<T, N> {
                rhs.array
                    .iter_mut()
                    .for_each(|i| *i = self.$func(i.clone()));
                rhs
            }
        }
    };

    // (@ops_implementation $type1: ty, $type2: ty) => {};
}

// BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Neg, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign

impl_operation!(@ops_implementation Add, add, AddAssign, add_assign);
impl_operation!(@ops_implementation Sub, sub, SubAssign, sub_assign);
impl_operation!(@ops_implementation Mul, mul, MulAssign, mul_assign);
impl_operation!(@ops_implementation Div, div, DivAssign, div_assign);
impl_operation!(@ops_implementation BitAnd, bitand, BitAndAssign, bitand_assign);
impl_operation!(@ops_implementation BitOr, bitor, BitOrAssign, bitor_assign);
impl_operation!(@ops_implementation BitXor, bitxor, BitXorAssign, bitxor_assign);
impl_operation!(@ops_implementation Rem, rem, RemAssign, rem_assign);
impl_operation!(@ops_implementation Shl, shl, ShlAssign, shl_assign);
impl_operation!(@ops_implementation Shr, shr, ShrAssign, shr_assign);

impl_operation!(@zero_implementation i8);
impl_operation!(@zero_implementation i16);
impl_operation!(@zero_implementation i32);
impl_operation!(@zero_implementation i64);
impl_operation!(@zero_implementation i128);
impl_operation!(@zero_implementation isize);

impl_operation!(@zero_implementation u8);
impl_operation!(@zero_implementation u16);
impl_operation!(@zero_implementation u32);
impl_operation!(@zero_implementation u64);
impl_operation!(@zero_implementation u128);
impl_operation!(@zero_implementation usize);

impl<T: Neg<Output = T> + Clone, const N: usize> Neg for MathVec<T, N> {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.array.iter_mut().for_each(|x| *x = x.clone().neg());
        self
    }
}

impl<T: fmt::Display, const N: usize> fmt::Display for MathVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.array.iter().map(|x| x.to_string()).join(", ")
        )
    }
}

impl<T: Default + Copy, const N: usize> Default for MathVec<T, N> {
    fn default() -> Self {
        Self {
            array: [T::default(); N],
        }
    }
}

// impl<T, const N: usize> Summable for MathVec<T, N> {}

impl<T, const N: usize> Index<usize> for MathVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.array.index(index)
    }
}

impl<T, const N: usize> IndexMut<usize> for MathVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.array.index_mut(index)
    }
}

macro_rules! impl_clipped_relu {
    ($from: ty, $to: ty) => {
        impl<const N: usize> ClippedRelu<$from, $to, N> for MathVec<$from, N> {
            fn clipped_relu(
                &self,
                scale_by_pow_of_two: $to,
                min: $from,
                max: $from,
            ) -> MathVec<$to, N> {
                let mut v = Vec::with_capacity(N);
                v.extend(
                    self.array
                        .map(|x| (x >> scale_by_pow_of_two).clamp(min, max) as $to),
                );
                MathVec::new(v.try_into().unwrap())
            }

            fn clipped_relu_into(
                &self,
                scale_by_pow_of_two: $to,
                min: $from,
                max: $from,
                output: &mut [$to; N],
            ) {
                self.array
                    .iter()
                    .zip(output.iter_mut())
                    .for_each(|(&i, j)| *j = (i >> scale_by_pow_of_two).clamp(min, max) as $to);
            }
        }
    };

    ($from: ty) => {
        impl_clipped_relu!($from, i8);
        impl_clipped_relu!($from, i16);
        impl_clipped_relu!($from, i32);
        impl_clipped_relu!($from, i64);
        impl_clipped_relu!($from, i128);
        impl_clipped_relu!($from, isize);

        impl_clipped_relu!($from, u8);
        impl_clipped_relu!($from, u16);
        impl_clipped_relu!($from, u32);
        impl_clipped_relu!($from, u64);
        impl_clipped_relu!($from, u128);
        impl_clipped_relu!($from, usize);
    };
}

impl_clipped_relu!(i8);
impl_clipped_relu!(i16);
impl_clipped_relu!(i32);
impl_clipped_relu!(i64);
impl_clipped_relu!(i128);
impl_clipped_relu!(isize);

impl_clipped_relu!(u8);
impl_clipped_relu!(u16);
impl_clipped_relu!(u32);
impl_clipped_relu!(u64);
impl_clipped_relu!(u128);
impl_clipped_relu!(usize);

impl<T, const N: usize> TryFrom<Vec<T>> for MathVec<T, N> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> std::result::Result<Self, Self::Error> {
        let array: [T; N] = value.try_into()?;
        Ok(array.into())
    }
}

impl<T: Clone, U: From<T> + Debug, const N: usize> From<&MathVec<T, N>> for MathVec<U, N> {
    fn from(value: &MathVec<T, N>) -> Self {
        value
            .iter()
            .cloned()
            .map_into()
            .collect_vec()
            .try_into()
            .unwrap()
    }
}
