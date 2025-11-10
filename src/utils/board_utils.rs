use super::*;

static BOARD_SKELETON: &str = r"

     A   B   C   D   E   F   G   H
   +---+---+---+---+---+---+---+---+
 8 | ? | ? | ? | ? | ? | ? | ? | ? | 8
   +---+---+---+---+---+---+---+---+
 7 | ? | ? | ? | ? | ? | ? | ? | ? | 7
   +---+---+---+---+---+---+---+---+
 6 | ? | ? | ? | ? | ? | ? | ? | ? | 6
   +---+---+---+---+---+---+---+---+
 5 | ? | ? | ? | ? | ? | ? | ? | ? | 5
   +---+---+---+---+---+---+---+---+
 4 | ? | ? | ? | ? | ? | ? | ? | ? | 4
   +---+---+---+---+---+---+---+---+
 3 | ? | ? | ? | ? | ? | ? | ? | ? | 3
   +---+---+---+---+---+---+---+---+
 2 | ? | ? | ? | ? | ? | ? | ? | ? | 2
   +---+---+---+---+---+---+---+---+
 1 | ? | ? | ? | ? | ? | ? | ? | ? | 1
   +---+---+---+---+---+---+---+---+
     A   B   C   D   E   F   G   H    

";

pub fn get_board_string<'a>(
    colored: bool,
    empty_symbol_replacement_func: impl Fn(Square) -> Cow<'a, str>,
) -> String {
    let get_colored_char = |c: char| {
        let styles = match c {
            '+' | '-' | '|' => BOARD_SKELETON_STYLE,
            'a'..='h' | 'A'..='H' | '1'..='8' => BOARD_LABEL_STYLE,
            _ => &[],
        };
        c.colorize(styles)
    };
    let mut board_string = String::new();
    let mut squares_horizontal_mirror_iter = SQUARES_HORIZONTAL_MIRROR.iter();
    for c in BOARD_SKELETON.trim_matches('\n').chars() {
        if c == '?' {
            let square = squares_horizontal_mirror_iter
                .next()
                .copied()
                .expect("More 'O's in board skeleton than squares");
            board_string.push_str(&empty_symbol_replacement_func(square));
        } else if colored {
            board_string.push_str(&get_colored_char(c));
        } else {
            board_string.push(c);
        }
    }
    debug_assert!(
        squares_horizontal_mirror_iter.next().is_none(),
        "Fewer '?'s in board skeleton than squares"
    );
    board_string
}
