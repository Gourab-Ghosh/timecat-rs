use super::*;

pub mod engine_utils {
    use super::*;

    #[inline]
    pub fn is_checkmate(score: Score) -> bool {
        let abs_score = score.abs();
        abs_score > CHECKMATE_THRESHOLD && abs_score <= CHECKMATE_SCORE
    }

    #[inline]
    pub fn get_upper_board_mask(rank: Rank, color: Color) -> BitBoard {
        get_item_unchecked!(UPPER_BOARD_MASK, color.to_index(), rank.to_index())
    }

    #[inline]
    pub fn get_lower_board_mask(rank: Rank, color: Color) -> BitBoard {
        get_upper_board_mask(rank, !color)
    }
}

pub mod piece_utils {
    use super::*;

    #[inline]
    pub const fn evaluate_piece(piece: PieceType) -> i16 {
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

    pub trait PieceTypeTrait {
        type PieceType;

        fn get_type(self) -> Self::PieceType;
    }

    impl PieceTypeTrait for PieceType {
        type PieceType = u8;

        fn get_type(self) -> Self::PieceType {
            Some(self).get_type()
        }
    }

    impl PieceTypeTrait for Option<PieceType> {
        type PieceType = u8;

        fn get_type(self) -> Self::PieceType {
            match self {
                Some(piece) => piece.to_index() as Self::PieceType + 1,
                None => 0,
            }
        }
    }
}

pub mod move_utils {
    use super::*;

    pub trait Compress {
        type CompressedItem;

        fn compress(self) -> Self::CompressedItem;
    }

    pub trait Decompress<T> {
        fn decompress(self) -> T;
    }

    impl Compress for Option<PieceType> {
        type CompressedItem = u8;

        fn compress(self) -> Self::CompressedItem {
            self.get_type() as Self::CompressedItem
        }
    }

    impl Compress for PieceType {
        type CompressedItem = u8;

        fn compress(self) -> Self::CompressedItem {
            Some(self).compress()
        }
    }

    impl Compress for Square {
        type CompressedItem = u16;

        fn compress(self) -> Self::CompressedItem {
            self.to_index() as Self::CompressedItem
        }
    }

    impl Compress for Move {
        type CompressedItem = u16;

        fn compress(self) -> Self::CompressedItem {
            let mut compressed_move = 0;
            compressed_move |= self.get_source().compress() << 6;
            compressed_move |= self.get_dest().compress();
            compressed_move |= (self.get_promotion().compress() as Self::CompressedItem) << 12;
            compressed_move
        }
    }

    impl Compress for Option<Move> {
        type CompressedItem = u16;

        fn compress(self) -> Self::CompressedItem {
            match self {
                Some(m) => m.compress(),
                None => Self::CompressedItem::MAX,
            }
        }
    }

    impl Decompress<Option<PieceType>> for u8 {
        fn decompress(self) -> Option<PieceType> {
            if self == 0 {
                return None;
            }
            Some(get_item_unchecked!(ALL_PIECE_TYPES, (self - 1) as usize))
        }
    }

    impl Decompress<Option<PieceType>> for u16 {
        fn decompress(self) -> Option<PieceType> {
            (self as u8).decompress()
        }
    }

    impl Decompress<Square> for u16 {
        fn decompress(self) -> Square {
            get_item_unchecked!(ALL_SQUARES, self as usize)
        }
    }

    impl Decompress<Option<Move>> for u16 {
        fn decompress(self) -> Option<Move> {
            if self == u16::MAX {
                return None;
            }
            let source = ((self >> 6) & 63).decompress();
            let dest = (self & 63).decompress();
            let promotion = (self >> 12).decompress();
            Some(Move::new(source, dest, promotion))
        }
    }

    // impl<T> Decompress<T> for CompressedObject where CompressedObject: Decompress<Option<T>> {
    //     fn decompress(self) -> T {
    //         self.decompress().unwrap_or_else(|| panic!("Failed to decompress"))
    //     }
    // }
}

pub mod string_utils {
    use super::*;

    pub fn remove_double_spaces_and_trim(s: &str) -> String {
        s.trim()
            .chars()
            .dedup_by(|&c1, &c2| c1 == ' ' && c2 == ' ')
            .join("")
    }

    pub fn simplify_fen(fen: &str) -> String {
        remove_double_spaces_and_trim(fen)
    }

