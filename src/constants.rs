use super::*;

pub mod description {
    pub const ENGINE_NAME: &str = "Timecat";
    pub const ENGINE_AUTHOR: &str = "Gourab Ghosh";
    pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod types {
    use super::TimecatError;

    pub type Ply = usize;
    pub type Depth = i8;
    pub type Score = i16;
    pub type MoveWeight = i64;
    pub type NumMoves = u16;
    pub type Spin = u128;

    #[cfg(feature = "colored")]
    pub type ColoredStringFunction = fn(colored::ColoredString) -> colored::ColoredString;

    pub type Result<T> = std::result::Result<T, TimecatError>;
}

pub mod bitboard_and_square {
    use super::*;
    use File::*;

    pub const BB_EMPTY: BitBoard = BitBoard::new(0);
    pub const BB_ALL: BitBoard = BitBoard::new(0xffff_ffff_ffff_ffff);
    pub const NUM_SQUARES: usize = 64;

    macro_rules! generate_bitboard_and_square_constants {
        (@bb_squares $(($file:expr, $rank:expr)), *$(,)?) => {
            paste! {
                $(
                    pub const [<BB_$file$rank>]: BitBoard = BitBoard::new(1 << (8 * ($rank - 1) + $file as usize));
                )*
                pub const ALL_SQUARES: [Square; NUM_SQUARES] = [$( Square::[<$file$rank>] ), *];
                pub const BB_SQUARES: [BitBoard; NUM_SQUARES] = [$( [<BB_$file$rank>] ), *];
                pub const SQUARES_VERTICAL_MIRROR: [Square; NUM_SQUARES] = [$( Square::[<$file$rank>].vertical_mirror() ), *];
                pub const SQUARES_HORIZONTAL_MIRROR: [Square; NUM_SQUARES] = [$( Square::[<$file$rank>].horizontal_mirror() ), *];
            }
        };

        (@bb_ranks_and_files $(($file:expr, $rank:expr)), *$(,)?) => {
            $(
                paste!{
                    pub const [<BB_RANK_$rank>]: BitBoard = BitBoard::new(0xff << (($rank - 1) << 3));
                    pub const [<BB_FILE_$file>]: BitBoard = BitBoard::new(0x0101_0101_0101_0101 << ($rank - 1));
                }
            )*
        };
    }

    #[rustfmt::skip]
    generate_bitboard_and_square_constants!(
        @bb_squares
        (A, 1), (B, 1), (C, 1), (D, 1), (E, 1), (F, 1), (G, 1), (H, 1),
        (A, 2), (B, 2), (C, 2), (D, 2), (E, 2), (F, 2), (G, 2), (H, 2),
        (A, 3), (B, 3), (C, 3), (D, 3), (E, 3), (F, 3), (G, 3), (H, 3),
        (A, 4), (B, 4), (C, 4), (D, 4), (E, 4), (F, 4), (G, 4), (H, 4),
        (A, 5), (B, 5), (C, 5), (D, 5), (E, 5), (F, 5), (G, 5), (H, 5),
        (A, 6), (B, 6), (C, 6), (D, 6), (E, 6), (F, 6), (G, 6), (H, 6),
        (A, 7), (B, 7), (C, 7), (D, 7), (E, 7), (F, 7), (G, 7), (H, 7),
        (A, 8), (B, 8), (C, 8), (D, 8), (E, 8), (F, 8), (G, 8), (H, 8),
    );
    generate_bitboard_and_square_constants!(
        @bb_ranks_and_files
        (A, 1), (B, 2), (C, 3), (D, 4), (E, 5), (F, 6), (G, 7), (H, 8),
    );

    pub const BB_CORNERS: BitBoard =
        BitBoard::new(BB_A1.get_mask() ^ BB_H1.get_mask() ^ BB_A8.get_mask() ^ BB_H8.get_mask());
    pub const BB_CENTER: BitBoard =
        BitBoard::new(BB_D4.get_mask() ^ BB_E4.get_mask() ^ BB_D5.get_mask() ^ BB_E5.get_mask());

    pub const BB_LIGHT_SQUARES: BitBoard = BitBoard::new(0x55aa_55aa_55aa_55aa);
    pub const BB_DARK_SQUARES: BitBoard = BitBoard::new(0xaa55_aa55_aa55_aa55);

    pub const BB_BACKRANKS: BitBoard = BitBoard::new(BB_RANK_1.get_mask() ^ BB_RANK_8.get_mask());

    pub const BB_UPPER_HALF_BOARD: BitBoard = BitBoard::new(0xffffffff00000000);
    pub const BB_LOWER_HALF_BOARD: BitBoard = BitBoard::new(0x00000000ffffffff);
    pub const BB_LEFT_HALF_BOARD: BitBoard = BitBoard::new(0xf0f0f0f0f0f0f0f0);
    pub const BB_RIGHT_HALF_BOARD: BitBoard = BitBoard::new(0x0f0f0f0f0f0f0f0f);

