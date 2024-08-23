// use super::*;

// pub type ScoreType = i16;

// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
// #[repr(transparent)]
// pub struct Score(ScoreType);

// impl Score {
//     pub const ZERO: Self = Self::new(0);

//     pub const fn new(value: ScoreType) -> Self {
//         Self(value)
//     }
// }

// impl Deref for Score {
//     type Target = ScoreType;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for Score {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl Neg for Score {
//     type Output = Self;

//     fn neg(self) -> Self::Output {
//         -self.0.into()
//     }
// }

// impl From<ScoreType> for Score {
//     fn from(value: ScoreType) -> Self {
//         Self::new(value)
//     }
// }

// macro_rules! implement_operations {
//     ($direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident) => {
//         // implement_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u128);
//         // implement_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, usize);
//         // implement_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, u64);
//         // implement_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, i128);
//         implement_operations!(@integer_implementation $direct_trait, $assign_trait, $direct_func, $assign_func, ScoreType);

//         impl $assign_trait<&Score> for Score {
//             #[inline]
//             fn $assign_func(&mut self, rhs: &Self) {
//                 self.$assign_func(rhs.0)
//             }
//         }

//         impl $assign_trait for Score {
//             #[inline]
//             fn $assign_func(&mut self, rhs: Self) {
//                 self.$assign_func(&rhs)
//             }
//         }

//         impl $direct_trait for &Score {
//             type Output = Score;

//             #[inline]
//             fn $direct_func(self, rhs: Self) -> Self::Output {
//                 self.$direct_func(rhs.0)
//             }
//         }

//         impl $direct_trait for Score {
//             type Output = Self;

//             #[inline]
//             fn $direct_func(self, rhs: Self) -> Self::Output {
//                 (&self).$direct_func(&rhs)
//             }
//         }

//         impl $direct_trait<Score> for &Score {
//             type Output = Score;

//             #[inline]
//             fn $direct_func(self, rhs: Score) -> Self::Output {
//                 self.$direct_func(&rhs)
//             }
//         }

//         impl $direct_trait<&Score> for Score {
//             type Output = Self;

//             #[inline]
//             fn $direct_func(self, rhs: &Self) -> Self::Output {
//                 (&self).$direct_func(rhs)
//             }
//         }
//     };

//     (@integer_implementation $direct_trait: ident, $assign_trait: ident, $direct_func: ident, $assign_func: ident, $int_type: ident) => {
//         impl $assign_trait<$int_type> for Score {
//             #[inline]
//             fn $assign_func(&mut self, rhs: $int_type) {
//                 self.0 = self.0.$direct_func(rhs as u64)
//             }
//         }

//         impl $assign_trait<&$int_type> for Score {
//             #[inline]
//             fn $assign_func(&mut self, rhs: &$int_type) {
//                 self.$assign_func(*rhs)
//             }
//         }

//         impl $direct_trait<$int_type> for Score {
//             type Output = Self;

//             #[inline]
//             fn $direct_func(self, rhs: $int_type) -> Self::Output {
//                 Self(self.0.$direct_func(rhs as u64))
//             }
//         }

//         impl $direct_trait<&$int_type> for Score {
//             type Output = Self;

//             #[inline]
//             fn $direct_func(self, rhs: &$int_type) -> Self::Output {
//                 self.$direct_func(*rhs)
//             }
//         }

//         impl $direct_trait<&$int_type> for &Score {
//             type Output = Score;

//             #[inline]
//             fn $direct_func(self, rhs: &$int_type) -> Self::Output {
//                 (*self).$direct_func(rhs)
//             }
//         }

//         impl $direct_trait<$int_type> for &Score {
//             type Output = Score;

//             #[inline]
//             fn $direct_func(self, rhs: $int_type) -> Self::Output {
//                 (*self).$direct_func(rhs)
//             }
//         }

//         impl $assign_trait<&Score> for $int_type {
//             #[inline]
//             fn $assign_func(&mut self, rhs: &Score) {
//                 self.$assign_func(rhs.0 as $int_type)
//             }
//         }

//         impl $assign_trait<Score> for $int_type {
//             #[inline]
//             fn $assign_func(&mut self, rhs: Score) {
//                 self.$assign_func(&rhs)
//             }
//         }

//         impl $direct_trait<&Score> for $int_type {
//             type Output = $int_type;

//             #[inline]
//             fn $direct_func(mut self, rhs: &Score) -> Self::Output {
//                 self.$assign_func(rhs);
//                 self
//             }
//         }

//         impl $direct_trait<Score> for $int_type {
//             type Output = $int_type;

//             #[inline]
//             fn $direct_func(self, rhs: Score) -> Self::Output {
//                 self.$direct_func(&rhs)
//             }
//         }

//         impl $direct_trait<&Score> for &$int_type {
//             type Output = $int_type;

//             #[inline]
//             fn $direct_func(self, rhs: &Score) -> Self::Output {
//                 (*self).$direct_func(rhs)
//             }
//         }

//         impl $direct_trait<Score> for &$int_type {
//             type Output = $int_type;

//             #[inline]
//             fn $direct_func(self, rhs: Score) -> Self::Output {
//                 self.$direct_func(&rhs)
//             }
//         }
//     };
// }

// implement_operations!(Add, AddAssign, add, add_assign);
// implement_operations!(Sub, SubAssign, sub, sub_assign);
// implement_operations!(Mul, MulAssign, mul, mul_assign);
// implement_operations!(Div, DivAssign, div, div_assign);

//////////////////////////////////////////////////////////// Temp Code ////////////////////////////////////////////////////////////

pub type Score = i16;

pub(crate) trait RandomTemporaryTrait {
    const ZERO: Self;
}

impl RandomTemporaryTrait for Score {
    const ZERO: Self = 0;
}
