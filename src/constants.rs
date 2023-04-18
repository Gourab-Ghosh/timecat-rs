pub const VERSION: &str = env!("CARGO_PKG_VERSION");

macro_rules! make_array_recursively {
    ($($x:expr),*) => ([$(make_array_recursively!($x)),*]);
    ($x:expr) => ($x);
}

macro_rules! generate_lmr_table {
    () => {
        let mut lmr_table = [[0; 64]; 64];
        for depth in 1..MAX_PLY {
            for move_number in 1..MAX_PLY {
                let reduction = (depth as f64).ln() * (move_number as f64).ln() / 3.0;
                lmr_table[depth][move_number] = reduction as usize;
            }
        }
        lmr_table
    };
}

pub mod types {
    pub type Ply = usize;
    pub type Depth = i8;
    pub type Score = i16;
    pub type MoveWeight = i32;
}

pub mod bitboard {
    use chess::BitBoard;

    pub const BB_EMPTY: BitBoard = BitBoard(0);
    pub const BB_ALL: BitBoard = BitBoard(0xffff_ffff_ffff_ffff);

    pub const BB_A1: BitBoard = BitBoard(1 << 0);
    pub const BB_B1: BitBoard = BitBoard(1 << 1);
    pub const BB_C1: BitBoard = BitBoard(1 << 2);
    pub const BB_D1: BitBoard = BitBoard(1 << 3);
    pub const BB_E1: BitBoard = BitBoard(1 << 4);
    pub const BB_F1: BitBoard = BitBoard(1 << 5);
    pub const BB_G1: BitBoard = BitBoard(1 << 6);
    pub const BB_H1: BitBoard = BitBoard(1 << 7);
    pub const BB_A2: BitBoard = BitBoard(1 << 8);
    pub const BB_B2: BitBoard = BitBoard(1 << 9);
    pub const BB_C2: BitBoard = BitBoard(1 << 10);
    pub const BB_D2: BitBoard = BitBoard(1 << 11);
    pub const BB_E2: BitBoard = BitBoard(1 << 12);
    pub const BB_F2: BitBoard = BitBoard(1 << 13);
    pub const BB_G2: BitBoard = BitBoard(1 << 14);
    pub const BB_H2: BitBoard = BitBoard(1 << 15);
    pub const BB_A3: BitBoard = BitBoard(1 << 16);
    pub const BB_B3: BitBoard = BitBoard(1 << 17);
    pub const BB_C3: BitBoard = BitBoard(1 << 18);
    pub const BB_D3: BitBoard = BitBoard(1 << 19);
    pub const BB_E3: BitBoard = BitBoard(1 << 20);
    pub const BB_F3: BitBoard = BitBoard(1 << 21);
    pub const BB_G3: BitBoard = BitBoard(1 << 22);
    pub const BB_H3: BitBoard = BitBoard(1 << 23);
    pub const BB_A4: BitBoard = BitBoard(1 << 24);
    pub const BB_B4: BitBoard = BitBoard(1 << 25);
    pub const BB_C4: BitBoard = BitBoard(1 << 26);
    pub const BB_D4: BitBoard = BitBoard(1 << 27);
    pub const BB_E4: BitBoard = BitBoard(1 << 28);
    pub const BB_F4: BitBoard = BitBoard(1 << 29);
    pub const BB_G4: BitBoard = BitBoard(1 << 30);
    pub const BB_H4: BitBoard = BitBoard(1 << 31);
    pub const BB_A5: BitBoard = BitBoard(1 << 32);
    pub const BB_B5: BitBoard = BitBoard(1 << 33);
    pub const BB_C5: BitBoard = BitBoard(1 << 34);
    pub const BB_D5: BitBoard = BitBoard(1 << 35);
    pub const BB_E5: BitBoard = BitBoard(1 << 36);
    pub const BB_F5: BitBoard = BitBoard(1 << 37);
    pub const BB_G5: BitBoard = BitBoard(1 << 38);
    pub const BB_H5: BitBoard = BitBoard(1 << 39);
    pub const BB_A6: BitBoard = BitBoard(1 << 40);
    pub const BB_B6: BitBoard = BitBoard(1 << 41);
    pub const BB_C6: BitBoard = BitBoard(1 << 42);
    pub const BB_D6: BitBoard = BitBoard(1 << 43);
    pub const BB_E6: BitBoard = BitBoard(1 << 44);
    pub const BB_F6: BitBoard = BitBoard(1 << 45);
    pub const BB_G6: BitBoard = BitBoard(1 << 46);
    pub const BB_H6: BitBoard = BitBoard(1 << 47);
    pub const BB_A7: BitBoard = BitBoard(1 << 48);
    pub const BB_B7: BitBoard = BitBoard(1 << 49);
    pub const BB_C7: BitBoard = BitBoard(1 << 50);
    pub const BB_D7: BitBoard = BitBoard(1 << 51);
    pub const BB_E7: BitBoard = BitBoard(1 << 52);
    pub const BB_F7: BitBoard = BitBoard(1 << 53);
    pub const BB_G7: BitBoard = BitBoard(1 << 54);
    pub const BB_H7: BitBoard = BitBoard(1 << 55);
    pub const BB_A8: BitBoard = BitBoard(1 << 56);
    pub const BB_B8: BitBoard = BitBoard(1 << 57);
    pub const BB_C8: BitBoard = BitBoard(1 << 58);
    pub const BB_D8: BitBoard = BitBoard(1 << 59);
    pub const BB_E8: BitBoard = BitBoard(1 << 60);
    pub const BB_F8: BitBoard = BitBoard(1 << 61);
    pub const BB_G8: BitBoard = BitBoard(1 << 62);
    pub const BB_H8: BitBoard = BitBoard(1 << 63);

