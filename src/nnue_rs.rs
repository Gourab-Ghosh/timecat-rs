//!# Rust NNUE inference library
//!`nnue-rs` is an [NNUE](https://www.chessprogramming.org/NNUE) inference library written in Rust.

pub mod ops {
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

    pub trait Dot<Rhs = Self> {
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
}

pub mod layers {
    //!Module for layer types.

    use std::ops::AddAssign;
    use std::usize;

    use super::ops::*;

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
                for (i, (sq, sq2)) in inputs.iter_mut().zip(&mut bit_dense_inputs).enumerate() {
                    *sq2 = ((bit_input >> i) & 1) != 0;
                    *sq = *sq2 as i8;
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
}

pub mod stockfish {
    //!Module for Stockfish networks.

    ///Module for the Stockfish HalfKP 256x2-32-32 architecture.
    use super::*;
    pub mod halfkp {
        use std::convert::TryInto;
        use std::io::prelude::*;

        use super::ops::*;
        use super::*;

        use binread::prelude::*;

        const INPUTS: usize = Square::NUM //For each king position...
            * (Piece::NUM - 1) //For each non-king piece...
            * Color::NUM //For each colour piece...
            * Square::NUM; //For each square...
        const IL_OUT: usize = 256;
        const H1_OUT: usize = 32;
        const H2_OUT: usize = 32;
        const OUTPUTS: usize = 1;

        ///The Stockfish HalfKP 256x2-32-32 architecture.
        ///See [`new_state`](Self::new_state) for how to use it.
        ///See [`SfHalfKpFullModel`] for how to load it.
        #[derive(Debug, Clone)]
        pub struct SfHalfKpModel {
            pub transformer: SfHalfKpFeatureTransformer,
            pub network: SfHalfKpNetwork,
        }

        const TRANSFORMER_ARCH: u32 = 0x5D69D7B8;

        ///The Stockfish HalfKP feature transformer.
        ///Due to its size, the transformer is heap allocated.
        #[derive(Debug, Clone)]
        pub struct SfHalfKpFeatureTransformer {
            pub input_layer: Box<BitDense<i16, INPUTS, IL_OUT>>,
        }

        impl BinRead for SfHalfKpFeatureTransformer {
            type Args = ();

            fn read_options<R: Read + Seek>(
                reader: &mut R,
                options: &binread::ReadOptions,
                _: Self::Args,
            ) -> BinResult<Self> {
                let mut this = Self {
                    input_layer: bytemuck::zeroed_box(),
                };
                this.input_layer.biases = BinRead::read_options(reader, options, ())?;
                //Iterate through each block of weights for a given king position
                const KING_POS_BLOCK: usize = INPUTS / Square::NUM;
                for weights in this.input_layer.weights.chunks_exact_mut(KING_POS_BLOCK) {
                    //Discard extra BONA_PIECE_ZERO weights
                    <[i16; IL_OUT]>::read_options(reader, options, ())?;
                    //Read the rest of the block
                    for weight in weights {
                        *weight = BinRead::read_options(reader, options, ())?;
                    }
                }
                Ok(this)
            }
        }

        const NETWORK_ARCH: u32 = 0x63337156;

        ///The Stockfish HalfKP network. This is the rest
        ///of the network after the feature transformer.
        #[derive(Debug, Clone)]
        pub struct SfHalfKpNetwork {
            pub hidden_layer_1: Dense<i8, i32, { IL_OUT * Color::NUM }, H1_OUT>,
            pub hidden_layer_2: Dense<i8, i32, H1_OUT, H2_OUT>,
            pub output_layer: Dense<i8, i32, H2_OUT, OUTPUTS>,
        }

        impl BinRead for SfHalfKpNetwork {
            type Args = ();

            fn read_options<R: Read + Seek>(
                reader: &mut R,
                options: &binread::ReadOptions,
                _: Self::Args,
            ) -> BinResult<Self> {
                #[derive(BinRead)]
                struct SfHalfKpNetwork {
                    hidden_layer_1: SfDense<i8, i32, { IL_OUT * Color::NUM }, H1_OUT>,
                    hidden_layer_2: SfDense<i8, i32, H1_OUT, H2_OUT>,
                    output_layer: SfDense<i8, i32, H2_OUT, OUTPUTS>,
                }
                let network = SfHalfKpNetwork::read_options(reader, options, ())?;
                Ok(Self {
                    hidden_layer_1: network.hidden_layer_1.into(),
                    hidden_layer_2: network.hidden_layer_2.into(),
                    output_layer: network.output_layer.into(),
                })
            }
        }

