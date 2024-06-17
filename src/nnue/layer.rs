use super::*;

#[derive(Clone, Debug, BinRead)]
pub struct Layer<
    W: BinRead<Args = ()> + Debug,
    B: BinRead<Args = ()>,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
> {
    biases: Box<MathVec<B, NUM_OUTPUTS>>,
    #[br(count = NUM_OUTPUTS, map = |v: Vec<MathVec<W, NUM_INPUTS>>| v.try_into().unwrap())]
    weights_transpose: Box<[MathVec<W, NUM_INPUTS>; NUM_OUTPUTS]>,
}

impl<
        W: BinRead<Args = ()> + Debug,
        B: BinRead<Args = ()>,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
    > Layer<W, B, NUM_INPUTS, NUM_OUTPUTS>
{
    #[inline]
    pub fn get_weights_transpose(&self) -> &[MathVec<W, NUM_INPUTS>; NUM_OUTPUTS] {
        &self.weights_transpose
    }

    #[inline]
    pub fn get_biases(&self) -> &MathVec<B, NUM_OUTPUTS> {
        &self.biases
    }
}

impl<
        W: BinRead<Args = ()> + Clone + Debug,
        B: BinRead<Args = ()> + Clone + AddAssign + From<W> + Mul + Sum<<B as Mul>::Output>,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
    > Layer<W, B, NUM_INPUTS, NUM_OUTPUTS>
{
    pub fn forward(&self, inputs: MathVec<W, NUM_INPUTS>) -> MathVec<B, NUM_OUTPUTS> {
        let mut outputs = self.get_biases().clone();
        for (o, w) in outputs.iter_mut().zip(self.weights_transpose.iter()) {
            *o += inputs.dot(w);
        }
        outputs
    }
}
