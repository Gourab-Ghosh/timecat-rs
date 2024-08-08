use super::*;

pub fn extract_pv_from_t_table(
    mini_board: &MiniBoard,
    transposition_table: &TranspositionTable,
) -> Vec<Move> {
    let mut pv = Vec::new();
    let best_move = transposition_table.read_best_move(mini_board.get_hash());
    if let Some(best_move) = best_move {
        pv.push(best_move);
        pv.append(&mut extract_pv_from_t_table(
            &mini_board.make_move_new(best_move),
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

pub fn get_pv_as_algebraic(mini_board: &MiniBoard, pv: &[Move], long: bool) -> String {
    let mut mini_board = mini_board.clone();
    let mut pv_string = String::new();
    for &move_ in pv {
        pv_string += &(if mini_board.is_legal(move_) {
            let (san, new_mini_board) = move_
                .algebraic_and_new_mini_board(&mini_board, long)
                .unwrap();
            mini_board = new_mini_board;
            san
        } else {
            move_.uci().colorize(ERROR_MESSAGE_STYLE)
        } + " ");
    }
    return pv_string.trim().to_string();
}

#[inline]
pub fn get_pv_as_san(mini_board: &MiniBoard, pv: &[Move]) -> String {
    get_pv_as_algebraic(mini_board, pv, false)
}

#[inline]
pub fn get_pv_as_lan(mini_board: &MiniBoard, pv: &[Move]) -> String {
    get_pv_as_algebraic(mini_board, pv, true)
}

#[inline]
pub fn get_pv_string(mini_board: &MiniBoard, pv: &[Move]) -> String {
    if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
        get_pv_as_algebraic(
            mini_board,
            pv,
            GLOBAL_TIMECAT_STATE.use_long_algebraic_notation(),
        )
    } else {
        get_pv_as_uci(pv)
    }
}
