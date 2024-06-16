use super::*;

const VERSION: u32 = 0x7AF32F16;
const ARCHITECTURE: u32 = 0x3E5AA6EE;
const TRANSFORMER_ARCHITECTURE: u32 = 0x5D69D7B8;
const NETWORK_ARCHITECTURE: u32 = 0x63337156;

const HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS: usize =
    NUM_SQUARES.pow(2) * (NUM_PIECE_TYPES - 1) * NUM_COLORS;
const HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS: usize = 256;
const FIRST_HIDDEN_LAYER_NUM_OUTPUTS: usize = 32;
const SECOND_HIDDEN_LAYER_NUM_OUTPUTS: usize = 32;
const FINAL_NUM_OUTPUTS: usize = 1;

const QUANTIZATION_SCALE_BY_POW_OF_TWO: i8 = 4;

#[derive(Clone)]
struct HalfKPFeatureTransformer {
    weights: Arc<
        Box<
            [MathVec<i16, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>;
                HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS],
        >,
    >,
    // http://www.talkchess.com/forum3/viewtopic.php?f=7&t=75296
    bona_piece_zero_weights: Arc<Box<[[i16; HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS]; NUM_SQUARES]>>,
    biases: Arc<Box<MathVec<i16, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>>>,
}

impl BinRead for HalfKPFeatureTransformer {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        _: Self::Args,
    ) -> binread::BinResult<Self> {
        let biases: Box<MathVec<i16, 256>> = BinRead::read_options(reader, options, ())?;
        let mut weights: Vec<MathVec<i16, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>> =
            Vec::with_capacity(HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS);
        let mut bona_piece_zero_weights: Vec<[i16; HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS]> =
            Vec::with_capacity(NUM_SQUARES);
        for _ in 0..NUM_SQUARES {
            bona_piece_zero_weights.push(BinRead::read_options(reader, options, ())?);
            for _ in 0..(HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS / NUM_SQUARES) {
                weights.push(BinRead::read_options(reader, options, ())?);
            }
        }
        Ok(Self {
            weights: Arc::new(weights.try_into().unwrap()),
            bona_piece_zero_weights: Arc::new(bona_piece_zero_weights.try_into().unwrap()),
            biases: Arc::new(biases),
        })
    }
}

impl HalfKPFeatureTransformer {
    pub fn get_weights(
        &self,
    ) -> &[MathVec<i16, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>;
            HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS] {
        &self.weights
    }

    pub fn get_biases(&self) -> &MathVec<i16, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS> {
        &self.biases
    }
}

impl fmt::Debug for HalfKPFeatureTransformer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HalfKPFeatureTransformer[{}->{}]",
            HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS,
        )
    }
}

#[derive(Clone, BinRead)]
struct HalfKPNetwork {
    pub hidden_layer_1: Layer<
        i8,
        i32,
        { HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS * NUM_COLORS },
        FIRST_HIDDEN_LAYER_NUM_OUTPUTS,
    >,
    pub hidden_layer_2:
        Layer<i8, i32, FIRST_HIDDEN_LAYER_NUM_OUTPUTS, SECOND_HIDDEN_LAYER_NUM_OUTPUTS>,
    pub output_layer: Layer<i8, i32, SECOND_HIDDEN_LAYER_NUM_OUTPUTS, FINAL_NUM_OUTPUTS>,
}

impl Debug for HalfKPNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HalfKPNetwork[{}x{}->{}->{}->{}]",
            HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS,
            NUM_COLORS,
            FIRST_HIDDEN_LAYER_NUM_OUTPUTS,
            SECOND_HIDDEN_LAYER_NUM_OUTPUTS,
            FINAL_NUM_OUTPUTS,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
struct Architecture {
    architecture: u32,
}

impl Deref for Architecture {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.architecture
    }
}

impl Debug for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.architecture)
    }
}

impl BinRead for Architecture {
    type Args = (u32,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        (magic,): Self::Args,
    ) -> BinResult<Self> {
        let architecture = BinRead::read_options(reader, options, ())?;
        if architecture == magic {
            Ok(Self { architecture })
        } else {
            Err(binread::Error::BadMagic {
                pos: reader.stream_position()?,
                found: Box::new(architecture),
            })
        }
    }
}

