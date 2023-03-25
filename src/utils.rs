use super::*;

pub mod common_utils {
    use super::*;

    #[inline(always)]
    pub fn is_checkmate(score: Score) -> bool {
        score.abs() > CHECKMATE_THRESHOLD
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

    pub fn format_info<T: ToString>(desc: &str, info: T) -> String {
        format!("{}: {}", colorize(desc, INFO_STYLE), info.to_string())
    }

    pub fn println_info<T: ToString>(desc: &str, info: T) {
        println!("{}", format_info(desc, info));
    }

    pub fn get_upper_board_mask(rank: Rank, color: Color) -> BitBoard {
        get_item_unchecked!(UPPER_BOARD_MASK, color.to_index(), rank.to_index())
    }

    pub fn get_lower_board_mask(rank: Rank, color: Color) -> BitBoard {
        get_item_unchecked!(UPPER_BOARD_MASK, 1 - color.to_index(), rank.to_index())
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
        return match color {
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
        };
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
        if is_checkmate(score) {
            let mut mate_string = String::from(if score.is_positive() { "M" } else { "-M" });
            let mate_distance = (CHECKMATE_SCORE - score.abs() + 1) / 2;
            mate_string += &mate_distance.to_string();
            return mate_string;
        }
        format!("{:.5}", (score as f32 / PAWN_VALUE as f32).to_string())
    }

    pub fn hash_to_string(hash: u64) -> String {
        return format!("{:x}", hash).to_uppercase();
    }
}

pub mod square_utils {
    use super::*;

    #[inline(always)]
    pub fn square_mirror(square: Square) -> Square {
        get_item_unchecked!(SQUARES_180, square.to_index())
    }

    #[inline(always)]
    pub fn get_square_bb(square: Square) -> BitBoard {
        get_item_unchecked!(BB_SQUARES, square.to_index())
    }

    pub fn square_distance(square1: Square, square2: Square) -> u8 {
        let (file1, rank1) = (square1.get_file(), square1.get_rank());
        let (file2, rank2) = (square2.get_file(), square2.get_rank());
        let file_distance = (file1 as i8 - file2 as i8).abs() as u8;
        let rank_distance = (rank1 as i8 - rank2 as i8).abs() as u8;
        return file_distance.max(rank_distance);
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

pub mod unsafe_utils {
    use super::*;

    static mut COLORED_OUTPUT: bool = true;

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
}
