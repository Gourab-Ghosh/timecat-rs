use super::*;

#[inline]
pub fn polyglot_move_int_to_move(move_int: u16) -> Result<Move> {
    Move::new(
        (((move_int >> 6) & 0x3F) as u8).decompress()?,
        ((move_int & 0x3F) as u8).decompress()?,
        *const { [None, Some(Knight), Some(Bishop), Some(Rook), Some(Queen)] }
            .get((move_int >> 12) as usize)
            .ok_or(TimecatError::BadPolyglotFile)?,
    )
}

#[inline]
pub const fn move_to_polyglot_move_int(move_: Move) -> Result<u16> {
    let mut move_int = match move_.get_promotion() {
        None => 0,
        Some(Knight) => 1,
        Some(Bishop) => 2,
        Some(Rook) => 3,
        Some(Queen) => 4,
        _ => return Err(TimecatError::PolyglotTableParseError),
    };
    move_int = (move_int << 6) ^ move_.get_source() as u16;
    move_int = (move_int << 6) ^ move_.get_dest() as u16;
    Ok(move_int)
}