    #[rustfmt::skip]
    pub const BB_SQUARES: [BitBoard; 64] = [
        BB_A1, BB_B1, BB_C1, BB_D1, BB_E1, BB_F1, BB_G1, BB_H1,
        BB_A2, BB_B2, BB_C2, BB_D2, BB_E2, BB_F2, BB_G2, BB_H2,
        BB_A3, BB_B3, BB_C3, BB_D3, BB_E3, BB_F3, BB_G3, BB_H3,
        BB_A4, BB_B4, BB_C4, BB_D4, BB_E4, BB_F4, BB_G4, BB_H4,
        BB_A5, BB_B5, BB_C5, BB_D5, BB_E5, BB_F5, BB_G5, BB_H5,
        BB_A6, BB_B6, BB_C6, BB_D6, BB_E6, BB_F6, BB_G6, BB_H6,
        BB_A7, BB_B7, BB_C7, BB_D7, BB_E7, BB_F7, BB_G7, BB_H7,
        BB_A8, BB_B8, BB_C8, BB_D8, BB_E8, BB_F8, BB_G8, BB_H8,
    ];

    pub const BB_CORNERS: BitBoard = BitBoard(BB_A1.0 | BB_H1.0 | BB_A8.0 | BB_H8.0);
    pub const BB_CENTER: BitBoard = BitBoard(BB_D4.0 | BB_E4.0 | BB_D5.0 | BB_E5.0);

    pub const BB_LIGHT_SQUARES: BitBoard = BitBoard(0x55aa_55aa_55aa_55aa);
    pub const BB_DARK_SQUARES: BitBoard = BitBoard(0xaa55_aa55_aa55_aa55);

    pub const BB_FILE_A: BitBoard = BitBoard(0x0101_0101_0101_0101);
    pub const BB_FILE_B: BitBoard = BitBoard(0x0101_0101_0101_0101 << 1);
    pub const BB_FILE_C: BitBoard = BitBoard(0x0101_0101_0101_0101 << 2);
    pub const BB_FILE_D: BitBoard = BitBoard(0x0101_0101_0101_0101 << 3);
    pub const BB_FILE_E: BitBoard = BitBoard(0x0101_0101_0101_0101 << 4);
    pub const BB_FILE_F: BitBoard = BitBoard(0x0101_0101_0101_0101 << 5);
    pub const BB_FILE_G: BitBoard = BitBoard(0x0101_0101_0101_0101 << 6);
    pub const BB_FILE_H: BitBoard = BitBoard(0x0101_0101_0101_0101 << 7);

