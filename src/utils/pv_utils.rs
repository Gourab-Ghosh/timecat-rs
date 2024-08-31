use super::*;

pub fn extract_pv_from_t_table(
    position: &BoardPosition,
    transposition_table: &TranspositionTable,
) -> Vec<Move> {
    let mut pv = Vec::new();
    let best_move = transposition_table.read_best_move(position.get_hash());
    if let Some(best_move) = best_move {
        pv.push(best_move);
        pv.append(&mut extract_pv_from_t_table(
            &position.make_move_new(best_move),
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

pub fn get_pv_as_algebraic(position: &BoardPosition, pv: &[Move], long: bool) -> String {
    let mut position = position.clone();
    let mut pv_string = String::new();
    for &move_ in pv {
        pv_string += &(if position.is_legal(move_) {
            let (san, new_position) = move_
                .algebraic_and_new_position(&position, long)
                .unwrap();
            position = new_position;
            san
        } else {
            move_.uci().colorize(ERROR_MESSAGE_STYLE)
        } + " ");
    }
    return pv_string.trim().to_string();
}

#[inline]
pub fn get_pv_as_san(position: &BoardPosition, pv: &[Move]) -> String {
    get_pv_as_algebraic(position, pv, false)
}

#[inline]
pub fn get_pv_as_lan(position: &BoardPosition, pv: &[Move]) -> String {
    get_pv_as_algebraic(position, pv, true)
}

#[inline]
pub fn get_pv_string(position: &BoardPosition, pv: &[Move]) -> String {
    if GLOBAL_TIMECAT_STATE.is_in_console_mode() {
        get_pv_as_algebraic(
            position,
            pv,
            GLOBAL_TIMECAT_STATE.use_long_algebraic_notation(),
        )
    } else {
        get_pv_as_uci(pv)
    }
}
