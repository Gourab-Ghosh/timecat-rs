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
