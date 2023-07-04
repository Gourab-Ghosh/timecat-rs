use super::*;

pub mod common_utils {
    use super::*;

    #[inline(always)]
    pub fn is_checkmate(score: Score) -> bool {
        let abs_score = score.abs();
        abs_score > CHECKMATE_THRESHOLD && abs_score <= CHECKMATE_SCORE
    }

    #[inline(always)]
    pub const fn evaluate_piece(piece: Piece) -> i16 {
        // never set knight and bishop values as same for knight bishop endgame
        match piece {
            Pawn => PAWN_VALUE,
            Knight => (32 * PAWN_VALUE) / 10,
            Bishop => (33 * PAWN_VALUE) / 10,
            Rook => 5 * PAWN_VALUE,
            Queen => 9 * PAWN_VALUE,
            King => 20 * PAWN_VALUE,
        }
    }

    #[inline(always)]
    pub fn format_info<T: ToString>(desc: &str, info: T) -> String {
        format!("{}: {}", colorize(desc, INFO_STYLE), info.to_string())
    }

    #[inline(always)]
    pub fn println_info<T: ToString>(desc: &str, info: T) {
        println!("{}", format_info(desc, info));
    }

    #[inline(always)]
    pub fn get_upper_board_mask(rank: Rank, color: Color) -> BitBoard {
        get_item_unchecked!(UPPER_BOARD_MASK, color.to_index(), rank.to_index())
    }

    #[inline(always)]
    pub fn get_lower_board_mask(rank: Rank, color: Color) -> BitBoard {
        get_upper_board_mask(rank, !color)
    }
}

pub mod string_utils {
    use super::*;
    use colored::{ColoredString, Colorize};

    pub fn remove_double_spaces_and_trim(s: &str) -> String {
        let mut string = String::new();
        for chr in s.trim().chars() {
            if !(chr == ' ' && string.ends_with(' ')) {
                string.push(chr);
            }
        }
        return string;
    }

    pub fn simplify_fen(fen: &str) -> String {
        remove_double_spaces_and_trim(fen).to_string()
    }

    fn colorize_string(s: ColoredString, color: &str) -> ColoredString {
        match color {
            "red" => s.red(),
            "blue" => s.blue(),
            "green" => s.green(),
            "white" => s.white(),
            "purple" => s.purple(),
            "bright_cyan" => s.bright_cyan(),
            "bright_red" => s.bright_red(),
            "on_bright_red" => s.on_bright_red(),
            "on_bright_black" => s.on_bright_black(),
            "bright_yellow" => s.bright_yellow(),
            "bold" => s.bold(),
            unknown_color => panic!("Cannot colorize string to {}", unknown_color),
        }
    }

    pub fn colorize<T: ToString>(obj: T, styles: &str) -> String {
        let s = obj.to_string();
        if !is_colored_output() {
            return s;
        }
        let styles = remove_double_spaces_and_trim(styles);
        let styles = styles.trim();
        if styles.is_empty() {
            return s;
        }
        let mut colored_string = ColoredString::from(s.as_str());
        for style in remove_double_spaces_and_trim(styles).split(' ') {
            colored_string = colorize_string(colored_string, style);
        }
        colored_string.to_string()
    }

    pub trait Stringify {
        fn stringify(&self) -> String;
    }

    pub trait StringifyScore {
        fn stringify_score_normal(self) -> String;
        fn stringify_score_uci(self) -> String;
    }

    impl StringifyScore for Score {
        fn stringify_score_normal(self) -> String {
            if self == INFINITY {
                return "INFINITY".to_string();
            }
            if self == -INFINITY {
                return "-INFINITY".to_string();
            }
            if is_checkmate(self) {
                let mut mate_string = String::from(if self.is_positive() { "M" } else { "-M" });
                let mate_distance = (CHECKMATE_SCORE - self.abs() + 1) / 2;
                mate_string += &mate_distance.to_string();
                return mate_string;
            }
            let to_return = self as f64 / PAWN_VALUE as f64;
            if to_return % 1.0 == 0.0 {
                format!("{}", to_return as i32)
            } else {
                format!("{:.2}", to_return)
            }
        }

        fn stringify_score_uci(self) -> String {
            if self == INFINITY {
                return "inf".to_string();
            }
            if self == -INFINITY {
                return "-inf".to_string();
            }
            if is_checkmate(self) {
                let mut mate_string = String::from("mate ");
                let mate_distance = (CHECKMATE_SCORE - self.abs() + 1) / 2;
                mate_string += &mate_distance.to_string();
                return mate_string;
            }
            format!("cp {}", (self as i32 * 100) / PAWN_VALUE as i32)
        }
    }

