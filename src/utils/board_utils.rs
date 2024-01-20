use super::*;

pub fn get_board_skeleton() -> String {
    let skeleton = String::from(BOARD_SKELETON.trim_matches('\n'));
    let mut colored_skeleton = String::new();
    fn get_colored_char(c: char) -> String {
        let mut _char = c.to_string();
        let styles = if "+-|".contains(c) {
            BOARD_SKELETON_STYLE
        } else if "abcdefghABCDEFGH12345678".contains(c) {
            BOARD_LABEL_STYLE
        } else {
            &[]
        };
        _char.colorize(styles)
    }
    for c in skeleton.chars() {
        colored_skeleton.push_str(&get_colored_char(c));
    }
    colored_skeleton
}