    pub fn flip_board_fen(fen: &str) -> Result<String, EngineError> {
        // TODO: ep square not flipped.
        let fen = remove_double_spaces_and_trim(fen);
        let (position_fen, rest_fen) = fen.split_once(' ').ok_or(EngineError::BadFen {
            fen: fen.to_string(),
        })?;
        Ok(format!(
            "{} {rest_fen}",
            position_fen
                .chars()
                .map(|c| match c {
                    c if c.is_uppercase() => c.to_ascii_lowercase(),
                    c if c.is_lowercase() => c.to_ascii_uppercase(),
                    _ => c,
                })
                .collect::<String>(),
        ))
    }

    pub trait CustomColorize {
        fn colorize(&self, styles_functions: &[ColoredStringFunction]) -> String;
    }

    impl<T: ToString> CustomColorize for T {
        fn colorize(&self, styles_functions: &[ColoredStringFunction]) -> String {
            let self_string = self.to_string();
            if styles_functions.is_empty() || !is_colored_output() {
                return self_string;
            }
            let mut colorized_string = self_string.as_str().into();
            for &func in styles_functions {
                colorized_string = func(colorized_string);
            }
            colorized_string.to_string()
        }
    }

    pub trait Stringify {
        fn stringify(&self) -> String;
    }

    pub trait StringifyScore {
        fn stringify_score(self) -> String;
        fn stringify_score_uci(self) -> String;
    }

    impl StringifyScore for Score {
        fn stringify_score(self) -> String {
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
            if is_in_console_mode() {
                self.stringify_score()
            } else {
                self.stringify_score_uci()
            }
        }
    }

    pub trait StringifyMove {
        fn uci(self) -> String;
        fn algebraic(self, board: &Board, long: bool) -> Result<String, BoardError>;
        fn san(self, board: &Board) -> Result<String, BoardError>
        where
            Self: Sized,
        {
            self.algebraic(board, false)
        }
        fn lan(self, board: &Board) -> Result<String, BoardError>
        where
            Self: Sized,
        {
            self.algebraic(board, true)
        }
        fn stringify_move(self, board: &Board) -> Result<String, BoardError>;
    }

    impl StringifyMove for Option<Move> {
        fn uci(self) -> String {
            match self {
                Some(m) => m.to_string(),
                None => String::from("0000"),
            }
        }

        fn algebraic(self, board: &Board, long: bool) -> Result<String, BoardError> {
            board.clone().algebraic_and_push(self, long)
        }

        fn stringify_move(self, board: &Board) -> Result<String, BoardError> {
            if is_in_console_mode() {
                self.algebraic(board, use_long_algebraic_notation())
            } else {
                Ok(self.uci())
            }
        }
    }

    impl StringifyMove for Move {
        fn uci(self) -> String {
            Some(self).uci()
        }

        fn algebraic(self, board: &Board, long: bool) -> Result<String, BoardError> {
            Some(self).algebraic(board, long)
        }

        fn stringify_move(self, board: &Board) -> Result<String, BoardError> {
            Some(self).stringify_move(board)
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

    impl<T: Stringify, E: Error> Stringify for Result<T, E> {
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
            format!("{self}")
        }
    }

    impl Stringify for PieceType {
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

pub mod time_utils {
    use super::*;

    impl Stringify for Duration {
        fn stringify(&self) -> String {
            if !is_in_console_mode() {
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

    pub fn measure_time<T>(func: impl Fn() -> T) -> T {
        let clock = Instant::now();
        let res = func();
        if is_in_console_mode() {
            println!();
        }
        println_info("Run Time", clock.elapsed().stringify());
        res
    }
}

pub mod hash_utils {
    use super::*;

    pub trait CustomHash {
        fn hash(&self) -> u64;
    }

    impl CustomHash for SubBoard {
        #[inline]
        fn hash(&self) -> u64 {
            self.get_hash().max(1)
        }
    }
}

pub mod square_utils {
    use super::*;

    #[inline]
    pub fn square_mirror(square: Square) -> Square {
        get_item_unchecked!(SQUARES_180, square.to_index())
    }

    #[inline]
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
    use super::*;
    use EngineError::*;

    #[derive(Clone, Fail, PartialEq, Eq, Debug)]
    pub enum EngineError {
        #[fail(display = "No input! Please try again!")]
        NoInput,

        #[fail(display = "")]
        UnknownCommand,

        #[fail(display = "Sorry, this command is not implemented yet :(")]
        NotImplemented,

        #[fail(display = "Engine is not running! Please try again!")]
        EngineNotRunning,

        #[fail(display = "Bad FEN string: {}! Please try Again!", fen)]
        BadFen { fen: String },

        #[fail(display = "Invalid depth {}! Please try again!", depth)]
        InvalidDepth { depth: Depth },

        #[fail(
            display = "Illegal move {} in position {}! Please try again!",
            move_text, board_fen
        )]
        IllegalMove {
            move_text: String,
            board_fen: String,
        },

        #[fail(display = "Colored output already set to {}! Please try again!", b)]
        ColoredOutputUnchanged { b: bool },

        #[fail(display = "Already in Console Mode! Please try again!")]
        ConsoleModeUnchanged,

        #[fail(display = "Move Stack is empty, pop not possible! Please try again!")]
        EmptyStack,

        #[fail(display = "Best move not found in position {}! Please try again!", fen)]
        BestMoveNotFound { fen: String },

        #[fail(
            display = "Cannot apply null move in position {}, as king is in check! Please try again!",
            fen
        )]
        NullMoveInCheck { fen: String },