    impl Stringify for Score {
        fn stringify(&self) -> String {
            if is_in_uci_mode() {
                self.stringify_score_uci()
            } else {
                self.stringify_score_normal()
            }
        }
    }

    pub trait StringifyMove {
        fn uci(&self) -> String;
        fn algebraic(&self, board: &Board, long: bool) -> Result<String, BoardError>;
        fn san(&self, board: &Board) -> Result<String, BoardError> {
            self.algebraic(board, false)
        }
        fn lan(&self, board: &Board) -> Result<String, BoardError> {
            self.algebraic(board, true)
        }
        fn stringify_move(&self, board: &Board) -> Result<String, BoardError>;
    }

    impl StringifyMove for Option<Move> {
        fn uci(&self) -> String {
            match self {
                Some(m) => m.to_string(),
                None => String::from("0000"),
            }
        }

        fn algebraic(&self, board: &Board, long: bool) -> Result<String, BoardError> {
            board.clone().algebraic_and_push(*self, long)
        }

        fn stringify_move(&self, board: &Board) -> Result<String, BoardError> {
            if is_in_uci_mode() {
                Ok(self.uci())
            } else {
                self.algebraic(board, use_long_algebraic_notation())
            }
        }
    }

    impl StringifyMove for Move {
        fn uci(&self) -> String {
            Some(*self).uci()
        }

        fn algebraic(&self, board: &Board, long: bool) -> Result<String, BoardError> {
            Some(*self).algebraic(board, long)
        }

        fn stringify_move(&self, board: &Board) -> Result<String, BoardError> {
            Some(*self).stringify_move(board)
        }
    }

    impl Stringify for u64 {
        fn stringify(&self) -> String {
            format!("{:x}", self).to_uppercase()
        }
    }

    impl Stringify for BitBoard {
        fn stringify(&self) -> String {
            let mut checkers_string = String::new();
            for square in *self {
                checkers_string += &(square.to_string() + " ");
            }
            checkers_string.trim().to_uppercase()
        }
    }

    impl Stringify for Duration {
        fn stringify(&self) -> String {
            if is_in_uci_mode() {
                return self.as_millis().to_string();
            }
            if self < &Duration::from_secs(1) {
                return self.as_millis().to_string() + " ms";
            }
            let precision = 3;
            let total_secs = self.as_secs_f64();
            for (threshold, unit) in [
                (86400.0, "days"),
                (3600.0, "hr"),
                (60.0, "min"),
            ] {
                if total_secs >= threshold {
                    let time_unit = total_secs as u128 / threshold as u128;
                    let secs = total_secs % threshold;
                    let mut string = format!("{} {}", time_unit, unit);
                    if time_unit > 1 {
                        string += "s";
                    }
                    if secs >= 10.0_f64.powi(-(precision as i32)) {
                        string += " ";
                        string += &Duration::from_secs_f64(secs).stringify();
                    }
                    return string;
                }
            }
            let mut string = format!("{:.1$} sec", total_secs, precision);
            if total_secs > 1.0 {
                string += "s";
            }
            string
        }
    }
}

pub mod hash_utils {
    pub trait CustomHash {
        fn hash(&self) -> u64;
    }

    impl CustomHash for chess::Board {
        #[inline(always)]
        fn hash(&self) -> u64 {
            self.get_hash().max(1)
        }
    }
}

pub mod square_utils {
    use super::*;

    #[inline(always)]
    pub fn square_mirror(square: Square) -> Square {
        get_item_unchecked!(SQUARES_180, square.to_index())
    }

    #[inline(always)]
    pub fn get_square_bb(sq: Square) -> BitBoard {
        get_item_unchecked!(BB_SQUARES, sq.to_index())
    }

    pub fn square_distance(square1: Square, square2: Square) -> u8 {
        let (file1, rank1) = (square1.get_file(), square1.get_rank());
        let (file2, rank2) = (square2.get_file(), square2.get_rank());
        let file_distance = (file1 as i8).abs_diff(file2 as i8);
        let rank_distance = (rank1 as i8).abs_diff(rank2 as i8);
        file_distance.max(rank_distance)
    }
}

pub mod engine_error {
    use std::fmt::Debug;

    use super::*;
    use EngineError::*;

    #[derive(Clone, Fail)]
    pub enum EngineError {
        #[fail(display = "")]
        UnknownCommand,

        #[fail(display = "Sorry, this command is not implemented yet :(")]
        NotImplemented,

        #[fail(display = "Bad FEN string: {}! Try Again!", fen)]
        BadFen { fen: String },

