use super::*;

pub fn extract_pv_from_t_table(
    sub_board: &SubBoard,
    transposition_table: &TranspositionTable,
) -> Vec<Move> {
    let mut pv = Vec::new();
    let best_move = transposition_table.read_best_move(sub_board.get_hash());
    if let Some(best_move) = best_move {
        pv.push(best_move);
        pv.append(&mut extract_pv_from_t_table(
            &sub_board.make_move_new(best_move),
            transposition_table,
        ));
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

pub fn get_pv_as_algebraic(sub_board: &SubBoard, pv: &[Option<Move>], long: bool) -> String {
    let mut sub_board = sub_board.clone();
    let mut pv_string = String::new();
    for &optional_move in pv {
        let is_legal_move = if let Some(optional_move) = optional_move {
            sub_board.is_legal(optional_move)
        } else {
            false
        };
        pv_string += &(if is_legal_move {
            let (san, new_sub_board) = optional_move
                .unwrap()
                .algebraic_and_new_sub_board(&sub_board, long)
                .unwrap();
            sub_board = new_sub_board;
            san
        } else {
            optional_move.uci().colorize(ERROR_MESSAGE_STYLE)
        } + " ");
    }
    return pv_string.trim().to_string();
}

#[inline]
pub fn get_pv_as_san(sub_board: &SubBoard, pv: &[Option<Move>]) -> String {
    get_pv_as_algebraic(sub_board, pv, false)
}

#[inline]
pub fn get_pv_as_lan(sub_board: &SubBoard, pv: &[Option<Move>]) -> String {
    get_pv_as_algebraic(sub_board, pv, true)
}

#[inline]
pub fn get_pv_string(sub_board: &SubBoard, pv: &[Option<Move>]) -> String {
    if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
        get_pv_as_algebraic(
            sub_board,
            pv,
            GLOBAL_TIMECAT_STATE.use_long_algebraic_notation(),
        )
    } else {
        get_pv_as_uci(pv)
    }
}
