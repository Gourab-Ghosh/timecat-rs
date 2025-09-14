use super::*;

static BOARD_SKELETON: &str = r"

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

pub fn get_board_skeleton() -> String {
    let skeleton = String::from(BOARD_SKELETON.trim_matches('\n'));
    let mut colored_skeleton = String::new();
    fn get_colored_char(c: char) -> String {
        let styles = match c {
            '+' | '-' | '|' => BOARD_SKELETON_STYLE,
            'a'..='h' | 'A'..='H' | '1'..='8' => BOARD_LABEL_STYLE,
            _ => &[],
        };
        c.colorize(styles)
    }
    for c in skeleton.chars() {
        colored_skeleton.push_str(&get_colored_char(c));
    }
    colored_skeleton
}
