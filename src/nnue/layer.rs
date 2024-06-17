use super::*;

#[derive(Clone, Debug)]
pub struct Layer<
    W: BinRead<Args = ()>,
    B: BinRead<Args = ()>,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
> {
    weights_transpose: Arc<Box<[MathVec<W, NUM_INPUTS>; NUM_OUTPUTS]>>,
    biases: Arc<MathVec<B, NUM_OUTPUTS>>,
}

impl<
        W: BinRead<Args = ()> + fmt::Debug + Copy + Default,
        B: BinRead<Args = ()>,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
    > BinRead for Layer<W, B, NUM_INPUTS, NUM_OUTPUTS>
{
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let biases = BinRead::read_options(reader, options, ())?;
        let mut weights_transpose: Vec<MathVec<W, NUM_INPUTS>> = Vec::with_capacity(NUM_OUTPUTS);
        for _ in 0..NUM_OUTPUTS {
            weights_transpose.push(BinRead::read_options(reader, options, ())?);
        }
        Ok(Self {
            weights_transpose: Arc::new(weights_transpose.try_into().unwrap()),
            biases: Arc::new(biases),
        })
    }
}

impl<
        W: BinRead<Args = ()> + fmt::Debug,
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
        W: BinRead<Args = ()> + Clone,
        B: BinRead<Args = ()> + Clone + AddAssign + From<W> + Mul + Sum<<B as Mul>::Output>,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
    > Layer<W, B, NUM_INPUTS, NUM_OUTPUTS>
{
    pub fn forward(&self, inputs: MathVec<W, NUM_INPUTS>) -> MathVec<B, NUM_OUTPUTS> {
        let mut outputs = self.biases.as_ref().clone();
        for (o, w) in outputs.iter_mut().zip(self.weights_transpose.iter()) {
            *o += inputs.dot(w);
        }
        outputs
    }
}
