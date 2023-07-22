use super::*;
use binread::BinRead;
// use board::Board;
use nnue_rs::stockfish::halfkp::{SfHalfKpFullModel, SfHalfKpModel};
use nnue_rs::Square as StockfishSquare;
use std::io::Cursor;

fn square_to_stockfish_square(square: Square) -> StockfishSquare {
    StockfishSquare::from_index(square.to_index())
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BinRead)]
#[derive(Debug)]
pub struct StockfishNetwork {
    model: SfHalfKpModel,
}

impl StockfishNetwork {
    pub fn new() -> Self {
        let mut reader = Cursor::new(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/nnue_dir/nn.nnue"
        )));
        let model = SfHalfKpFullModel::read(&mut reader).expect("Bad NNUE file!");
        Self { model: model.model }
    }

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
