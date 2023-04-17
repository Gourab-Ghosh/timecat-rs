use std::convert::TryInto;
use std::io::prelude::*;

use super::*;
use crate::nnue_rs::layers::*;
use crate::nnue_rs::ops::*;
use crate::nnue_rs::*;

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
