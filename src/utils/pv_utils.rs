use super::*;

pub fn extract_pv_from_t_table(
    minimum_board: &MinimumBoard,
    transposition_table: &TranspositionTable,
) -> Vec<Move> {
    let mut pv = Vec::new();
    let best_move = transposition_table.read_best_move(minimum_board.get_hash());
    if let Some(best_move) = best_move {
        pv.push(best_move);
        pv.append(&mut extract_pv_from_t_table(
            &minimum_board.make_move_new(best_move),
            transposition_table,
        ));
    }
    pv
}

pub fn get_pv_as_uci(pv: &[Move]) -> String {
    let mut pv_string = String::new();
    for move_ in pv {
        pv_string += &(move_.uci() + " ");
    }
    return pv_string.trim().to_string();
}

pub fn get_pv_as_algebraic(minimum_board: &MinimumBoard, pv: &[Move], long: bool) -> String {
    let mut minimum_board = minimum_board.clone();
    let mut pv_string = String::new();
    for &move_ in pv {
        pv_string += &(if minimum_board.is_legal(move_) {
            let (san, new_minimum_board) = move_.algebraic_and_new_minimum_board(&minimum_board, long).unwrap();
            minimum_board = new_minimum_board;
            san
        } else {
            move_.uci().colorize(ERROR_MESSAGE_STYLE)
        } + " ");
    }
    return pv_string.trim().to_string();
}

#[inline]
pub fn get_pv_as_san(minimum_board: &MinimumBoard, pv: &[Move]) -> String {
    get_pv_as_algebraic(minimum_board, pv, false)
}

#[inline]
pub fn get_pv_as_lan(minimum_board: &MinimumBoard, pv: &[Move]) -> String {
    get_pv_as_algebraic(minimum_board, pv, true)
}

#[inline]
pub fn get_pv_string(minimum_board: &MinimumBoard, pv: &[Move]) -> String {
    if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
        get_pv_as_algebraic(
            minimum_board,
            pv,
            GLOBAL_TIMECAT_STATE.use_long_algebraic_notation(),
        )
    } else {
        get_pv_as_uci(pv)
    }
}