        #[fail(display = "You didn't mention wtime! Please try again!")]
        WTimeNotMentioned,

        #[fail(display = "You didn't mention btime! Please try again!")]
        BTimeNotMentioned,

        #[fail(display = "Game is already over! Please start a game from another position!")]
        GameAlreadyOver,

        #[fail(
            display = "Debug command {} is unknown! The possible commands are on or off! Please try again!",
            command
        )]
        UnknownDebugCommand { command: String },

        #[fail(
            display = "Cannot set value of {} to {}, the value must be from {} to {}! Please try again!",
            name, value, min, max
        )]
        InvalidSpinValue {
            name: String,
            value: Spin,
            min: Spin,
            max: Spin,
        },

        #[fail(display = "Got invalid SAN move string {}! Please try again!", s)]
        InvalidSanMoveString { s: String },

        #[fail(display = "Got invalid rank string {}! Please try again!", s)]
        InvalidRankString { s: String },

        #[fail(display = "Got invalid file string {}! Please try again!", s)]
        InvalidFileString { s: String },

        #[fail(display = "Got invalid square string {}! Please try again!", s)]
        InvalidSquareString { s: String },

        #[fail(display = "You didn't mention wtime! Please try again!")]
        InvalidUciMoveString { s: String },

        #[fail(display = "Invalid sub board generated:\n\n{:#?}", board)]
        InvalidSubBoard { board: SubBoard },

        #[fail(display = "{}", err_msg)]
        CustomError { err_msg: String },
    }

    impl EngineError {
        pub fn stringify_with_optional_raw_input(
            &self,
            optional_raw_input: Option<&str>,
        ) -> String {
            match self {
                Self::UnknownCommand => {
                    let command_type = if is_in_console_mode() {
                        "Console"
                    } else {
                        "UCI"
                    };
                    match optional_raw_input {
                        Some(raw_input) => format!(
                            "Unknown {command_type} Command: {:?}\nType help for more information!",
                            raw_input.trim_end_matches('\n')
                        ),
                        None => format!("Unknown {command_type} Command!\nPlease try again!"),
                    }
                }
                other_err => other_err.to_string(),
            }
        }
    }

    impl Stringify for EngineError {
        fn stringify(&self) -> String {
            self.stringify_with_optional_raw_input(None)
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
                err_msg: format!("Failed to parse bool, {error}! Please try again!"),
            }
        }
    }

    impl From<ParseIntError> for EngineError {
        fn from(error: ParseIntError) -> Self {
            CustomError {
                err_msg: format!("Failed to parse integer, {error}! Please try again!"),
            }
        }
    }

    macro_rules! impl_error_convert {
        ($class:ty) => {
            impl From<$class> for EngineError {
                fn from(error: $class) -> Self {
                    CustomError {
                        err_msg: format!("{error}! Please try again!"),
                    }
                }
            }
        };
    }

    impl_error_convert!(std::io::Error);
    impl_error_convert!(std::array::TryFromSliceError);

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
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum CacheTableSize {
        Max(usize),
        Min(usize),
        Round(usize),
        Exact(usize),
    }

    impl CacheTableSize {
        pub fn unwrap(&self) -> usize {
            match self {
                Self::Max(size) => *size,
                Self::Min(size) => *size,
                Self::Round(size) => *size,
                Self::Exact(size) => *size,
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
        pub fn is_exact(&self) -> bool {
            matches!(self, Self::Exact(_))
        }

        pub const fn get_entry_size<T: Copy + Clone + PartialEq>() -> usize {
            std::mem::size_of::<CacheTableEntry<T>>()
        }

        pub fn to_cache_table_and_entry_size<T: Copy + Clone + PartialEq>(self) -> (usize, usize) {
            let mut size = self.unwrap();
            let entry_size = Self::get_entry_size::<T>();
            size *= 2_usize.pow(20);
            size /= entry_size;
            if self.is_exact() {
                return (size, entry_size);
            }
            let pow_f64 = (size as f64).log2();
            let pow = match self {
                Self::Max(_) => pow_f64.floor(),
                Self::Min(_) => pow_f64.ceil(),
                Self::Round(_) => pow_f64.round(),
                Self::Exact(_) => unreachable!(),
            } as u32;
            size = 2_usize.pow(pow);
            (size, entry_size)
        }

        pub fn to_cache_table_size<T: Copy + Clone + PartialEq>(self) -> usize {
            self.to_cache_table_and_entry_size::<T>().0
        }

        pub fn to_cache_table_memory_size<T: Copy + Clone + PartialEq>(self) -> usize {
            let (size, entry_size) = self.to_cache_table_and_entry_size::<T>();
            size * entry_size / 2_usize.pow(20)
        }
    }

    impl fmt::Display for CacheTableSize {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} MB", self.unwrap())
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
            if *count_entry == 1 {
                self.count_map.remove(&key);
                return;
            }
            *count_entry -= 1;
        }

        pub fn clear(&mut self) {
            self.count_map.clear();
        }
    }
}