    pub const CENTER_SQUARES_BB: BitBoard = BitBoard::new(0x0000001818000000);
    pub const PSEUDO_CENTER_SQUARES_BB: BitBoard = BitBoard::new(0x00003C24243C0000);

    pub const UPPER_BOARD_MASK: [[BitBoard; 8]; 2] = [
        [
            BitBoard::new(0xffff_ffff_ffff_ff00),
            BitBoard::new(0xffff_ffff_ffff_0000),
            BitBoard::new(0xffff_ffff_ff00_0000),
            BitBoard::new(0xffff_ffff_0000_0000),
            BitBoard::new(0xffff_ff00_0000_0000),
            BitBoard::new(0xffff_0000_0000_0000),
            BitBoard::new(0xff00_0000_0000_0000),
            BitBoard::new(0x0000_0000_0000_0000),
        ],
        [
            BitBoard::new(0x00ff_ffff_ffff_ffff),
            BitBoard::new(0x0000_ffff_ffff_ffff),
            BitBoard::new(0x0000_00ff_ffff_ffff),
            BitBoard::new(0x0000_0000_ffff_ffff),
            BitBoard::new(0x0000_0000_00ff_ffff),
            BitBoard::new(0x0000_0000_0000_ffff),
            BitBoard::new(0x0000_0000_0000_00ff),
            BitBoard::new(0x0000_0000_0000_0000),
        ],
    ];

    pub const BOARD_QUARTER_MASKS: [BitBoard; 4] = [
        BitBoard::new(0x0f0f_0f0f_0000_0000),
        BitBoard::new(0xf0f0_f0f0_0000_0000),
        BitBoard::new(0x0000_0000_0f0f_0f0f),
        BitBoard::new(0x0000_0000_f0f0_f0f0),
    ];
}

pub mod board {
    pub const EMPTY_SPACE_SYMBOL: &str = " ";
    pub const EMPTY_SPACE_UNICODE_SYMBOL: &str = " ";
    pub const WHITE_PIECE_UNICODE_SYMBOLS: [&str; 6] = ["♙", "♘", "♗", "♖", "♕", "♔"];
    pub const BLACK_PIECE_UNICODE_SYMBOLS: [&str; 6] = ["♟", "♞", "♝", "♜", "♛", "♚"];
}

pub mod fen {
    pub const EMPTY_FEN: &str = "8/8/8/8/8/8/8/8 w - - 0 1";
    pub const STARTING_POSITION_FEN: &str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
}

pub mod print_style {
    use super::*;

    #[cfg(feature = "colored")]
    macro_rules! generate_constants {
        ($constant_name:ident, [$( $func_name:ident ), *]) => {
            pub const $constant_name: &[ColoredStringFunction] = &[$( colored::Colorize::$func_name ), *];
        };
    }

    #[cfg(not(feature = "colored"))]
    macro_rules! generate_constants {
        ($constant_name:ident, [$( $_:ident ), *]) => {
            pub const $constant_name: &[fn(String) -> String] = &[identity_function];
        };
    }

    generate_constants!(WHITE_PIECES_STYLE, [white, bold]);
    generate_constants!(BLACK_PIECES_STYLE, [purple, bold]);
    generate_constants!(BITBOARD_OCCUPIED_SQUARE_STYLE, [white, bold]);
    generate_constants!(BOARD_SKELETON_STYLE, [green]);
    generate_constants!(BOARD_LABEL_STYLE, [red, bold]);
    generate_constants!(INFO_MESSAGE_STYLE, [bright_cyan, bold]);
    generate_constants!(CHECK_STYLE, [on_bright_red]);
    generate_constants!(CHECKERS_STYLE, [bright_red, bold]);
    generate_constants!(CHECKMATE_SCORE_STYLE, [bright_red, bold]);
    generate_constants!(PERFT_MOVE_STYLE, [green, bold]);
    generate_constants!(PERFT_COUNT_STYLE, []);
    generate_constants!(INPUT_MESSAGE_STYLE, [blue, bold]);
    generate_constants!(SUCCESS_MESSAGE_STYLE, [green, bold]);
    generate_constants!(ERROR_MESSAGE_STYLE, [red, bold]);
    generate_constants!(LAST_MOVE_HIGHLIGHT_STYLE, [on_bright_black]);
    generate_constants!(WARNING_MESSAGE_STYLE, [bright_yellow, bold]);
}

pub mod evaluate {
    use super::*;

