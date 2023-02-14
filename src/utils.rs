use super::*;

pub mod command_utils {
    use super::*;
    use std::io::Write;

    pub fn input<T: std::fmt::Display>(q: T) -> String {
        print!("{q}");
        std::io::stdout().flush().unwrap();
        let mut user_input = String::new();
        std::io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line!");
        user_input
    }

    fn simplify_command(user_input: &str) -> String {
        let user_input = user_input.trim();
        let mut user_input = user_input.to_string();
        for _char in [",", ":"] {
            user_input = user_input.replace(_char, " ")
        }
        user_input = remove_double_spaces(&user_input);
        user_input
    }

    pub fn parse_command(board: &mut Board, user_input: &str) -> Option<String> {
        let DEFAULT_ERROR = Some(String::new());
        let NOT_IMPLEMENTED_ERROR = Some(String::from(
            "Sorry, this command is not implemented yet :(",
        ));
        let BAD_FEN_ERROR = Some(String::from("The given fen is wrong fen! Try again!"));
        let BAD_BOOL_ERROR = Some(String::from(
            "The given boolean value is wrong fen! Try again!",
        ));
        let ILLEGAL_MOVE_ERROR = Some(String::from(
            "The move you are trying to make is illegal! Try again!",
        ));

        let user_input = simplify_command(user_input);
        let user_input = user_input.as_str();
        let mut commands = user_input.split(' ');
        let first_command = match commands.next() {
            Some(second_command) => second_command,
            None => return DEFAULT_ERROR,
        }
        .to_lowercase();

        if user_input.to_lowercase() == "d" {
            println!("{}", board);
            return None;
        }

        if ["go", "do"].contains(&first_command.as_str()) {
            let second_command = match commands.next() {
                Some(second_command) => second_command,
                None => return DEFAULT_ERROR,
            }
            .to_lowercase();
            let depth: u8 = commands.next().unwrap_or_default().parse().unwrap_or(0);
            if commands.next().is_some() {
                return DEFAULT_ERROR;
            }
            if depth == 0 {
                return Some("Invalid depth in perft! Try again!".to_string());
            }
            let mut board = Board::from(board);
            println!("{}\n", board);
            let position_count: usize;
            let now = Instant::now();

            if second_command == "perft" {
                position_count = perft(&mut board, depth);
            } else if second_command == "depth" {
                return NOT_IMPLEMENTED_ERROR;
            } else {
                return DEFAULT_ERROR;
            }

            let elapsed_time = now.elapsed().as_secs_f64();
            println!("\nPosition Count: {}", position_count);
            println!("Time: {} s", elapsed_time as f32);
            println!(
                "Speed: {} Nodes/s",
                ((position_count as f64) / elapsed_time) as usize
            );
            return None;
        }

        if first_command == "set" {
            let second_command = match commands.next() {
                Some(command) => command,
                None => return DEFAULT_ERROR,
            }
            .to_lowercase();
            if second_command == "board" {
                let third_command = match commands.next() {
                    Some(command) => command,
                    None => return DEFAULT_ERROR,
                }
                .to_lowercase();
                if third_command == "fen" {
                    let mut fen = String::new();
                    for fen_part in commands {
                        fen.push_str(fen_part);
                        fen.push(' ');
                    }
                    if !Board::is_good_fen(&fen) {
                        return BAD_FEN_ERROR;
                    }
                    board.set_fen(&fen);
                    println!("{board}");
                    return None;
                }
            } else if second_command == "color" {
                let third_command = match commands.next() {
                    Some(command) => command,
                    None => return DEFAULT_ERROR,
                }
                .to_lowercase();
                if commands.next().is_some() {
                    return DEFAULT_ERROR;
                }
                if third_command == "true" {
                    set_colored_output(true);
                } else if third_command == "false" {
                    set_colored_output(false);
                } else {
                    return BAD_BOOL_ERROR;
                }
                return None;
            }
        }

        if first_command == "push" {
            let second_command = match commands.next() {
                Some(command) => command,
                None => return DEFAULT_ERROR,
            }
            .to_lowercase();
            for move_text in commands {
                let possible_move: Result<Move, ChessError>;
                if second_command == "san" {
                    possible_move = board.get_move_from_san(move_text);
                } else if second_command == "uci" {
                    possible_move = board.get_move_from_uci(move_text);
                } else {
                    return DEFAULT_ERROR;
                }
                let _move = match possible_move {
                    Ok(_move) => _move,
                    Err(e) => return Some(e.to_string() + "! Try again!"),
                };
                if !board.is_legal(_move) {
                    return ILLEGAL_MOVE_ERROR;
                }
                board.push(_move);
                println!(
                    "{} {}",
                    colorize("Made move:", SUCCESS_MESSAGE_STYLE),
                    colorize(move_text, INFO_STYLE),
                );
            }
            return None;
        }

        DEFAULT_ERROR
    }

