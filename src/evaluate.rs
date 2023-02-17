use super::*;
use nnue::{Network, Piece as P, StockfishNetwork};

#[derive(Default)]
pub struct Evaluator {
    network: Network,
    stockfish_network: StockfishNetwork,
    network_backup: Vec<Network>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            network: Network::new(),
            stockfish_network: StockfishNetwork::new(),
            network_backup: Vec::new(),
        }
    }

    pub fn default() -> Self {
        Self::new()
    }

    fn convert_piece_to_p_piece(&self, piece: Piece, color: Color) -> P {
        match color {
            Color::White => match piece {
                Piece::Pawn => P::WhitePawn,
                Piece::Knight => P::WhiteKnight,
                Piece::Bishop => P::WhiteBishop,
                Piece::Rook => P::WhiteRook,
                Piece::Queen => P::WhiteQueen,
                Piece::King => P::WhiteKing,
            },
            Color::Black => match piece {
                Piece::Pawn => P::BlackPawn,
                Piece::Knight => P::BlackKnight,
                Piece::Bishop => P::BlackBishop,
                Piece::Rook => P::BlackRook,
                Piece::Queen => P::BlackQueen,
                Piece::King => P::BlackKing,
            },
        }
    }

    pub fn activate_nnue(&mut self, piece: Piece, color: Color, square: Square) {
        // self.network.activate(
        //     self.convert_piece_to_p_piece(piece, color),
        //     square.to_index(),
        // );
    }

    pub fn deactivate_nnue(&mut self, piece: Piece, color: Color, square: Square) {
        // self.network.deactivate(
        //     self.convert_piece_to_p_piece(piece, color),
        //     square.to_index(),
        // );
    }

    pub fn backup(&mut self) {
        // self.network_backup.push(self.network.clone());
    }

    pub fn restore(&mut self) {
        // self.network = self.network_backup.pop().expect("No network backup found!");
    }

    pub fn evaluate(&self, board: &Board) -> i16 {
        // let mut score = self.stockfish_network.eval(board);
        // if board.is_endgame() {
        //     score += self.network.eval(board);
        // }
        // return score as i16;
        self.stockfish_network.eval(board) as i16
    }
}
