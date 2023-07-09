use super::*;

pub mod engine_utils {
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
        string
    }

    pub fn simplify_fen(fen: &str) -> String {
        remove_double_spaces_and_trim(fen)
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
            for (threshold, unit) in [(86400.0, "days"), (3600.0, "hr"), (60.0, "min")] {
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

    impl Stringify for Move {
        fn stringify(&self) -> String {
            self.uci()
        }
    }

    impl Stringify for WeightedMove {
        fn stringify(&self) -> String {
            format!("({}, {})", self.move_.stringify(), self.weight)
        }
    }

    impl<T: Stringify> Stringify for Option<T> {
        fn stringify(&self) -> String {
            match self {
                Some(t) => format!("Some({})", t.stringify()),
                None => String::from("None"),
            }
        }
    }

    impl<T: Stringify, E: Display> Stringify for Result<T, E> {
        fn stringify(&self) -> String {
            match self {
                Ok(t) => format!("Ok({})", t.stringify()),
                Err(e) => format!("Err({})", e),
            }
        }
    }

    impl<T: Stringify> Stringify for [T] {
        fn stringify(&self) -> String {
            format!("[{}]", self.iter().map(|t| t.stringify()).join(", "))
        }
    }

    impl<T: Stringify> Stringify for Vec<T> {
        fn stringify(&self) -> String {
            self.as_slice().stringify()
        }
    }

    impl Stringify for CacheTableSize {
        fn stringify(&self) -> String {
            self.to_cache_table_memory_size::<TranspositionTableEntry>()
                .to_string()
        }
    }

    impl Stringify for Piece {
        fn stringify(&self) -> String {
            match self {
                Pawn => "Pawn",
                Knight => "Knight",
                Bishop => "Bishop",
                Rook => "Rook",
                Queen => "Queen",
                King => "King",
            }
            .to_string()
        }
    }

    impl Stringify for Color {
        fn stringify(&self) -> String {
            match self {
                White => "White",
                Black => "Black",
            }
            .to_string()
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
    use std::{array::TryFromSliceError, fmt::Debug};

    use super::*;
    use EngineError::*;

    #[derive(Clone, Fail, PartialEq, Eq)]
    pub enum EngineError {
        #[fail(display = "No input! Please try again!")]
        NoInput,

        #[fail(display = "")]
        UnknownCommand,

        #[fail(display = "Sorry, this command is not implemented yet :(")]
        NotImplemented,

        #[fail(display = "Engine is not running! Try again!")]
        EngineNotRunning,

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
            display = "Cannot set number of threads below the limit! Please choose a value above {MIN_NUM_THREADS}!"
        )]
        MinThreadsExceeded,

        #[fail(
            display = "Cannot set number of threads above the limit! Please choose a value below {MAX_NUM_THREADS}!"
        )]
        MaxThreadsExceeded,

        #[fail(
            display = "Cannot set hash table size below range {}! Please try again!",
            range
        )]
        MinHashTableSizeExceeded { range: String },

        #[fail(
            display = "Cannot set hash table size above range {}! Please try again!",
            range
        )]
        MaxHashTableSizeExceeded { range: String },

        #[fail(
            display = "Cannot set move overhead below range {}! Please try again!",
            range
        )]
        MinMoveOverheadExceeded { range: String },

        #[fail(
            display = "Cannot set move overhead above range {}! Please try again!",
            range
        )]
        MaxMoveOverheadExceeded { range: String },

        #[fail(display = "{}", err_msg)]
        CustomError { err_msg: String },
    }

    impl EngineError {
        pub fn stringify_with_optional_raw_input(
            &self,
            optional_raw_input: Option<&str>,
        ) -> String {
            match self {
                Self::UnknownCommand => match optional_raw_input {
                    Some(raw_input) => {
                        format!(
                            "Unknown command: {:?}\nType help for more information!",
                            raw_input
                        )
                    }
                    None => String::from("Unknown command!\nPlease try again!"),
                },
                other_err => format!("{}", other_err),
            }
        }
    }

    impl Stringify for EngineError {
        fn stringify(&self) -> String {
            self.stringify_with_optional_raw_input(None)
        }
    }

    impl Debug for EngineError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.stringify())
        }
    }

    impl From<EngineError> for String {
        fn from(error: EngineError) -> Self {
            error.stringify()
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

    impl From<std::io::Error> for EngineError {
        fn from(error: std::io::Error) -> Self {
            CustomError {
                err_msg: format!("{}! Try again!", error),
            }
        }
    }

    impl From<TryFromSliceError> for EngineError {
        fn from(error: TryFromSliceError) -> Self {
            CustomError {
                err_msg: format!("{}! Try again!", error),
            }
        }
    }

    impl From<String> for EngineError {
        fn from(err_msg: String) -> Self {
            CustomError { err_msg }
        }
    }

    impl From<&str> for EngineError {
        fn from(err_msg: &str) -> Self {
            err_msg.to_string().into()
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    }
}

pub mod info_utils {
    use super::*;

    #[inline(always)]
    pub fn format_info<T: ToString>(desc: &str, info: T) -> String {
        let mut desc = desc.trim().trim_end_matches(':').to_string();
        desc = colorize(desc, INFO_MESSAGE_STYLE);
        let info = info.to_string();
        if is_in_uci_mode() {
            format!("{desc} {info}")
        } else {
            format!("{desc}: {info}")
        }
    }

    #[inline(always)]
    pub fn println_info<T: ToString>(desc: &str, info: T) {
        println!("{}", format_info(desc, info));
    }

    #[inline(always)]
    pub fn get_engine_version() -> String {
        format!("{ENGINE_NAME} v{ENGINE_VERSION}")
    }

    pub fn print_engine_version(color: bool) {
        let version = get_engine_version();
        if color {
            println!("{}", colorize(version, SUCCESS_MESSAGE_STYLE));
            return;
        }
        println!("{version}");
    }

    pub fn print_engine_info() {
        print_engine_version(true);
        println!();
        TRANSPOSITION_TABLE.print_info();
        EVALUATOR.print_info();
    }
}

pub mod pv_utils {
    use super::*;

    pub fn extract_pv_from_t_table(board: &mut Board) -> Vec<Option<Move>> {
        let mut pv = Vec::new();
        let best_move = TRANSPOSITION_TABLE.read_best_move(board.hash());
        if let Some(best_move) = best_move {
            pv.push(Some(best_move));
            board.push(best_move);
            pv.append(&mut extract_pv_from_t_table(board));
            board.pop();
        }
        pv
    }

    pub fn get_pv_as_uci(pv: &[Option<Move>]) -> String {
        let mut pv_string = String::new();
        for move_ in pv {
            pv_string += &(move_.uci() + " ");
        }
        return pv_string.trim().to_string();
    }

    pub fn get_pv_as_algebraic(board: &Board, pv: &[Option<Move>], long: bool) -> String {
        let mut board = board.clone();
        let mut pv_string = String::new();
        for &move_ in pv {
            let is_legal_move = if let Some(move_) = move_ {
                board.is_legal(move_)
            } else {
                false
            };
            pv_string += &(if is_legal_move {
                board.algebraic_and_push(move_, long).unwrap()
            } else {
                colorize(move_.uci(), ERROR_MESSAGE_STYLE)
            } + " ");
        }
        return pv_string.trim().to_string();
    }

    pub fn get_pv_as_san(board: &Board, pv: &[Option<Move>]) -> String {
        get_pv_as_algebraic(board, pv, false)
    }

    pub fn get_pv_as_lan(board: &Board, pv: &[Option<Move>]) -> String {
        get_pv_as_algebraic(board, pv, true)
    }

    pub fn get_pv_string(board: &Board, pv: &[Option<Move>]) -> String {
        if is_in_uci_mode() {
            get_pv_as_uci(pv)
        } else {
            get_pv_as_algebraic(board, pv, use_long_algebraic_notation())
        }
    }
}

pub mod io_utils {
    use super::*;
    use std::io::{self, Read, Write};

    pub fn print_line<T: Display>(line: T) {
        let to_print = format!("{line}");
        if to_print.is_empty() {
            return;
        }
        print!("{to_print}");
        io::stdout().flush().unwrap();
    }

    pub struct IoReader {
        user_input: Mutex<String>,
        received_input: AtomicBool,
    }

    impl IoReader {
        pub fn new() -> Self {
            Self {
                user_input: Mutex::new(String::new()),
                received_input: AtomicBool::new(false),
            }
        }

        pub fn start_reader(&self) {
            loop {
                if self.received_input.load(MEMORY_ORDERING) {
                    continue;
                }
                std::io::stdin()
                    .read_line(&mut self.user_input.lock().unwrap())
                    .expect("Failed to read line!");
                self.received_input.store(true, MEMORY_ORDERING);
            }
        }

        pub fn read_line_once(&self) -> Option<String> {
            if !self.received_input.load(MEMORY_ORDERING) {
                thread::sleep(Duration::from_millis(1));
                return None;
            }
            let mut user_input = self.user_input.lock().unwrap();
            let input = user_input.to_owned();
            user_input.clear();
            self.received_input.store(false, MEMORY_ORDERING);
            Some(input)
        }

        pub fn read_line(&self) -> String {
            loop {
                if let Some(input) = self.read_line_once() {
                    return input;
                }
            }
        }
    }

    impl Default for IoReader {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub mod global_utils {
    use super::*;

    static TERMINATE_ENGINE: AtomicBool = AtomicBool::new(false);
    static COLORED_OUTPUT: AtomicBool = AtomicBool::new(true);
    static UCI_MODE: AtomicBool = AtomicBool::new(false);
    static T_TABLE_SIZE: Mutex<CacheTableSize> = Mutex::new(DEFAULT_T_TABLE_SIZE);
    static LONG_ALGEBRAIC_NOTATION: AtomicBool = AtomicBool::new(false);
    static NUM_THREADS: AtomicUsize = AtomicUsize::new(DEFAULT_NUM_THREADS);
    static MOVE_OVERHEAD: Mutex<Duration> = Mutex::new(DEFAULT_MOVE_OVERHEAD);
    static USE_OWN_BOOK: AtomicBool = AtomicBool::new(DEFAULT_USE_OWN_BOOK);

    fn print_info<T: Display>(message: &str, info: T) {
        if !is_in_uci_mode() {
            println!(
                "{} {}",
                colorize(message, SUCCESS_MESSAGE_STYLE),
                colorize(info, INFO_MESSAGE_STYLE),
            );
        }
    }

    #[inline(always)]
    pub fn terminate_engine() -> bool {
        TERMINATE_ENGINE.load(MEMORY_ORDERING)
    }

    pub fn set_engine_termination(b: bool) {
        TERMINATE_ENGINE.store(b, MEMORY_ORDERING);
    }

    #[inline(always)]
    pub fn is_colored_output() -> bool {
        COLORED_OUTPUT.load(MEMORY_ORDERING)
    }

    pub fn set_colored_output(b: bool, print: bool) {
        COLORED_OUTPUT.store(b, MEMORY_ORDERING);
        if print {
            print_info("Colored output is set to", b);
        }
    }

    #[inline(always)]
    pub fn is_in_uci_mode() -> bool {
        UCI_MODE.load(MEMORY_ORDERING)
    }

    pub fn set_uci_mode(b: bool, print: bool) {
        UCI_MODE.store(b, MEMORY_ORDERING);
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
        T_TABLE_SIZE.lock().unwrap().to_owned()
    }

    pub fn set_t_table_size(size: CacheTableSize) {
        *T_TABLE_SIZE.lock().unwrap() = size;
        TRANSPOSITION_TABLE.reset_size();
        if !is_in_uci_mode() {
            TRANSPOSITION_TABLE.print_info();
        }
        print_info(
            "Transposition table is set to size to",
            size.to_cache_table_memory_size::<TranspositionTableEntry>(),
        );
    }

    #[inline(always)]
    pub fn use_long_algebraic_notation() -> bool {
        LONG_ALGEBRAIC_NOTATION.load(MEMORY_ORDERING)
    }

    pub fn set_long_algebraic_notation(b: bool) {
        LONG_ALGEBRAIC_NOTATION.store(b, MEMORY_ORDERING);
        print_info("Long algebraic notation is set to", b);
    }

    #[inline(always)]
    pub fn get_num_threads() -> usize {
        NUM_THREADS.load(MEMORY_ORDERING)
    }

    pub fn set_num_threads(num_threads: usize, print: bool) {
        NUM_THREADS.store(num_threads, MEMORY_ORDERING);
        if print {
            print_info("Number of threads is set to", num_threads);
        }
    }

    #[inline(always)]
    pub fn get_move_overhead() -> Duration {
        MOVE_OVERHEAD.lock().unwrap().to_owned()
    }

    pub fn set_move_overhead(duration: Duration) {
        *MOVE_OVERHEAD.lock().unwrap() = duration;
        print_info("Move Overhead is set to", duration.stringify());
    }

    #[inline(always)]
    pub fn use_own_book() -> bool {
        USE_OWN_BOOK.load(MEMORY_ORDERING)
    }

    pub fn set_using_own_book(b: bool) {
        USE_OWN_BOOK.store(b, MEMORY_ORDERING);
        print_info("Own Book Usage is set to", b);
    }

    pub fn clear_all_hash_tables() {
        TRANSPOSITION_TABLE.clear();
        EVALUATOR.clear();
        if !is_in_uci_mode() {
            println!("{}", colorize("All hash tables are cleared!", SUCCESS_MESSAGE_STYLE));
        }
    }
}