    pub const BB_RANK_1: BitBoard = BitBoard(0xff);
    pub const BB_RANK_2: BitBoard = BitBoard(0xff << (1 << 3));
    pub const BB_RANK_3: BitBoard = BitBoard(0xff << (2 << 3));
    pub const BB_RANK_4: BitBoard = BitBoard(0xff << (3 << 3));
    pub const BB_RANK_5: BitBoard = BitBoard(0xff << (4 << 3));
    pub const BB_RANK_6: BitBoard = BitBoard(0xff << (5 << 3));
    pub const BB_RANK_7: BitBoard = BitBoard(0xff << (6 << 3));
    pub const BB_RANK_8: BitBoard = BitBoard(0xff << (7 << 3));

    pub const BB_BACKRANKS: BitBoard = BitBoard(BB_RANK_1.0 | BB_RANK_8.0);

    pub const BB_UPPER_HALF_BOARD: BitBoard = BitBoard(0xffffffff00000000);
    pub const BB_LOWER_HALF_BOARD: BitBoard = BitBoard(0x00000000ffffffff);
    pub const BB_LEFT_HALF_BOARD: BitBoard = BitBoard(0xf0f0f0f0f0f0f0f0);
    pub const BB_RIGHT_HALF_BOARD: BitBoard = BitBoard(0x0f0f0f0f0f0f0f0f);

    pub const CENTER_SQUARES_BB: BitBoard = BitBoard(0x0000001818000000);
    pub const PSEUDO_CENTER_SQUARES_BB: BitBoard = BitBoard(0x00003C24243C0000);

    pub const UPPER_BOARD_MASK: [[BitBoard; 8]; 2] = [
        [
            BitBoard(0xffff_ffff_ffff_ff00),
            BitBoard(0xffff_ffff_ffff_0000),
            BitBoard(0xffff_ffff_ff00_0000),
            BitBoard(0xffff_ffff_0000_0000),
            BitBoard(0xffff_ff00_0000_0000),
            BitBoard(0xffff_0000_0000_0000),
            BitBoard(0xff00_0000_0000_0000),
            BitBoard(0x0000_0000_0000_0000),
        ],
        [
            BitBoard(0x00ff_ffff_ffff_ffff),
            BitBoard(0x0000_ffff_ffff_ffff),
            BitBoard(0x0000_00ff_ffff_ffff),
            BitBoard(0x0000_0000_ffff_ffff),
            BitBoard(0x0000_0000_00ff_ffff),
            BitBoard(0x0000_0000_0000_ffff),
            BitBoard(0x0000_0000_0000_00ff),
            BitBoard(0x0000_0000_0000_0000),
        ],
    ];

    pub const BOARD_QUARTER_MASKS: [BitBoard; 4] = [
        BitBoard(0x0f0f_0f0f_0000_0000),
        BitBoard(0xf0f0_f0f0_0000_0000),
        BitBoard(0x0000_0000_0f0f_0f0f),
        BitBoard(0x0000_0000_f0f0_f0f0),
    ];
}

pub mod square {
    use chess::Square;
    #[rustfmt::skip]
    pub const SQUARES_180: [Square; 64] = [
        Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
        Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
        Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
        Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
        Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
        Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
        Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
        Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
    ];
}

pub mod board_representation {
    pub const BOARD_SKELETON: &str = r"

     A   B   C   D   E   F   G   H
   +---+---+---+---+---+---+---+---+
 8 | O | O | O | O | O | O | O | O | 8
   +---+---+---+---+---+---+---+---+
 7 | O | O | O | O | O | O | O | O | 7
   +---+---+---+---+---+---+---+---+
 6 | O | O | O | O | O | O | O | O | 6
   +---+---+---+---+---+---+---+---+
 5 | O | O | O | O | O | O | O | O | 5
   +---+---+---+---+---+---+---+---+
 4 | O | O | O | O | O | O | O | O | 4
   +---+---+---+---+---+---+---+---+
 3 | O | O | O | O | O | O | O | O | 3
   +---+---+---+---+---+---+---+---+
 2 | O | O | O | O | O | O | O | O | 2
   +---+---+---+---+---+---+---+---+
 1 | O | O | O | O | O | O | O | O | 1
   +---+---+---+---+---+---+---+---+
     A   B   C   D   E   F   G   H

";

