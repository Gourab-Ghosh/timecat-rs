#![allow(clippy::multiple_bound_locations)]

//!Module for Stockfish networks.

///Module for the Stockfish HalfKP 256x2-32-32 architecture.
pub mod halfkp;

use std::io::prelude::*;

use crate::nnue_rs::layers::*;

use binread::prelude::*;
use bytemuck::Zeroable;

///A helper struct for reading [`Dense`] layers from Stockfish NNUE formats with [`BinRead`].
#[derive(Debug, Zeroable, BinRead)]
pub struct SfDense<
    W: Zeroable + BinRead<Args = ()>,
    B: Zeroable + BinRead<Args = ()>,
    const INPUTS: usize,
    const OUTPUTS: usize,
> {
    pub biases: [B; OUTPUTS],
    pub weights: [[W; INPUTS]; OUTPUTS],
}

impl<
        W: Zeroable + BinRead<Args = ()>,
        B: Zeroable + BinRead<Args = ()>,
        const INPUTS: usize,
        const OUTPUTS: usize,
    > From<SfDense<W, B, INPUTS, OUTPUTS>> for Dense<W, B, INPUTS, OUTPUTS>
{
    fn from(dense: SfDense<W, B, INPUTS, OUTPUTS>) -> Self {
        Self {
            weights: dense.weights,
            biases: dense.biases,
        }
    }
}

///A helper struct for reading [`BitDense`] layers from Stockfish NNUE formats with [`BinRead`].
#[derive(Debug, Zeroable, BinRead)]
pub struct SfBitDense<WB: Zeroable + BinRead<Args = ()>, const INPUTS: usize, const OUTPUTS: usize>
{
    pub biases: [WB; OUTPUTS],
    pub weights: [[WB; OUTPUTS]; INPUTS],
}

impl<WB: Zeroable + BinRead<Args = ()>, const INPUTS: usize, const OUTPUTS: usize>
    From<SfBitDense<WB, INPUTS, OUTPUTS>> for BitDense<WB, INPUTS, OUTPUTS>
{
    fn from(dense: SfBitDense<WB, INPUTS, OUTPUTS>) -> Self {
        Self {
            weights: dense.weights,
            biases: dense.biases,
        }
    }
}

///Helper [`BinRead`]able type for a more powerful `magic`.
#[derive(Debug)]
pub(crate) struct Magic<T>(std::marker::PhantomData<T>);

impl<T: BinRead<Args = ()> + Copy + PartialEq + Send + Sync + 'static> BinRead for Magic<T> {
    type Args = (T,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        (magic,): Self::Args,
    ) -> BinResult<Self> {
        let read = T::read_options(reader, options, ())?;
        if read != magic {
            Err(binread::Error::BadMagic {
                pos: reader.stream_position()?,
                found: Box::new(read),
            })
        } else {
            Ok(Self(std::marker::PhantomData))
        }
    }
}
