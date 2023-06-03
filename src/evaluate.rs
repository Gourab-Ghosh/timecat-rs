use super::*;
use nnue::{Network, Piece as P, StockfishNetwork};

pub struct Evaluator {
    network: Network,
    stockfish_network: StockfishNetwork,
    network_backup: Vec<Network>,
    cache: CacheTable<Score>,
}

impl Evaluator {
    pub fn new() -> Self {
        let (size, entry_size) = CacheTableSize::Max(20).to_cache_table_and_entry_size::<Score>();
        println_info(
            "Evaluator Cache size",
            format!("{} MB", size * entry_size / 2_usize.pow(20)),
        );
        Self {
            network: Network::new(),
            stockfish_network: StockfishNetwork::new(),
            network_backup: Vec::new(),
            cache: CacheTable::new(size, 0),
        }
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
        for (bb, &corner_square) in BOARD_QUARTER_MASKS
            .iter()
            .zip([Square::A8, Square::H8, Square::A1, Square::H1].iter())
        {
            if bb & get_square_bb(self_king_square) != BB_EMPTY {
                least_distance_corner = corner_square;
                break;
            }
        }
        let mut score = (8 - kings_distance as Score) * PAWN_VALUE;
        score += (16
            - opponent_king_square
                .get_rank()
                .to_index()
                .abs_diff(least_distance_corner.get_rank().to_index()) as Score
            - opponent_king_square
                .get_file()
                .to_index()
                .abs_diff(least_distance_corner.get_file().to_index()) as Score)
            * PAWN_VALUE;
        board.score_flipped(score)
    }

    fn force_king_to_center(&self, board: &Board) -> Score {
        let self_king_square = board.get_king_square(board.turn());
        let mut least_distance_center = Square::default();
        for (bb, &corner_square) in BOARD_QUARTER_MASKS
            .iter()
            .zip([Square::D5, Square::E5, Square::D4, Square::E4].iter())
        {
            if bb & get_square_bb(self_king_square) != BB_EMPTY {
                least_distance_center = corner_square;
                break;
            }
        }
        let score =
            (8 - square_distance(self_king_square, least_distance_center) as Score) * PAWN_VALUE;
        board.score_flipped(score)
    }

    pub fn evaluate_immutable(&self, sub_board: &chess::Board) -> Score {
        // if board.is_endgame() {
        //     let mut eval = self.network.eval(board);
        //     if eval.abs() < 0 {
        //         return self.stockfish_network.eval(board);
        //     }
        //     eval = eval +  if eval.is_positive() { self.force_king_to_center(board) } else { self.force_king_to_corner(board) } / 10;
        //     return 50 * PAWN_VALUE * eval.signum() + eval;
        // }
        let mut score = self.stockfish_network.eval(sub_board);
        let mut piece_diff_score = 0;
        let black_occupied = sub_board.color_combined(Black);
        for &piece in chess::ALL_PIECES[0..5].iter() {
            let piece_mask = sub_board.pieces(piece);
            if piece_mask == &BB_EMPTY {
                continue;
            }
            piece_diff_score += (piece_mask.popcnt() as Score
                - 2 * (piece_mask & black_occupied).popcnt() as Score)
                * evaluate_piece(piece);
        }
        score += piece_diff_score / 20;
        score
    }

    pub fn evaluate(&mut self, sub_board: &chess::Board) -> Score {
        let hash = sub_board.get_hash();
        if let Some(score) = self.cache.get(hash) {
            return score;
        }
        let score = self.evaluate_immutable(sub_board);
        self.cache.add(hash, score);
        score
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}