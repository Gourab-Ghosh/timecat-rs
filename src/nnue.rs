use super::*;
use binread::BinRead;
use board::Board;
use nnue_rs::stockfish::halfkp::{SfHalfKpFullModel, SfHalfKpModel};
use nnue_rs::Square as StockfishSquare;
use nnue_weights::*;
use std::io::Cursor;

fn square_to_stockfish_square(square: Square) -> StockfishSquare {
    StockfishSquare::from_index(square.to_index())
}

fn construct_model() -> SfHalfKpFullModel {
    let bytes = include_bytes!("nnue_files/nn-62ef826d1a6d.nnue");
    // let bytes = include_bytes!("nnue_files/nn-46832cfbead3.nnue");
    // let bytes = include_bytes!("nnue_files/nn-7756374aaed3.nnue");
    // let bytes = include_bytes!("nnue_files/nn-8a08400ed089.nnue");
    // let bytes = include_bytes!("nnue_files/nn-ad9b42354671.nnue");
    // let bytes = include_bytes!("nnue_files/nn-e8321e467bf6.nnue");
    let mut reader = Cursor::new(bytes);
    SfHalfKpFullModel::read(&mut reader).expect("Bad NNUE file!")
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BinRead)]
#[derive(Debug)]
pub struct StockfishNetwork {
    model: SfHalfKpModel,
}

impl StockfishNetwork {
    pub fn new() -> Self {
        Self {
            model: construct_model().model,
        }
    }

    // pub fn activate(&mut self, piece: Piece, square: usize) {
    //     // self.network.model.
    //     // self.network.activate(piece, square);
    // }

    // pub fn deactivate(&mut self, piece: Piece, square: usize) {
    //     todo!();
    //     // self.network.deactivate(piece, square);
    // }

    fn probe_piece(piece: chess::Piece) -> nnue_rs::Piece {
        match piece {
            Pawn => nnue_rs::Piece::Pawn,
            Knight => nnue_rs::Piece::Knight,
            Bishop => nnue_rs::Piece::Bishop,
            Rook => nnue_rs::Piece::Rook,
            Queen => nnue_rs::Piece::Queen,
            King => panic!("King should not be in non king occupied squares"),
        }
    }

    fn probe_color(color: chess::Color) -> nnue_rs::Color {
        match color {
            White => nnue_rs::Color::White,
            Black => nnue_rs::Color::Black,
        }
    }

    pub fn get_state(&self, sub_board: &chess::Board) -> nnue_rs::stockfish::halfkp::SfHalfKpState {
        let kings_bitboatrd = sub_board.pieces(King);
        let mut state = self.model.new_state(
            square_to_stockfish_square(
                (kings_bitboatrd & sub_board.color_combined(White)).to_square(),
            ),
            square_to_stockfish_square(
                (kings_bitboatrd & sub_board.color_combined(Black)).to_square(),
            ),
        );
        for square in sub_board.combined() & !kings_bitboatrd {
            let piece = Self::probe_piece(sub_board.piece_on(square).unwrap());
            let piece_color = Self::probe_color(sub_board.color_on(square).unwrap());
            for &color in &nnue_rs::Color::ALL {
                state.add(
                    color,
                    piece,
                    piece_color,
                    square_to_stockfish_square(square),
                );
            }
        }
        state
    }

    pub fn eval(&self, sub_board: &chess::Board) -> Score {
        let mut state = self.get_state(sub_board);
        let color = match sub_board.side_to_move() {
            White => nnue_rs::Color::White,
            Black => nnue_rs::Color::Black,
        };
        let score = (state.activate(color)[0] / 16) as Score;
        if color == nnue_rs::Color::White {
            score
        } else {
            -score
        }
    }
}

impl Default for StockfishNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn factor(&self) -> i32 {
        if *self == Self::White {
            1
        } else {
            -1
        }
    }
}

impl From<u8> for Color {
    #[inline(always)]
    fn from(n: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(n) }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Debug)]