#[derive(Debug, Clone, BinRead)]
pub struct HalfKPModelReader {
    #[br(args(VERSION))]
    version: Architecture,
    #[br(args(ARCHITECTURE))]
    architecture: Architecture,
    desc_len: u32,
    #[br(count = desc_len, try_map = String::from_utf8)]
    desc: String,
    #[br(args(TRANSFORMER_ARCHITECTURE))]
    transformer_architecture: Architecture,
    transformer: HalfKPFeatureTransformer,
    #[br(args(NETWORK_ARCHITECTURE))]
    network_architecture: Architecture,
    network: HalfKPNetwork,
}

impl HalfKPModelReader {
    pub fn to_model(&self, sub_board: &SubBoard) -> HalfKPModel {
        let accumulators = [
            self.transformer.get_biases().clone(),
            self.transformer.get_biases().clone(),
        ];
        let mut halfkp_model = HalfKPModel {
            accumulator: Accumulator {
                king_squares_rotated: [
                    sub_board.get_king_square(White),
                    sub_board.get_king_square(Black).rotate(),
                ],
                accumulators: CustomDebug::new(accumulators, |_| {
                    format!("Accumulators[{}x2]", HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS)
                }),
            },
            transformer: self.transformer.clone(),
            network: self.network.clone(),
            last_sub_board: CustomDebug::new(sub_board.clone(), |sub_board| sub_board.get_fen()),
        };
        halfkp_model.update_empty_model(sub_board);
        halfkp_model
    }

    pub fn to_default_model(&self) -> HalfKPModel {
        self.to_model(&SubBoard::default())
    }
}

#[derive(Clone, Debug)]
struct Accumulator {
    king_squares_rotated: [Square; 2],
    accumulators: CustomDebug<[MathVec<i16, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>; 2]>,
}

#[derive(Clone, Debug)]
pub struct HalfKPModel {
    transformer: HalfKPFeatureTransformer,
    network: HalfKPNetwork,
    accumulator: Accumulator,
    last_sub_board: CustomDebug<SubBoard>,
}

impl HalfKPModel {
    pub fn index(&self, turn: Color, piece: Piece, mut square: Square) -> usize {
        let mut piece_color = piece.get_color();
        let mut size = HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS;
        let mut index = 0;
        macro_rules! index {
            ($index:expr; $total:expr) => {
                size /= $total;
                index += size * $index;
            };
        }

        let king = get_item_unchecked!(self.accumulator.king_squares_rotated, turn.to_index());
        if turn == Black {
            square = square.rotate();
            piece_color = !piece_color;
        }
        index!(king.to_index(); NUM_SQUARES);
        index!(piece.get_piece_type().to_index(); NUM_PIECE_TYPES - 1);
        index!(piece_color.to_index(); NUM_COLORS);
        index!(square.to_index(); NUM_SQUARES);
        index
    }

    pub fn activate_non_king_piece(&mut self, turn: Color, piece: Piece, square: Square) {
        let index = self.index(turn, piece, square);
        *get_item_unchecked_mut!(self.accumulator.accumulators, turn.to_index()) +=
            get_item_unchecked!(&self.transformer.get_weights(), index);
    }

    pub fn deactivate_non_king_piece(&mut self, turn: Color, piece: Piece, square: Square) {
        let index = self.index(turn, piece, square);
        *get_item_unchecked_mut!(self.accumulator.accumulators, turn.to_index()) -=
            get_item_unchecked!(&self.transformer.get_weights(), index);
    }

    #[inline]
    fn update_empty_model_with_color(&mut self, sub_board: &SubBoard, turn: Color) {
        sub_board
            .custom_iter(&ALL_PIECE_TYPES[..5], &[White, Black], BB_ALL)
            .for_each(|(piece, square)| self.activate_non_king_piece(turn, piece, square))
    }

    #[inline]
    fn update_empty_model(&mut self, sub_board: &SubBoard) {
        ALL_COLORS
            .into_iter()
            .for_each(|turn| self.update_empty_model_with_color(sub_board, turn));
    }

    #[inline]
    pub fn clear(&mut self) {
        self.accumulator
            .accumulators
            .iter_mut()
            .for_each(|x| x.clone_from(self.transformer.get_biases()));
    }

    #[inline]
    fn update_last_sub_board(&mut self, sub_board: SubBoard) {
        self.last_sub_board = CustomDebug::new(sub_board.clone(), |sub_board| sub_board.get_fen());
    }