        #[fail(display = "Invalid depth {}! Try again!", depth)]
        InvalidDepth { depth: String },

        #[fail(
            display = "Illegal move {} in position {}! Try again!",
            move_text, board_fen
        )]
        IllegalMove {
            move_text: String,
            board_fen: String,
        },

        #[fail(display = "Colored output already set to {}! Try again!", b)]
        ColoredOutputUnchanged { b: String },

        #[fail(display = "UCI mode already set to {}! Try again!", b)]
        UCIModeUnchanged { b: String },

        #[fail(display = "Move Stack is empty, pop not possible! Try again!")]
        EmptyStack,

        #[fail(display = "Best move not found in position {}! Try again!", fen)]
        BestMoveNotFound { fen: String },

        #[fail(
            display = "Cannot apply null move in position {}, as king is in check! Try again!",
            fen
        )]
        NullMoveInCheck { fen: String },

        #[fail(display = "You didn't mention wtime! Try again!")]
        WTimeNotMentioned,

        #[fail(display = "You didn't mention btime! Try again!")]
        BTimeNotMentioned,

        #[fail(display = "Game is already over! Please start a game from another position!")]
        GameAlreadyOver,

        #[fail(display = "Cannot set number of threads to 0! Please try again!")]
        ZeroThreads,

        #[fail(
            display = "Cannot exceed number of threads limit! Please choose a value up to {MAX_NUM_THREADS}!"
        )]
        MaxThreadsExceeded,

        #[fail(display = "{}", err_msg)]
        CustomError { err_msg: String },
    }

    impl EngineError {
        pub fn stringify(&self, optional_raw_input: Option<&str>) -> String {
            match self {
                Self::UnknownCommand => match optional_raw_input {
                    Some(raw_input) => {
                        format!("Unknown command: {}\nPlease try again!", raw_input.trim())
                    }
                    None => String::from("Unknown command!\nPlease try again!"),
                },
                other_err => format!("{}", other_err),
            }
        }
    }

    impl Debug for EngineError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.stringify(None))
        }
    }

    impl From<EngineError> for String {
        fn from(error: EngineError) -> Self {
            error.stringify(None)
        }
    }

    impl From<&Self> for EngineError {
        fn from(error: &Self) -> Self {
            error.clone()
        }
    }

    impl From<ParseBoolError> for EngineError {
        fn from(error: ParseBoolError) -> Self {
            CustomError {
                err_msg: format!("Failed to parse bool, {}! Try again!", error),
            }
        }
    }

    impl From<ParseIntError> for EngineError {
        fn from(error: ParseIntError) -> Self {
            CustomError {
                err_msg: format!("Failed to parse integer, {}! Try again!", error),
            }
        }
    }

    impl From<chess::Error> for EngineError {
        fn from(error: chess::Error) -> Self {
            CustomError {
                err_msg: format!("{}! Try again!", error),
            }
        }
    }
}

pub mod bitboard_utils {
    use super::*;

    pub fn get_queen_moves(sq: Square, blockers: BitBoard) -> BitBoard {
        get_rook_moves(sq, blockers) | get_bishop_moves(sq, blockers)
    }
}

pub mod cache_table_utils {
    use super::CacheTableEntry;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CacheTableSize {
        Max(usize),
        Min(usize),
        Round(usize),
    }

    impl CacheTableSize {
        pub fn unwrap(&self) -> usize {
            match self {
                Self::Max(size) => *size,
                Self::Min(size) => *size,
                Self::Round(size) => *size,
            }
        }

        pub fn is_min(&self) -> bool {
            matches!(self, Self::Min(_))
        }

        pub fn is_max(&self) -> bool {
            matches!(self, Self::Max(_))
        }

        pub fn is_round(&self) -> bool {
            matches!(self, Self::Round(_))
        }

        pub fn get_entry_size<T: Copy + Clone + PartialEq + PartialOrd>() -> usize {
            std::mem::size_of::<CacheTableEntry<T>>()
        }

        pub fn to_cache_table_and_entry_size<T: Copy + Clone + PartialEq + PartialOrd>(
            self,
        ) -> (usize, usize) {
            let mut size = self.unwrap();
            let entry_size = Self::get_entry_size::<T>();
            size *= 2_usize.pow(20);
            size /= entry_size;
            let pow_f64 = (size as f64).log2();
            let pow = match self {
                Self::Max(_) => pow_f64.floor(),
                Self::Min(_) => pow_f64.ceil(),
                Self::Round(_) => pow_f64.round(),
            } as u32;
            size = 2_usize.pow(pow);
            (size, entry_size)
        }