    pub const ENDGAME_PIECE_THRESHOLD: u32 = 12;
    pub const EVALUATOR_SIZE: CacheTableSize = CacheTableSize::Exact(16);
    pub const DRAW_SCORE: Score = PAWN_VALUE / 2;
    pub const CHECKMATE_SCORE: Score = 25_000;
    pub const CHECKMATE_THRESHOLD: Score = CHECKMATE_SCORE - MAX_PLY as Score - 1;
    pub const INFINITY: Score = CHECKMATE_SCORE + 4 * MAX_PLY as Score;
    pub const PAWN_VALUE: Score = 100;
    pub const MAX_PLY: usize = 255;
    pub const INITIAL_MATERIAL_SCORE_ABS: Score = 16 * PAWN_VALUE
        + 4 * (Knight.evaluate() + Bishop.evaluate() + Rook.evaluate())
        + 2 * Queen.evaluate();
    pub const MAX_MATERIAL_SCORE: Score = INITIAL_MATERIAL_SCORE_ABS / 2;
    pub const WINNING_SCORE_THRESHOLD: Score = 15 * PAWN_VALUE;
}

pub mod cache_table {
    use super::*;

    // pub const DEFAULT_HASH: NonZeroU64 = NonZeroU64::new(1).unwrap();
    pub const DEFAULT_HASH: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1) };
}

#[cfg(feature = "engine")]
pub mod engine {
    use super::*;

    pub const DEFAULT_SELFPLAY_COMMAND: GoCommand = GoCommand::from_millis(3000);

    pub const CLEAR_TABLE_AFTER_EACH_SEARCH: bool = true;
    pub const NUM_KILLER_MOVES: usize = 3;

    pub const DISABLE_ALL_PRUNINGS: bool = false;
    pub const DISABLE_LMR: bool = false || DISABLE_ALL_PRUNINGS;

    pub const NULL_MOVE_MIN_DEPTH: Depth = 2;
    pub const NULL_MOVE_MIN_REDUCTION: Depth = 2;
    pub const NULL_MOVE_DEPTH_DIVIDER: Depth = 4;

    pub const FULL_DEPTH_SEARCH_LMR: usize = 4;
    pub const REDUCTION_LIMIT_LMR: Depth = 3;
    pub const LMR_BASE_REDUCTION: f64 = 0.75;
    pub const LMR_MOVE_DIVIDER: f64 = 2.25;

    pub const ASPIRATION_WINDOW_CUTOFF: Score = PAWN_VALUE / 2;
    pub const MAX_MOVES_PER_POSITION: usize = 250;

    pub const FOLLOW_PV: bool = true;
    pub const PRINT_MOVE_INFO_DURATION_THRESHOLD: Duration = Duration::from_millis(1000);

    pub const NUM_BEST_ROOT_MOVES_TO_SEARCH_FIRST: usize = 3;

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

pub mod io {
    use super::*;

    pub const COMMUNICATION_CHECK_INTERVAL: Duration = Duration::from_millis(1);
}

pub mod atomic {
    pub const MEMORY_ORDERING: std::sync::atomic::Ordering = std::sync::atomic::Ordering::Relaxed;
}

pub mod color {
    use super::*;

    pub const NUM_COLORS: usize = 2;
    pub const ALL_COLORS: [Color; NUM_COLORS] = [Color::White, Color::Black];
}

pub mod piece {
    use super::*;

    pub const NUM_PIECE_TYPES: usize = 6;
    pub const ALL_PIECE_TYPES: [PieceType; NUM_PIECE_TYPES] =
        [Pawn, Knight, Bishop, Rook, Queen, King];
    pub const NUM_PROMOTION_PIECES: usize = 4;
    pub const PROMOTION_PIECES: [PieceType; NUM_PROMOTION_PIECES] = [Queen, Knight, Rook, Bishop];
}

pub mod ranks {
    use super::*;

    pub const NUM_RANKS: usize = 8;
    pub const ALL_RANKS: [Rank; NUM_RANKS] = [
        Rank::First,
        Rank::Second,
        Rank::Third,
        Rank::Fourth,
        Rank::Fifth,
        Rank::Sixth,
        Rank::Seventh,
        Rank::Eighth,
    ];
}

pub mod files {
    use super::*;

    pub const NUM_FILES: usize = 8;
    pub const ALL_FILES: [File; NUM_FILES] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];
}

pub mod default_parameters {
    use super::*;

    pub const TIMECAT_DEFAULTS: TimecatDefaults = TimecatDefaults {
        #[cfg(feature = "colored")]
        colored: true,
        console_mode: true,
        t_table_size: CacheTableSize::Exact(16),
        long_algebraic_notation: false,
        num_threads: unsafe { NonZeroUsize::new_unchecked(1) },
        move_overhead: Duration::from_millis(200),
        use_own_book: false,
        debug_mode: true,
        chess960_mode: false,
    };
}