pub enum Piece {
    WhitePawn = 0b0000,
    WhiteKnight = 0b0001,
    WhiteBishop = 0b0010,
    WhiteRook = 0b0011,
    WhiteQueen = 0b0100,
    WhiteKing = 0b0101,
    BlackPawn = 0b1000,
    BlackKnight = 0b1001,
    BlackBishop = 0b1010,
    BlackRook = 0b1011,
    BlackQueen = 0b1100,
    BlackKing = 0b1101,
    None = 0b1110,
}

impl Piece {
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize - 2 * self.color_of().index()
    }

    #[inline(always)]
    pub fn flip(self) -> Piece {
        Self::from(self as u8 ^ 0b1000)
    }

    #[inline(always)]
    pub fn color_of(self) -> Color {
        Color::from((self as u8 & 0b1000) >> 3)
    }
}

impl From<u8> for Piece {
    #[inline(always)]
    fn from(n: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(n) }
    }
}

impl TryFrom<char> for Piece {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if Self::PIECE_STR.contains(value) {
            return Ok(Self::from(
                Self::PIECE_STR.chars().position(|c| c == value).unwrap() as u8,
            ));
        }
        Err("Piece symbols should be one of \"KQRBNPkqrbnp\"")
    }
}

impl Default for Piece {
    fn default() -> Self {
        Self::None
    }
}

impl Piece {
    pub const N_PIECES: usize = 13;
    const PIECE_STR: &'static str = "PNBRQK  pnbrqk ";
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
struct Layer {
    weights: &'static [i16],
    biases: &'static [i16],
    activations: Vec<i16>, // used for incremental layer
}

impl Layer {
    pub fn new(weights: &'static [i16], biases: &'static [i16]) -> Self {
        Self {
            weights,
            biases,
            activations: Vec::from(biases),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Network {
    input_layer: Layer,
    hidden_layer: Layer,
}

impl Network {
    pub fn new() -> Self {
        Self {
            input_layer: Layer::new(&NNUE_INPUT_WEIGHTS, &NNUE_INPUT_BIASES),
            hidden_layer: Layer::new(&NNUE_HIDDEN_WEIGHTS, &NNUE_HIDDEN_BIASES),
        }
    }

    #[inline(always)]
    pub fn activate(&mut self, piece: Piece, sq_index: usize) {
        let feature_idx = ((piece.index()) * 64 + sq_index) * self.input_layer.activations.len();
        let weights = self.input_layer.weights
            [feature_idx..feature_idx + self.input_layer.activations.len()]
            .iter();

        self.input_layer
            .activations
            .iter_mut()
            .zip(weights)
            .for_each(|(activation, weight)| *activation += weight);
    }

    #[inline(always)]
    pub fn deactivate(&mut self, piece: Piece, sq_index: usize) {
        let feature_idx = ((piece.index()) * 64 + sq_index) * self.input_layer.activations.len();
        let weights = self.input_layer.weights
            [feature_idx..feature_idx + self.input_layer.activations.len()]
            .iter();

        self.input_layer
            .activations
            .iter_mut()
            .zip(weights)
            .for_each(|(activation, weight)| *activation -= weight);
    }

    pub fn eval(&self, board: &Board) -> Score {
        let bucket = (board.occupied().popcnt() as usize - 1) / 4;
        let bucket_idx = bucket * self.input_layer.activations.len();
        let mut output = self.hidden_layer.biases[bucket] as i32;

        let weights = self.hidden_layer.weights
            [bucket_idx..bucket_idx + self.input_layer.activations.len()]
            .iter();

        self.input_layer
            .activations
            .iter()
            .map(|x| Self::clipped_relu(*x))
            .zip(weights)
            .for_each(|(clipped_activation, weight)| {
                output += (clipped_activation as i32) * (*weight as i32)
            });
        (output / (Self::SCALE.pow(2)) as i32) as Score
    }

    #[inline(always)]
    fn clipped_relu(x: i16) -> i16 {
        x.max(0).min(Self::SCALE)
    }
}

impl Network {
    const SCALE: i16 = 64;
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}
