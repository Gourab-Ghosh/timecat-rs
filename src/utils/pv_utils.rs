use super::*;

pub fn extract_pv_from_t_table(board: &mut Board) -> Vec<Move> {
    let mut pv = Vec::new();
    let best_move = TRANSPOSITION_TABLE.read_best_move(board.hash());
    if let Some(best_move) = best_move {
        pv.push(best_move);
        board.push(best_move);
        pv.append(&mut extract_pv_from_t_table(board));
        board.pop();
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

pub fn get_pv_as_algebraic(board: &Board, pv: &[Option<Move>], long: bool) -> String {
    let mut board = board.clone();
    let mut pv_string = String::new();
    for &move_ in pv {
        let is_legal_move = if let Some(move_) = move_ {
            board.is_legal(move_)
        } else {
            false
        };
        pv_string += &(if is_legal_move {
            board.algebraic_and_push(move_, long).unwrap()
        } else {
            move_.uci().colorize(ERROR_MESSAGE_STYLE)
        } + " ");
    }
    return pv_string.trim().to_string();
}

#[inline(always)]
pub fn get_pv_as_san(board: &Board, pv: &[Option<Move>]) -> String {
    get_pv_as_algebraic(board, pv, false)
}

#[inline(always)]
pub fn get_pv_as_lan(board: &Board, pv: &[Option<Move>]) -> String {
    get_pv_as_algebraic(board, pv, true)
}

#[inline(always)]
pub fn get_pv_string(board: &Board, pv: &[Option<Move>]) -> String {
    if UCI_STATE.is_in_console_mode() {
        get_pv_as_algebraic(board, pv, UCI_STATE.use_long_algebraic_notation())
    } else {
        get_pv_as_uci(pv)
    }
}
