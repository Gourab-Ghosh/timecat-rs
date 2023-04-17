//!Helper operation traits for NN inference.

//TODO: Add explicit SIMD.

pub trait VecAdd<Rhs = Self> {
    fn vec_add(&mut self, other: &Self);
}

pub trait VecSub<Rhs = Self> {
    fn vec_sub(&mut self, other: &Self);
}

macro_rules! vec_op_fallbacks {
    ($trait:ident, $fn:ident, $op:tt $(, $type:ty)*) => {
        $(impl<const SIZE: usize> $trait for [$type; SIZE] {
            fn $fn(&mut self, other: &Self) {
                for (l, r) in self.iter_mut().zip(other) {
                    *l = l.$op(*r);
                }
            }
        })*
    };
}

macro_rules! vec_add_sub_fallbacks {
    ($($type:ty),*) => {
        vec_op_fallbacks!(VecAdd, vec_add, wrapping_add $(, $type)*);
        vec_op_fallbacks!(VecSub, vec_sub, wrapping_sub $(, $type)*);
    };
}

vec_add_sub_fallbacks!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

pub trait Dot<Rhs=Self> {
    type Output;

    fn dot(&self, other: &Self) -> Self::Output;
}

macro_rules! dot_product_fallbacks {
    ($($type:ty => $out:ty),*) => {
        $(impl<const SIZE: usize> Dot for [$type; SIZE] {
            type Output = $out;

            fn dot(&self, other: &Self) -> Self::Output {
                self.iter().zip(other).map(|(&l, &r)| l as Self::Output * r as Self::Output).sum()
            }
        })*
    };
}

dot_product_fallbacks! {
    i8 => i32,
    i16 => i32,
    i32 => i32,
    i64 => i64
}

pub trait ClippedRelu<O, const SIZE: usize> {
    fn clipped_relu(&self, scale: O, min: O, max: O, out: &mut [O; SIZE]);
}

macro_rules! clipped_relu_fallbacks {
    ($($type:ty => $out:ty),*) => {
        $(impl<const SIZE: usize> ClippedRelu<$out, SIZE> for [$type; SIZE] {
            fn clipped_relu(&self, scale: $out, min: $out, max: $out, out: &mut [$out; SIZE]) {
                for (&v, o) in self.iter().zip(out) {
                    *o = (v >> scale as $type).clamp(min as $type, max as $type) as $out;
                }
            }
        })*
    };
}

clipped_relu_fallbacks! {
    i16 => i8,
    i32 => i8
}