    pub const PIECE_SYMBOLS: [&str; 7] = [" ", "p", "n", "b", "r", "q", "k"];
    pub const WHITE_PIECE_UNICODE_SYMBOLS: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
    pub const BLACK_PIECE_UNICODE_SYMBOLS: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
    pub const EMPTY_SPACE_UNICODE_SYMBOL: &str = " ";
}

pub mod fen {
    pub const EMPTY_BOARD_FEN: &str = "8/8/8/8/8/8/8/8 w - - 0 1";
    pub const STARTING_BOARD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
}

pub mod print_style {
    pub const WHITE_PIECES_STYLE: &str = "white bold";
    pub const BLACK_PIECES_STYLE: &str = "purple bold";
    pub const BOARD_SKELETON_STYLE: &str = "green";
    pub const BOARD_LABEL_STYLE: &str = "red bold";
    pub const INFO_STYLE: &str = "bright_cyan bold";
    pub const CHECK_STYLE: &str = "on_bright_red";
    pub const CHECKERS_STYLE: &str = "bright_red bold";
    pub const CHECKMATE_SCORE_STYLE: &str = "bright_red bold";
    pub const PERFT_MOVE_STYLE: &str = "green bold";
    pub const PERFT_COUNT_STYLE: &str = "";
    pub const INPUT_MESSAGE_STYLE: &str = "blue bold";
    pub const SUCCESS_MESSAGE_STYLE: &str = "green bold";
    pub const ERROR_MESSAGE_STYLE: &str = "red bold";
    pub const LAST_MOVE_HIGHLIGHT_STYLE: &str = "on_bright_black";
    pub const WARNING_MESSAGE_STYLE: &str = "bright_yellow bold";
}

pub mod engine_constants {
    use std::time::Duration;

    use crate::utils::common_utils::evaluate_piece;
    use chess::Piece::*;

    use super::types::*;
    pub const MAX_PLY: usize = 255;
    pub const INFINITY: Score = 30_000;
    pub const DRAW_SCORE: Score = PAWN_VALUE / 2;
    pub const CHECKMATE_SCORE: Score = 25_000;
    pub const CHECKMATE_THRESHOLD: Score = CHECKMATE_SCORE - MAX_PLY as Score - 1;
    pub const NUM_KILLER_MOVES: usize = 3;
    pub const PAWN_VALUE: Score = 100;

    pub const DISABLE_ALL_PRUNINGS: bool = false;

    pub const NULL_MOVE_MIN_DEPTH: Depth = 2;
    pub const NULL_MOVE_MIN_REDUCTION: Depth = 2;
    pub const NULL_MOVE_DEPTH_DIVIDER: Depth = 4;

    pub const FULL_DEPTH_SEARCH_LMR: usize = 4;
    pub const REDUCTION_LIMIT_LMR: Depth = 3;
    pub const LMR_BASE_REDUCTION: f32 = 0.75;
    pub const LMR_MOVE_DIVIDER: f32 = 2.25;
    pub const DISABLE_LMR: bool = false || DISABLE_ALL_PRUNINGS;

    pub const ASPIRATION_WINDOW_CUTOFF: Score = PAWN_VALUE / 2;
    pub const DISABLE_T_TABLE: bool = false;
    pub const MAX_MOVES_PER_POSITION: usize = 250;
    pub const ENDGAME_PIECE_THRESHOLD: u32 = 12;

    pub const FOLLOW_PV: bool = true;
    pub const PRINT_MOVE_INFO_DURATION_THRESHOLD: Duration = Duration::from_millis(1000);

    pub const INITIAL_MATERIAL_SCORE_ABS: Score = 16 * PAWN_VALUE
        + 4 * (evaluate_piece(Knight) + evaluate_piece(Bishop) + evaluate_piece(Rook))
        + 2 * evaluate_piece(Queen);

    #[rustfmt::skip]
    pub const MVV_LVA: [[MoveWeight; 6]; 6] = [
        [105, 205, 305, 405, 505, 605],
        [104, 204, 304, 404, 504, 604],
        [103, 203, 303, 403, 503, 603],
        [102, 202, 302, 402, 502, 602],
        [101, 201, 301, 401, 501, 601],
        [100, 200, 300, 400, 500, 600],
    ];

    pub const LMR_TABLE: [[Depth; 64]; 64] = [[0; 64]; 64];
}
