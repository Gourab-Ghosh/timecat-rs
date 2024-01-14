use super::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum CastleRights {
    None,
    KingSide,
    QueenSide,
    Both,
}

impl CastleRights {
    // /// Can I castle kingside?
    // pub fn has_kingside(self) -> bool {
    //     self.to_index() & 1 == 1
    // }

    // /// Can I castle queenside?
    // pub fn has_queenside(self) -> bool {
    //     self.to_index() & 2 == 2
    // }

    // pub fn square_to_castle_rights(color: Color, sq: Square) -> CastleRights {
    //     CastleRights::from_index(unsafe {
    //         *CASTLES_PER_SQUARE
    //             .get_unchecked(color.to_index())
    //             .get_unchecked(sq.to_index())
    //     } as usize)
    // }

    // /// What squares need to be empty to castle kingside?
    // pub fn kingside_squares(self, color: Color) -> BitBoard {
    //     unsafe { *KINGSIDE_CASTLE_SQUARES.get_unchecked(color.to_index()) }
    // }

    // /// What squares need to be empty to castle queenside?
    // pub fn queenside_squares(self, color: Color) -> BitBoard {
    //     unsafe { *QUEENSIDE_CASTLE_SQUARES.get_unchecked(color.to_index()) }
    // }

    // /// Remove castle rights, and return a new `CastleRights`.
    // pub fn remove(self, remove: CastleRights) -> CastleRights {
    //     CastleRights::from_index(self.to_index() & !remove.to_index())
    // }

    // /// Add some castle rights, and return a new `CastleRights`.
    // pub fn add(self, add: CastleRights) -> CastleRights {
    //     CastleRights::from_index(self.to_index() | add.to_index())
    // }

    /// Convert `CastleRights` to `usize` for table lookups
    pub fn to_index(self) -> usize {
        self as usize
    }

    /// Convert `usize` to `CastleRights`.  Panic if invalid number.
    pub fn from_index(i: usize) -> CastleRights {
        match i {
            0 => CastleRights::None,
            1 => CastleRights::KingSide,
            2 => CastleRights::QueenSide,
            3 => CastleRights::Both,
            _ => unreachable!(),
        }
    }

    // /// Which rooks can we "guarantee" we haven't moved yet?
    // pub fn unmoved_rooks(self, color: Color) -> BitBoard {
    //     match self {
    //         CastleRights::None => EMPTY_BITBOARD,
    //         CastleRights::KingSide => BitBoard::set(color.to_my_backrank(), File::H),
    //         CastleRights::QueenSide => BitBoard::set(color.to_my_backrank(), File::A),
    //         CastleRights::Both => {
    //             BitBoard::set(color.to_my_backrank(), File::A)
    //                 ^ BitBoard::set(color.to_my_backrank(), File::H)
    //         }
    //     }
    // }

    pub fn to_string(self, color: Color) -> String {
        let result = match self {
            CastleRights::None => "",
            CastleRights::KingSide => "k",
            CastleRights::QueenSide => "q",
            CastleRights::Both => "kq",
        };

        if color == Color::White {
            result.to_uppercase()
        } else {
            result.to_string()
        }
    }

    /// Given a square of a rook, which side is it on?
    /// Note: It is invalid to pass in a non-rook square.  The code may panic.
    pub fn rook_square_to_castle_rights(square: Square) -> CastleRights {
        match square.get_file() {
            File::A => CastleRights::QueenSide,
            File::H => CastleRights::KingSide,
            _ => unreachable!(),
        }
    }
}