    pub fn reset_model(&mut self, sub_board: &SubBoard) {
        self.clear();
        self.accumulator.king_squares_rotated = [
            sub_board.get_king_square(White),
            sub_board.get_king_square(Black).rotate(),
        ];
        self.update_empty_model(sub_board);
        self.update_last_sub_board(sub_board.clone());
    }

    // pub fn update_king(&mut self, turn: Color, square: Square, sub_board: &SubBoard) {
    //     self.accumulator.king_squares_rotated[turn.to_index()] = if turn == White {
    //         square
    //     } else {
    //         square.rotate()
    //     };
    //     self.clear();
    //     self.update_empty_model(sub_board);
    //     self.update_last_sub_board(sub_board.clone());
    // }

    pub fn update_model(&mut self, sub_board: &SubBoard) {
        if self.last_sub_board.get_king_square(White) != sub_board.get_king_square(White)
            || self.last_sub_board.get_king_square(Black) != sub_board.get_king_square(Black)
        {
            self.reset_model(sub_board);
            return;
        }
        #[derive(Clone)]
        enum Change {
            Added((Piece, Square)),
            Removed((Piece, Square)),
        }
        let piece_masks = self.last_sub_board.get_piece_masks().to_owned();
        let occupied_cos = [
            self.last_sub_board.occupied_co(White),
            self.last_sub_board.occupied_co(Black),
        ];
        ALL_PIECE_TYPES[..5]
            .into_iter()
            .cartesian_product(ALL_COLORS)
            .map(|(&piece_type, color)| {
                let prev_occupied =
                    occupied_cos[color.to_index()] & piece_masks[piece_type.to_index()];
                let new_occupied =
                    sub_board.occupied_co(color) & sub_board.get_piece_mask(piece_type);
                (!prev_occupied & new_occupied)
                    .map(move |square| Change::Added((Piece::new(piece_type, color), square)))
                    .chain((prev_occupied & !new_occupied).map(move |square| {
                        Change::Removed((Piece::new(piece_type, color), square))
                    }))
            })
            .flatten()
            .cartesian_product(ALL_COLORS)
            .for_each(|(change, turn)| match change {
                Change::Added((piece, square)) => self.activate_non_king_piece(turn, piece, square),
                Change::Removed((piece, square)) => {
                    self.deactivate_non_king_piece(turn, piece, square)
                }
            });
        self.update_last_sub_board(sub_board.clone());
    }

    pub(crate) fn evaluate_flipped(&self, turn: Color) -> Score {
        let mut inputs: [i8; 512] = [0; HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS * 2];
        for color in ALL_COLORS {
            let input = if color == turn {
                &mut inputs[..HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS]
            } else {
                &mut inputs[HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS..]
            };
            self.accumulator.accumulators[color.to_index()].clipped_relu_into(
                0,
                0,
                127,
                input.try_into().unwrap(),
            );
        }
        let inputs = MathVec::new(inputs);
        let inputs = self
            .network
            .hidden_layer_1
            .forward(inputs)
            .clipped_relu(6, 0, 127);
        let inputs = self
            .network
            .hidden_layer_2
            .forward(inputs)
            .clipped_relu(6, 0, 127);
        let outputs = self.network.output_layer.forward(inputs);
        (get_item_unchecked!(outputs, 0) / 16) as Score
    }

    pub(crate) fn evaluate(&self, turn: Color) -> Score {
        let score_flipped = self.evaluate_flipped(turn);
        if turn == White {
            score_flipped
        } else {
            -score_flipped
        }
    }

    pub fn update_model_and_evaluate(&mut self, sub_board: &SubBoard) -> Score {
        self.update_model(sub_board);
        self.evaluate(sub_board.turn())
    }

    pub fn evaluate_from_sub_board(&self, sub_board: &SubBoard) -> Score {
        let mut model = Self {
            transformer: self.transformer.clone(),
            network: self.network.clone(),
            accumulator: Accumulator {
                accumulators: CustomDebug::new(
                    [
                        self.transformer.get_biases().clone(),
                        self.transformer.get_biases().clone(),
                    ],
                    self.accumulator
                        .accumulators
                        .get_debug_message_func()
                        .to_owned(),
                ),
                king_squares_rotated: [
                    sub_board.get_king_square(White),
                    sub_board.get_king_square(Black).rotate(),
                ],
            },
            last_sub_board: CustomDebug::new(sub_board.clone(), |sub_board| sub_board.get_fen()),
        };
        model.update_empty_model(sub_board);
        model.evaluate(sub_board.turn())
    }
}
