//!Module for layer types.

use std::ops::AddAssign;
use std::usize;

use crate::nnue_rs::ops::*;

use bytemuck::Zeroable;

///A dense layer.
#[derive(Debug, Clone, Zeroable)]
pub struct Dense<W: Zeroable, B: Zeroable, const INPUTS: usize, const OUTPUTS: usize> {
    pub weights: [[W; INPUTS]; OUTPUTS],
    pub biases: [B; OUTPUTS],
}

impl<
        W: Copy + Zeroable,
        B: Copy + Zeroable + AddAssign + From<<[W; INPUTS] as Dot>::Output>,
        const INPUTS: usize,
        const OUTPUTS: usize,
    > Dense<W, B, INPUTS, OUTPUTS>
where
    [W; INPUTS]: Dot,
{
    pub fn activate(&self, inputs: &[W; INPUTS], outputs: &mut [B; OUTPUTS]) {
        *outputs = self.biases;
        for (o, w) in outputs.iter_mut().zip(&self.weights) {
            *o += inputs.dot(w).into();
        }
    }
}

///A specialized [`Dense`] layer that operates on boolean inputs
///and can incrementally update the output accumulator.
#[derive(Debug, Clone, Zeroable)]
pub struct BitDense<WB: Zeroable, const INPUTS: usize, const OUTPUTS: usize> {
    pub weights: [[WB; OUTPUTS]; INPUTS],
    pub biases: [WB; OUTPUTS],
}

impl<WB: Zeroable + Clone, const INPUTS: usize, const OUTPUTS: usize> BitDense<WB, INPUTS, OUTPUTS>
where
    [WB; OUTPUTS]: VecAdd + VecSub,
{
    ///Clear an accumulator to a default state.
    pub fn empty(&self, outputs: &mut [WB; OUTPUTS]) {
        *outputs = self.biases.clone();
    }

    ///Add an input feature to an accumulator.
    pub fn add(&self, index: usize, outputs: &mut [WB; OUTPUTS]) {
        outputs.vec_add(&self.weights[index]);
    }

    ///Remove an input feature from an accumulator.
    pub fn sub(&self, index: usize, outputs: &mut [WB; OUTPUTS]) {
        outputs.vec_sub(&self.weights[index]);
    }

    ///Debug function for testing; Recommended to use the [`add`](Self::add) and [`sub`](Self::sub)
    ///functions instead to incrementally add and remove input features from an accumulator.
    pub fn activate(&self, inputs: &[bool; INPUTS], outputs: &mut [WB; OUTPUTS]) {
        self.empty(outputs);
        for (i, &input) in inputs.iter().enumerate() {
            if input {
                self.add(i, outputs);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct Rng(u128);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(0xDA942042E4DD58B5);
            (self.0 >> 64) as u64
        }
    }

    const RNG: Rng = Rng(0x576F77596F75466F756E645468697321);

    #[test]
    fn bitboard_dense_is_dense() {
        const INPUTS: usize = 64;
        const OUTPUTS: usize = 256;

        let mut rng = RNG;
        for _ in 0..100 {
            let mut dense = Dense {
                weights: [[0; INPUTS]; OUTPUTS],
                biases: [0; OUTPUTS],
            };
            let mut bit_dense = BitDense {
                weights: [[0; OUTPUTS]; INPUTS],
                biases: [0; OUTPUTS],
            };
            for output in 0..OUTPUTS {
                for input in 0..64 {
                    let weight = rng.next() as i8;
                    dense.weights[output][input] = weight;
                    bit_dense.weights[input][output] = weight;
                }
            }
            for (d, b) in dense.biases.iter_mut().zip(&mut bit_dense.biases) {
                let bias = rng.next() as i8;
                *d = bias as i32;
                *b = bias;
            }

            let bit_input = rng.next();
            let mut inputs = [0; 64];
            let mut bit_dense_inputs = [false; 64];
            for (i, (square, square2)) in inputs.iter_mut().zip(&mut bit_dense_inputs).enumerate() {
                *square2 = ((bit_input >> i) & 1) != 0;
                *square = *square2 as i8;
            }

            let mut dense_output = [0; OUTPUTS];
            let mut bit_dense_output = [0; OUTPUTS];
            dense.activate(&inputs, &mut dense_output);
            bit_dense.activate(&bit_dense_inputs, &mut bit_dense_output);
            assert!(dense_output
                .iter()
                .zip(&bit_dense_output)
                .all(|(&d, &b)| d as i8 == b));
        }
    }
}