        pub fn to_cache_table_size<T: Copy + Clone + PartialEq + PartialOrd>(self) -> usize {
            self.to_cache_table_and_entry_size::<T>().0
        }

        pub fn to_cache_table_memory_size<T: Copy + Clone + PartialEq + PartialOrd>(self) -> usize {
            let (size, entry_size) = self.to_cache_table_and_entry_size::<T>();
            size * entry_size / 2_usize.pow(20)
        }
    }
}

pub mod classes {
    use super::*;
    // use std::collections::hash_map::DefaultHasher;
    // use std::hash::{Hash, Hasher};

    #[derive(Default, Debug, Clone)]
    pub struct RepetitionTable {
        count_map: HashMap<u64, usize>,
    }

    impl RepetitionTable {
        pub fn new() -> Self {
            Self {
                count_map: HashMap::default(),
            }
        }

        pub fn get_repetition(&self, key: u64) -> u8 {
            *self.count_map.get(&key).unwrap_or(&0) as u8
        }

        pub fn insert_and_get_repetition(&mut self, key: u64) -> u8 {
            let count_entry = self.count_map.entry(key).or_insert(0);
            *count_entry += 1;
            *count_entry as u8
        }

        pub fn remove(&mut self, key: u64) {
            let count_entry = self.count_map.get_mut(&key).unwrap_or_else(|| {
                panic!(
                    "Tried to remove the key {} that doesn't exist!",
                    key.stringify()
                )
            });
            *count_entry -= 1;
            if *count_entry == 0 {
                self.count_map.remove(&key);
            }
        }

        pub fn clear(&mut self) {
            self.count_map.clear();
        }

        // fn hash<T: Hash>(t: &T) -> u64 {
        //     let mut s = DefaultHasher::new();
        //     t.hash(&mut s);
        //     s.finish()
        // }
    }
}

pub mod score_utils {
    #[derive(Clone, Copy, Debug)]
    enum Score {
        Cp(i16),
        Mate(i16),
        Infinity,
    }
}

pub mod global_utils {
    use super::*;

    static mut COLORED_OUTPUT: bool = true;
    static mut UCI_MODE: bool = false;
    static mut T_TABLE_SIZE: CacheTableSize = INITIAL_T_TABLE_SIZE;
    static mut LONG_ALGEBRAIC_NOTATION: bool = false;
    static mut NUM_THREADS: usize = INITIAL_NUM_THREADS;

    fn print_info<T: Display>(message: &str, info: T) {
        if !is_in_uci_mode() {
            println!(
                "{} {}",
                colorize(message, SUCCESS_MESSAGE_STYLE),
                colorize(info, INFO_STYLE),
            );
        }
    }

    #[inline(always)]
    pub fn is_colored_output() -> bool {
        unsafe { COLORED_OUTPUT }
    }

    pub fn set_colored_output(b: bool, print: bool) {
        unsafe {
            COLORED_OUTPUT = b;
        }
        if print {
            print_info("Colored output is set to", b);
        }
    }

    #[inline(always)]
    pub fn is_in_uci_mode() -> bool {
        unsafe { UCI_MODE }
    }

    pub fn set_uci_mode(b: bool, print: bool) {
        unsafe {
            UCI_MODE = b;
        }
        if print {
            print_info("UCI mode is set to", b);
        }
    }

    pub fn enable_uci_and_disable_color() {
        set_colored_output(false, false);
        set_uci_mode(true, false);
    }

    #[inline(always)]
    pub fn get_t_table_size() -> CacheTableSize {
        unsafe { T_TABLE_SIZE }
    }

    pub fn set_t_table_size(size: CacheTableSize) {
        unsafe {
            T_TABLE_SIZE = size;
        }
        print_info(
            "Set t-table size to",
            size.to_cache_table_memory_size::<TranspositionTableEntry>(),
        );
    }

    #[inline(always)]
    pub fn use_long_algebraic_notation() -> bool {
        unsafe { LONG_ALGEBRAIC_NOTATION }
    }

    pub fn set_long_algebraic_notation(b: bool) {
        unsafe {
            LONG_ALGEBRAIC_NOTATION = b;
        }
        print_info("Set long algebraic notation to", b);
    }

    #[inline(always)]
    pub fn get_num_threads() -> usize {
        unsafe { NUM_THREADS }
    }

    pub fn set_num_threads(num_threads: usize, print: bool) {
        unsafe {
            NUM_THREADS = num_threads;
        }
        if print {
            print_info("Number of threads set to", num_threads);
        }
    }
}
