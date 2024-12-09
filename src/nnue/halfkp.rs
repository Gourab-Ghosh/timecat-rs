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

type AccumulatorDataType = i16;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct HalfKPFeatureTransformer<T> {
    weights: Box<
        SerdeWrapper<
            [MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>;
                HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS],
        >,
    >,
    // http://www.talkchess.com/forum3/viewtopic.php?f=7&t=75296
    bona_piece_zero_weights:
        Box<SerdeWrapper<[MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>; NUM_SQUARES]>>,
    biases: Box<MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>>,
}

impl<T: BinRead<Args = ()> + Debug> BinRead for HalfKPFeatureTransformer<T> {
    type Args = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binread::ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let biases = BinRead::read_options(reader, options, ())?;
        let mut weights: Vec<MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>> =
            Vec::with_capacity(HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS);
        let mut bona_piece_zero_weights: Vec<MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>> =
            Vec::with_capacity(NUM_SQUARES);
        for _ in 0..NUM_SQUARES {
            bona_piece_zero_weights.push(BinRead::read_options(reader, options, ())?);
            for _ in 0..(HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS / NUM_SQUARES) {
                weights.push(BinRead::read_options(reader, options, ())?);
            }
        }
        Ok(Self {
            weights: SerdeWrapper::from_boxed_value(weights.try_into().map_err(|_| {
                binread::Error::AssertFail {
                    pos: 0,
                    message: "Failed to convert weights into fixed-size array".to_string(),
                }
            })?),
            bona_piece_zero_weights: SerdeWrapper::from_boxed_value(
                bona_piece_zero_weights
                    .try_into()
                    .map_err(|_| binread::Error::AssertFail {
                        pos: 0,
                        message: "Failed to convert weights into fixed-size array".to_string(),
                    })?,
            ),
            biases,
        })
    }
}

impl<T> HalfKPFeatureTransformer<T> {
    pub fn get_weights(
        &self,
    ) -> &[MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>; HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS]
    {
        &self.weights
    }

    pub fn get_biases(&self) -> &MathVec<T, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS> {
        &self.biases
    }
}

impl<T> Debug for HalfKPFeatureTransformer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HalfKPFeatureTransformer[{}->{}x2]",
            HALFKP_FEATURE_TRANSFORMER_NUM_INPUTS, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS,
        )
    }
}