        ///The Stockfish HalfKP accumulator state. Use [`add`](Self::add) and [`sub`](Self::sub)
        ///to add and remove input features, and [`activate`](Self::activate) to get an output.
        pub struct SfHalfKpState<'m> {
            model: &'m SfHalfKpModel,
            //Already normalized king positions
            kings: [Square; Color::NUM],
            accumulator: [[i16; IL_OUT]; Color::NUM],
        }

        impl SfHalfKpModel {
            ///Create a new accumulator state with the given king positions.
            pub fn new_state(&self, white_king: Square, black_king: Square) -> SfHalfKpState {
                let mut accumulator = [[0; IL_OUT]; Color::NUM];
                for half in &mut accumulator {
                    self.transformer.input_layer.empty(half);
                }
                SfHalfKpState {
                    model: self,
                    kings: [white_king, black_king.rotate()],
                    accumulator,
                }
            }
        }

        impl SfHalfKpState<'_> {
            fn index(
                &self,
                color: Color,
                piece: Piece,
                mut piece_color: Color,
                mut square: Square,
            ) -> usize {
                let mut size = INPUTS;
                let mut index = 0;
                macro_rules! index {
                    ($index:expr; $total:expr) => {
                        size /= $total;
                        index += size * $index;
                    };
                }

                //Normalize indexes. King positions come pre-normalized.
                let king = self.kings[color as usize];
                if color == Color::Black {
                    square = square.rotate();
                    piece_color = !piece_color;
                }
                index!(king as usize; Square::NUM);
                index!(piece as usize; Piece::NUM - 1);
                index!(piece_color as usize; Color::NUM);
                index!(square as usize; Square::NUM);
                index
            }

            ///Add an input feature to one half of the accumulator.
            pub fn add(&mut self, color: Color, piece: Piece, piece_color: Color, square: Square) {
                let index = self.index(color, piece, piece_color, square);
                self.model
                    .transformer
                    .input_layer
                    .add(index, &mut self.accumulator[color as usize]);
            }

            ///Remove an input feature from one half of the accumulator.
            pub fn sub(&mut self, color: Color, piece: Piece, piece_color: Color, square: Square) {
                let index = self.index(color, piece, piece_color, square);
                self.model
                    .transformer
                    .input_layer
                    .sub(index, &mut self.accumulator[color as usize]);
            }

            ///Moves a king for a side to a new square. Clears the accumulator for that side.
            pub fn update_king(&mut self, color: Color, square: Square) {
                self.kings[color as usize] = if color == Color::Black {
                    square.rotate()
                } else {
                    square
                };
                self.model
                    .transformer
                    .input_layer
                    .empty(&mut self.accumulator[color as usize]);
            }

            ///Activate the network, getting an output. The output is relative to
            ///the side to move. It usually has to be rescaled to another range.
            ///A helper [`scale_nn_to_centipawns`] is provided for centipawns.
            pub fn activate(&mut self, side_to_move: Color) -> [i32; OUTPUTS] {
                const RELU_MIN: i8 = 0;
                const RELU_MAX: i8 = 127;
                const RELU_SCALE: i8 = 6;

                let mut inputs = [0; IL_OUT * 2];
                for &color in &Color::ALL {
                    let input = if color == side_to_move {
                        &mut inputs[..IL_OUT]
                    } else {
                        &mut inputs[IL_OUT..]
                    }
                    .try_into()
                    .unwrap();
                    self.accumulator[color as usize].clipped_relu(0, RELU_MIN, RELU_MAX, input);
                }
                let mut outputs = [0; H1_OUT];
                self.model
                    .network
                    .hidden_layer_1
                    .activate(&inputs, &mut outputs);

                let mut inputs = [0; H1_OUT];
                outputs.clipped_relu(RELU_SCALE, RELU_MIN, RELU_MAX, &mut inputs);
                let mut outputs = [0; H2_OUT];
                self.model
                    .network
                    .hidden_layer_2
                    .activate(&inputs, &mut outputs);

                let mut inputs = [0; H2_OUT];
                outputs.clipped_relu(RELU_SCALE, RELU_MIN, RELU_MAX, &mut inputs);
                let mut outputs = [0; OUTPUTS];
                self.model
                    .network
                    .output_layer
                    .activate(&inputs, &mut outputs);

                outputs
            }
        }