pub mod info_utils {
    use super::*;

    #[inline]
    pub fn format_info<T: fmt::Display>(desc: &str, info: T) -> String {
        let desc = desc
            .trim()
            .trim_end_matches(':')
            .colorize(INFO_MESSAGE_STYLE);
        if is_in_console_mode() {
            format!("{desc}: {info}")
        } else {
            format!("{desc} {info}")
        }
    }

    pub fn force_println_info<T: fmt::Display>(desc: &str, info: T) {
        let formatted_info = format_info(desc, info);
        let to_print = if is_in_console_mode() {
            formatted_info
        } else {
            format!(
                "{} {formatted_info}",
                "info string".colorize(INFO_MESSAGE_STYLE)
            )
        };
        println!("{to_print}");
    }

    pub fn println_info<T: fmt::Display>(desc: &str, info: T) {
        if is_in_debug_mode() {
            force_println_info(desc, info);
        }
    }

    #[inline]
    pub fn get_engine_version() -> String {
        format!("{ENGINE_NAME} v{ENGINE_VERSION}")
    }

    pub fn print_engine_version(color: bool) {
        let version = get_engine_version();
        if color {
            println!("{}", version.colorize(SUCCESS_MESSAGE_STYLE));
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

    pub fn print_cache_table_info(
        name: &str,
        table_len: impl fmt::Display,
        table_size: impl fmt::Display,
    ) {
        let mut to_print = format!(
            "{name} initialization complete with {table_len} entries taking {table_size} space."
        );
        if !is_in_console_mode() {
            to_print = "info string ".to_string() + to_print.trim();
        }
        println!("{}", to_print.colorize(INFO_MESSAGE_STYLE));
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
                move_.uci().colorize(ERROR_MESSAGE_STYLE)
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
        if is_in_console_mode() {
            get_pv_as_algebraic(board, pv, use_long_algebraic_notation())
        } else {
            get_pv_as_uci(pv)
        }
    }
}

pub mod io_utils {
    use super::*;
    use std::io::{self, Write};

    pub fn print_line<T: fmt::Display>(line: T) {
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
            drop(user_input);
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
    static CONSOLE_MODE: AtomicBool = AtomicBool::new(true);
    static T_TABLE_SIZE: Mutex<CacheTableSize> = Mutex::new(T_TABLE_SIZE_UCI.get_default());
    static LONG_ALGEBRAIC_NOTATION: AtomicBool = AtomicBool::new(false);
    static NUM_THREADS: AtomicUsize = AtomicUsize::new(NUM_THREADS_UCI.get_default());
    static MOVE_OVERHEAD: Mutex<Duration> = Mutex::new(MOVE_OVERHEAD_UCI.get_default());
    static USE_OWN_BOOK: AtomicBool = AtomicBool::new(DEFAULT_USE_OWN_BOOK);
    static DEBUG_MODE: AtomicBool = AtomicBool::new(DEFAULT_DEBUG_MODE);

    fn print_info<T: fmt::Display>(message: &str, info: impl Into<Option<T>>) {
        if !is_in_debug_mode() {
            return;
        }
        let mut to_print = if let Some(info_message) = info.into() {
            format!(
                "{} {}",
                message.colorize(SUCCESS_MESSAGE_STYLE),
                info_message.colorize(INFO_MESSAGE_STYLE),
            )
        } else {
            message.colorize(SUCCESS_MESSAGE_STYLE)
        };
        if !is_in_console_mode() {
            to_print = format!("{} {to_print}", "info string".colorize(INFO_MESSAGE_STYLE))
        }
        println!("{to_print}");
    }

    #[inline]
    pub fn terminate_engine() -> bool {
        TERMINATE_ENGINE.load(MEMORY_ORDERING)
    }

    pub fn set_engine_termination(b: bool) {
        TERMINATE_ENGINE.store(b, MEMORY_ORDERING);
    }

    #[inline]
    pub fn is_colored_output() -> bool {
        COLORED_OUTPUT.load(MEMORY_ORDERING)
    }

    pub fn set_colored_output(b: bool, print: bool) {
        COLORED_OUTPUT.store(b, MEMORY_ORDERING);
        if print {
            print_info("Colored output is set to", b);
        }
    }

    #[inline]
    pub fn is_in_console_mode() -> bool {
        CONSOLE_MODE.load(MEMORY_ORDERING)
    }

    pub fn set_console_mode(b: bool, print: bool) {
        CONSOLE_MODE.store(b, MEMORY_ORDERING);
        DEBUG_MODE.store(b, MEMORY_ORDERING);
        if print {
            print_info("UCI mode is set to", b);
        }
    }

    #[inline]
    pub fn get_t_table_size() -> CacheTableSize {
        T_TABLE_SIZE.lock().unwrap().to_owned()
    }

    pub fn set_t_table_size(size: CacheTableSize) {
        *T_TABLE_SIZE.lock().unwrap() = size;
        TRANSPOSITION_TABLE.reset_size();
        if is_in_debug_mode() {
            TRANSPOSITION_TABLE.print_info();
        }
        print_info(
            "Transposition table is set to size to",
            size.to_cache_table_memory_size::<TranspositionTableEntry>(),
        );
    }

    #[inline]
    pub fn use_long_algebraic_notation() -> bool {
        LONG_ALGEBRAIC_NOTATION.load(MEMORY_ORDERING)
    }

    pub fn set_long_algebraic_notation(b: bool) {
        LONG_ALGEBRAIC_NOTATION.store(b, MEMORY_ORDERING);
        print_info("Long algebraic notation is set to", b);
    }

    #[inline]
    pub fn get_num_threads() -> usize {
        NUM_THREADS.load(MEMORY_ORDERING)
    }

    pub fn set_num_threads(num_threads: usize, print: bool) {
        NUM_THREADS.store(num_threads, MEMORY_ORDERING);
        if print {
            print_info("Number of threads is set to", num_threads);
        }
    }

    #[inline]
    pub fn get_move_overhead() -> Duration {
        MOVE_OVERHEAD.lock().unwrap().to_owned()
    }

    pub fn set_move_overhead(duration: Duration) {
        *MOVE_OVERHEAD.lock().unwrap() = duration;
        print_info("Move Overhead is set to", duration.stringify());
    }

    #[inline]
    pub fn use_own_book() -> bool {
        USE_OWN_BOOK.load(MEMORY_ORDERING)
    }

    pub fn set_using_own_book(b: bool) {
        USE_OWN_BOOK.store(b, MEMORY_ORDERING);
        print_info("Own Book Usage is set to", b);
    }

    #[inline]
    pub fn is_in_debug_mode() -> bool {
        DEBUG_MODE.load(MEMORY_ORDERING)
    }

    pub fn set_debug_mode(b: bool) {
        DEBUG_MODE.store(b, MEMORY_ORDERING);
        print_info("Debug Mode is set to", b);
    }

    #[inline]
    pub fn is_in_console_and_debug_mode() -> bool {
        is_in_console_mode() && is_in_debug_mode()
    }

    pub fn clear_all_hash_tables() {
        TRANSPOSITION_TABLE.clear();
        EVALUATOR.clear();
        print_info::<&str>("All hash tables are cleared!", None);
    }
}