impl<T: Clone, U: From<T> + Debug> From<&HalfKPFeatureTransformer<T>>
    for HalfKPFeatureTransformer<U>
{
    fn from(value: &HalfKPFeatureTransformer<T>) -> Self {
        HalfKPFeatureTransformer {
            weights: SerdeWrapper::from_boxed_value(
                value
                    .weights
                    .iter()
                    .map_into()
                    .collect_vec()
                    .try_into()
                    .unwrap(),
            ),
            bona_piece_zero_weights: SerdeWrapper::from_boxed_value(
                value
                    .bona_piece_zero_weights
                    .iter()
                    .map_into()
                    .collect_vec()
                    .try_into()
                    .unwrap(),
            ),
            biases: Box::new(value.get_biases().into()),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(BinRead)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, BinRead)]
pub struct HalfKPModelReader {
    #[br(args(VERSION))]
    version: BinaryMagic<u32>,
    #[br(args(ARCHITECTURE))]
    architecture: BinaryMagic<u32>,
    description_len: u32,
    #[br(count = description_len, try_map = String::from_utf8)]
    description: String,
    #[br(args(TRANSFORMER_ARCHITECTURE))]
    transformer_architecture: BinaryMagic<u32>,
    #[cfg_attr(feature = "serde", serde(with = "SerdeHandler"))]
    #[br(map = Arc::new)]
    // #[br(map = |transformer: HalfKPFeatureTransformer<i16>| Arc::new((&transformer).into()))]
    transformer: Arc<HalfKPFeatureTransformer<AccumulatorDataType>>,
    #[br(args(NETWORK_ARCHITECTURE))]
    network_architecture: BinaryMagic<u32>,
    #[cfg_attr(feature = "serde", serde(with = "SerdeHandler"))]
    #[br(map = Arc::new)]
    network: Arc<HalfKPNetwork>,
}

impl HalfKPModelReader {
    pub fn to_empty_model(
        &self,
        white_king_square: Square,
        black_king_square: Square,
    ) -> HalfKPModel {
        let accumulators = [
            self.transformer.get_biases().into(),
            self.transformer.get_biases().into(),
        ];
        let position: BoardPosition = BoardPositionBuilder::new()
            .add_piece(white_king_square, WhiteKing)
            .add_piece(black_king_square, BlackKing)
            .try_into()
            .unwrap();
        HalfKPModel {
            accumulator: Accumulator {
                king_squares_rotated: [white_king_square, black_king_square.rotate()],
                accumulators,
            },
            transformer: self.transformer.clone(),
            network: self.network.clone(),
            last_position: position,
        }
    }

    pub fn to_model(&self, position: &BoardPosition) -> HalfKPModel {
        let mut halfkp_model = self.to_empty_model(
            position.get_king_square(White),
            position.get_king_square(Black),
        );
        halfkp_model.update_empty_model(position);
        halfkp_model.update_last_position(position.clone());
        halfkp_model
    }

    pub fn to_default_model(&self) -> HalfKPModel {
        self.to_model(&BoardPosition::default())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
struct Accumulator {
    king_squares_rotated: [Square; 2],
    accumulators: [MathVec<AccumulatorDataType, HALFKP_FEATURE_TRANSFORMER_NUM_OUTPUTS>; 2],
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct HalfKPModel {
    #[cfg_attr(feature = "serde", serde(with = "SerdeHandler"))]
    transformer: Arc<HalfKPFeatureTransformer<AccumulatorDataType>>,
    #[cfg_attr(feature = "serde", serde(with = "SerdeHandler"))]
    network: Arc<HalfKPNetwork>,
    accumulator: Accumulator,
    last_position: BoardPosition,
}

impl HalfKPModel {
    pub fn index(&self, turn: Color, mut piece: Piece, mut square: Square) -> usize {
        let king = get_item_unchecked!(self.accumulator.king_squares_rotated, turn.to_index());
        if turn == Black {
            square = square.rotate();
            piece.flip_color();
        }
        NUM_SQUARES * (NUM_COLORS * (NUM_PIECE_TYPES - 1) * king.to_index() + piece.to_index())
            + square.to_index()
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
    fn update_empty_model_of_one_side(&mut self, position: &BoardPosition, turn: Color) {
        position
            .custom_iter(&ALL_PIECE_TYPES[..5], &[White, Black], BB_ALL)
            .for_each(|(piece, square)| self.activate_non_king_piece(turn, piece, square))
    }

    #[inline]
    fn update_empty_model(&mut self, position: &BoardPosition) {
        ALL_COLORS
            .into_iter()
            .for_each(|turn| self.update_empty_model_of_one_side(position, turn));
    }

    #[inline]
    pub fn clear_one_side(&mut self, turn: Color) {
        self.accumulator.accumulators[turn.to_index()].clone_from(self.transformer.get_biases());
    }

    #[inline]
    pub fn clear(&mut self) {
        ALL_COLORS
            .into_iter()
            .for_each(|turn| self.clear_one_side(turn));
    }

    #[inline]
    fn update_last_position(&mut self, position: BoardPosition) {
        self.last_position = position;
    }

    pub fn reset_model(&mut self, position: &BoardPosition) {
        self.clear();
        self.accumulator.king_squares_rotated = [
            position.get_king_square(White),
            position.get_king_square(Black).rotate(),
        ];
        self.update_empty_model(position);
        self.update_last_position(position.clone());
    }

    fn update_king(&mut self, position: &BoardPosition, color: Color) {
        let new_king_square = if color == White {
            position.get_king_square(color)
        } else {
            position.get_king_square(color).rotate()
        };
        self.accumulator.king_squares_rotated[color.to_index()] = new_king_square;
        self.clear_one_side(color);
        self.update_empty_model_of_one_side(position, color);
    }

    pub fn update_model(&mut self, position: &BoardPosition) {
        let mut white_king_updated = false;
        let mut black_king_updated = false;
        if self.last_position.get_king_square(White) != position.get_king_square(White) {
            self.update_king(position, White);
            white_king_updated = true;
        }
        if self.last_position.get_king_square(Black) != position.get_king_square(Black) {
            self.update_king(position, Black);
            black_king_updated = true;
        }
        let mut colors_to_update = Vec::with_capacity(2);
        if !white_king_updated {
            colors_to_update.push(White);
        }
        if !black_king_updated {
            colors_to_update.push(Black);
        }
        #[derive(Clone)]
        enum Change {
            Added((Piece, Square)),
            Removed((Piece, Square)),
        }
        let last_position_piece_masks = self.last_position.get_all_piece_masks().to_owned();
        let last_position_occupied_colors = [
            self.last_position.occupied_color(White),
            self.last_position.occupied_color(Black),
        ];
        colors_to_update
            .into_iter()
            .cartesian_product(
                ALL_PIECE_TYPES[..5]
                    .iter()
                    .cartesian_product(ALL_COLORS)
                    .flat_map(|(&piece_type, color)| {
                        let prev_occupied = last_position_occupied_colors[color.to_index()]
                            & last_position_piece_masks[piece_type.to_index()];
                        let new_occupied =
                            position.occupied_color(color) & position.get_piece_mask(piece_type);
                        (!prev_occupied & new_occupied)
                            .map(move |square| {
                                Change::Added((Piece::new(piece_type, color), square))
                            })
                            .chain((prev_occupied & !new_occupied).map(move |square| {
                                Change::Removed((Piece::new(piece_type, color), square))
                            }))
                    }),
            )
            .for_each(|(turn, change)| match change {
                Change::Added((piece, square)) => self.activate_non_king_piece(turn, piece, square),
                Change::Removed((piece, square)) => {
                    self.deactivate_non_king_piece(turn, piece, square)
                }
            });
        self.update_last_position(position.clone());
    }

    pub fn evaluate_current_state_flipped(&self, turn: Color) -> Score {
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

    pub fn evaluate_current_state(&self, turn: Color) -> Score {
        let score_flipped = self.evaluate_current_state_flipped(turn);
        if turn == White {
            score_flipped
        } else {
            -score_flipped
        }
    }

    pub fn update_model_and_evaluate(&mut self, position: &BoardPosition) -> Score {
        self.update_model(position);
        self.evaluate_current_state(position.turn())
    }

    pub fn slow_evaluate_from_position(&self, position: &BoardPosition) -> Score {
        let mut model = Self {
            transformer: self.transformer.clone(),
            network: self.network.clone(),
            accumulator: Accumulator {
                accumulators: [
                    self.transformer.get_biases().clone(),
                    self.transformer.get_biases().clone(),
                ],
                king_squares_rotated: [
                    position.get_king_square(White),
                    position.get_king_square(Black).rotate(),
                ],
            },
            last_position: position.clone(),
        };
        model.update_empty_model(position);
        model.evaluate_current_state(position.turn())
    }
}
