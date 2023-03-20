use super::*;
use nnue::{Network, Piece as P, StockfishNetwork};

#[derive(Default, Debug)]
pub struct Evaluator {
    network: Network,
    stockfish_network: StockfishNetwork,
    network_backup: Vec<Network>,
    cache: HashMap<u64, Score>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            network: Network::new(),
            stockfish_network: StockfishNetwork::new(),
            network_backup: Vec::new(),
            cache: HashMap::default(),
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
        self.network.activate(
            self.convert_piece_to_p_piece(piece, color),
            square.to_index(),
        );
    }

    pub fn deactivate_nnue(&mut self, piece: Piece, color: Color, square: Square) {
        self.network.deactivate(
            self.convert_piece_to_p_piece(piece, color),
            square.to_index(),
        );
    }

    pub fn backup(&mut self) {
        self.network_backup.push(self.network.clone());
    }

    pub fn restore(&mut self) {
        self.network = self.network_backup.pop().expect("No network backup found!");
    }

    fn force_king_to_corner(&self, board: &Board) -> Score {
        let self_king_square = board.get_king_square(board.turn());
        let opponent_king_square = board.get_king_square(!board.turn());
        let kings_distance = square_distance(self_king_square, opponent_king_square);
        let mut least_distance_corner = Square::default();
        for (bb, &corner_square) in BOARD_QUARTER_MASKS.iter().zip([Square::A8, Square::H8, Square::A1, Square::H1].iter()) {
            if bb & get_square_bb(self_king_square) != BB_EMPTY {
                least_distance_corner = corner_square;
                break;
            }
        }
        let mut score = (8 - kings_distance as Score) * PAWN_VALUE / 2;
        score += (8 - square_distance(opponent_king_square, least_distance_corner) as Score) * PAWN_VALUE;
        score
    }

    fn force_king_to_center(&self, board: &Board) -> Score {
        let self_king_square = board.get_king_square(board.turn());
        let mut least_distance_center = Square::default();
        for (bb, &corner_square) in BOARD_QUARTER_MASKS.iter().zip([Square::D5, Square::E5, Square::D4, Square::E4].iter()) {
            if bb & get_square_bb(self_king_square) != BB_EMPTY {
                least_distance_center = corner_square;
                break;
            }
        }
        (8 - square_distance(self_king_square, least_distance_center) as Score) * PAWN_VALUE
    }

    pub fn evaluate(&self, board: &Board) -> Score {
        let mut score = self.stockfish_network.eval(board);
        let corner_forcing_score = self.force_king_to_corner(board);
        let center_forcing_score = self.force_king_to_corner(board);
        if board.get_num_pieces() < 6 && score.abs() > evaluate_piece(Knight) {
            if (if board.turn() == White { score } else { -score }).is_positive() {
                score += if board.turn() == White { corner_forcing_score } else { -corner_forcing_score };
            } else {
                score += if board.turn() == White { center_forcing_score } else { -center_forcing_score };
            }
        }
        score
    }
}
