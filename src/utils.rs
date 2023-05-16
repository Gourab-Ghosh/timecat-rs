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

    pub fn remove_double_spaces(s: &str) -> String {
        let mut s = s.to_owned();
        while s.contains("  ") {
            s = s.replace("  ", " ");
        }
        return s.trim().to_string();
    }

    pub fn simplify_fen(fen: &str) -> String {
        remove_double_spaces(fen).trim().to_string()
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
        let styles = remove_double_spaces(styles);
        let styles = styles.trim();
        if styles.is_empty() {
            return s;
        }
        let mut colored_string = ColoredString::from(s.as_str());
        for style in remove_double_spaces(styles).split(' ') {
            colored_string = colorize_string(colored_string, style);
        }
        colored_string.to_string()
    }

    pub fn score_to_string(score: Score) -> String {
        if score == INFINITY {
            return "INFINITY".to_string();
        }
        if score == -INFINITY {
            return "-INFINITY".to_string();
        }
        if is_checkmate(score) {
            let mut mate_string = String::from(if score.is_positive() { "M" } else { "-M" });
            let mate_distance = (CHECKMATE_SCORE - score.abs() + 1) / 2;
            mate_string += &mate_distance.to_string();
            return mate_string;
        }
        let to_return = score as f64 / PAWN_VALUE as f64;
        if to_return % 1.0 == 0.0 {
            format!("{}", to_return as i32)
        } else {
            format!("{:.2}", to_return)
        }
    }

    pub fn hash_to_string(hash: u64) -> String {
        format!("{:x}", hash).to_uppercase()
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

pub mod bitboard_utils {
    use super::*;

    pub fn get_queen_moves(sq: Square, blockers: BitBoard) -> BitBoard {
        get_rook_moves(sq, blockers) | get_bishop_moves(sq, blockers)
    }
}

pub mod cache_table_utils {
    struct CacheTableEntry<T> {
        hash: u64,
        entry: T,
    }

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
            match self {
                Self::Min(_) => true,
                _ => false,
            }
        }

        pub fn is_max(&self) -> bool {
            match self {
                Self::Max(_) => true,
                _ => false,
            }
        }

        pub fn is_round(&self) -> bool {
            match self {
                Self::Round(_) => true,
                _ => false,
            }
        }

        pub fn get_entry_size<T>() -> usize {
            std::mem::size_of::<CacheTableEntry<T>>()
        }

        pub fn to_cache_table_and_entry_size<T>(&self) -> (usize, usize) {
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

        pub fn to_cache_table_size<T>(&self) -> usize {
            self.to_cache_table_and_entry_size::<T>().0
        }

        pub fn to_cache_table_memory_size<T>(&self) -> usize {
            let (size, entry_size) = self.to_cache_table_and_entry_size::<T>();
            size * entry_size / 2_usize.pow(20)
        }
    }
}

pub mod classes {
    use super::*;
    // use std::collections::hash_map::DefaultHasher;
    // use std::hash::{Hash, Hasher};

    #[derive(Default, Clone, Debug)]
    pub struct RepetitionTable {
        count: Arc<Mutex<HashMap<u64, usize>>>,
    }

    impl RepetitionTable {
        pub fn new() -> Self {
            Self {
                count: Arc::new(Mutex::new(HashMap::default())),
            }
        }

        pub fn get_repetition(&self, key: u64) -> u8 {
            *self.count.lock().unwrap().get(&key).unwrap_or(&0) as u8
        }

        pub fn insert_and_get_repetition(&self, key: u64) -> u8 {
            let mut count_map = self.count.lock().unwrap();
            let count_entry = count_map.entry(key).or_insert(0);
            *count_entry += 1;
            *count_entry as u8
        }

        pub fn remove(&self, key: u64) {
            let mut count_map = self.count.lock().unwrap();
            let count_entry = count_map.get_mut(&key).unwrap_or_else(|| {
                panic!(
                    "Tried to remove the key {} that doesn't exist!",
                    hash_to_string(key)
                )
            });
            *count_entry -= 1;
            if *count_entry == 0 {
                count_map.remove(&key);
            }
        }

        pub fn clear(&self) {
            self.count.lock().unwrap().clear();
        }

        // fn hash<T: Hash>(t: &T) -> u64 {
        //     let mut s = DefaultHasher::new();
        //     t.hash(&mut s);
        //     s.finish()
        // }
    }
}

pub mod score_utils {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    enum Score {
        Cp(i16),
        Mate(i16),
        Infinity,
    }
}

pub mod unsafe_utils {
    use super::*;

    static mut COLORED_OUTPUT: bool = true;
    static mut T_TABLE_SIZE: CacheTableSize = INITIAL_T_TABLE_SIZE;

    pub fn is_colored_output() -> bool {
        unsafe { COLORED_OUTPUT }
    }

    pub fn set_colored_output(_bool: bool) {
        unsafe {
            COLORED_OUTPUT = _bool;
        }
        println!(
            "{} {}",
            colorize("Set colored output to", SUCCESS_MESSAGE_STYLE),
            colorize(_bool, INFO_STYLE),
        );
    }

    pub fn get_t_table_size() -> CacheTableSize {
        unsafe { T_TABLE_SIZE }
    }

    pub fn set_t_table_size(size: CacheTableSize) {
        unsafe {
            T_TABLE_SIZE = size;
        }
        println!(
            "{} {}",
            colorize("Set t-table size to", SUCCESS_MESSAGE_STYLE),
            colorize(size.to_cache_table_memory_size::<TranspositionTableEntry>(), INFO_STYLE),
        );
    }
}
