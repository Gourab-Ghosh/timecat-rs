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

pub fn get_board_skeleton(colored: bool) -> String {
    let skeleton = String::from(BOARD_SKELETON.trim_matches('\n'));
    if !colored {
        return skeleton;
    }
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