        ///Scale the NNUE output to centipawns
        pub const fn scale_nn_to_centipawns(nn_output: i32) -> i32 {
            //Values grabbed from the Stockfish Evaluation Guide
            nn_output * 100 / 16 / 208
        }

        ///The NNUE format version that this architecture can be read from.
        pub const VERSION: u32 = 0x7AF32F16;

        const ARCH: u32 = 0x3E5AA6EE;

        ///A [`SfHalfKpModel`] with a description. Read it from a reader with [`BinRead::read`].
        ///It will error if the NNUE file does not have a matching [`VERSION`].
        #[derive(Debug, Clone)]
        pub struct SfHalfKpFullModel {
            pub desc: String,
            pub model: SfHalfKpModel,
        }

        impl BinRead for SfHalfKpFullModel {
            type Args = ();

            fn read_options<R: Read + Seek>(
                reader: &mut R,
                options: &binread::ReadOptions,
                args: Self::Args,
            ) -> BinResult<Self> {
                #[derive(Debug, BinRead)]
                pub struct SfHalfKpModelReader {
                    #[br(args(VERSION))]
                    version: Magic<u32>,
                    #[br(args(ARCH))]
                    arch: Magic<u32>,
                    desc_len: u32,
                    #[br(count = desc_len, try_map = String::from_utf8)]
                    desc: String,
                    #[br(args(TRANSFORMER_ARCH))]
                    transformer_arch: Magic<u32>,
                    transformer: SfHalfKpFeatureTransformer,
                    #[br(args(NETWORK_ARCH))]
                    network_arch: Magic<u32>,
                    network: SfHalfKpNetwork,
                }
                let model = SfHalfKpModelReader::read_options(reader, options, args)?;
                Ok(SfHalfKpFullModel {
                    desc: model.desc,
                    model: SfHalfKpModel {
                        transformer: model.transformer,
                        network: model.network,
                    },
                })
            }
        }
    }

    use std::io::prelude::*;

    use super::layers::*;

    use binread::prelude::*;
    use bytemuck::Zeroable;

    ///A helper struct for reading [`Dense`] layers from Stockfish NNUE formats with [`BinRead`](BinRead).
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
    pub struct SfBitDense<
        WB: Zeroable + BinRead<Args = ()>,
        const INPUTS: usize,
        const OUTPUTS: usize,
    > {
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
    pub(super) struct Magic<T>(std::marker::PhantomData<T>);

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
}

macro_rules! simple_enum {
    ($(
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),*
        }
    )*) => {$(
        $(#[$attr])*
        $vis enum $name {
            $($variant),*
        }

        impl $name {
            pub const NUM: usize = [$(Self::$variant),*].len();
            pub const ALL: [Self; Self::NUM] = [$(Self::$variant),*];
            #[inline]
            pub fn from_index(index: usize) -> Self {
                $(#[allow(non_upper_case_globals)]
                const $variant: usize = $name::$variant as usize;)*
                #[allow(non_upper_case_globals)]
                match index {
                    $($variant => Self::$variant,)*
                    _ => panic!("Index {} is out of range.", index)
                }
            }
        }
    )*};
}

simple_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
    pub enum Piece {
        Pawn,
        Knight,
        Bishop,
        Rook,
        Queen,
        King
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Color {
        White,
        Black
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8
    }
}

impl std::ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Square {
    pub fn flip(self) -> Self {
        //Flip upper 3 bits, which represent the rank
        Self::from_index(self as usize ^ 0b111000)
    }

    pub fn rotate(self) -> Self {
        //Flip both rank and file bits
        Self::from_index(self as usize ^ 0b111111)
    }
}