    pub fn is_checkmate(score: Score) -> bool {
        score.abs() > CHECKMATE_THRESHOLD
    }

    pub fn evaluate_piece(piece: Piece) -> i16 {
        match piece {
            Pawn => PAWN_VALUE,
            Knight => (32 * PAWN_VALUE) / 10,
            Bishop => (33 * PAWN_VALUE) / 10,
            Rook => 5 * PAWN_VALUE,
            Queen => 9 * PAWN_VALUE,
            King => 20 * PAWN_VALUE,
        }
    }
}

pub mod string_utils {
    use super::*;
    use colored::{ColoredString, Colorize};
    use std::fmt::Display;

    pub fn remove_double_spaces(s: &str) -> String {
        let mut s = s.to_owned();
        while s.contains("  ") {
            s = s.replace("  ", " ");
        }
        return s.trim().to_string();
    }

    pub fn simplify_fen(fen: &str) -> String {
        remove_double_spaces(fen)
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
            "bold" => s.bold(),
            &_ => panic!("Cannot colorize string to {}", color),
        };
    }

    pub fn colorize<T: ToString>(obj: T, styles: &str) -> String {
        let s = obj.to_string();
        let s = s.as_str();
        if !is_colored_output() {
            return s.to_string();
        }
        let styles = remove_double_spaces(styles);
        let styles = styles.trim();
        if styles.is_empty() {
            return s.to_string();
        }
        let mut colored_string = s.clear();
        for style in remove_double_spaces(styles).split(' ') {
            colored_string = colorize_string(colored_string, style);
        }
        colored_string.to_string()
    }

    pub fn score_to_string(score: Score) -> String {
        (score as f32 / 100.0).to_string()
    }

    pub fn hash_to_string(hash: u64) -> String {
        return format!("{:x}", hash).to_uppercase();
    }
}

pub mod square_utils {
    use super::*;

    pub fn square_mirror(square: Square) -> Square {
        SQUARES_180[square.to_index()]
    }
}

pub mod classes {
    use super::*;
    // use std::collections::hash_map::DefaultHasher;
    // use std::hash::{Hash, Hasher};

    #[derive(Default, Clone)]
    pub struct RepetitionTable {
        count: Arc<Mutex<HashMap<u64, usize>>>,
    }

    impl RepetitionTable {
        pub fn new() -> Self {
            Self {
                count: Arc::new(Mutex::new(HashMap::default())),
            }
        }

        pub fn insert_and_get_repetition(&self, key: u64) -> u8 {
            let mut count_map = self.count.lock().unwrap();
            let count_entry = count_map.entry(key).or_insert(0);
            *count_entry += 1;
            *count_entry as u8
        }

        pub fn insert_and_detect_threefold_repetition(&self, key: u64) -> bool {
            let mut count_map = self.count.lock().unwrap();
            let count_entry = count_map.entry(key).or_insert(0);
            *count_entry += 1;
            *count_entry > 3
        }

        pub fn remove(&self, key: u64) {
            let mut count_map = self.count.lock().unwrap();
            if let Some(count_entry) = count_map.get_mut(&key) {
                *count_entry -= 1;
                if *count_entry == 0 {
                    count_map.remove(&key);
                }
            } else {
                panic!(
                    "Tried to remove the key {} that doesn't exist!",
                    hash_to_string(key)
                );
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

    pub fn set_colored_output(b: bool) {
        unsafe {
            COLORED_OUTPUT = b;
        }
        println!(
            "\n{} {}\n",
            colorize("Set colored output to", SUCCESS_MESSAGE_STYLE),
            colorize(b, INFO_STYLE),
        );
    }
}
