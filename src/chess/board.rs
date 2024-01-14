use super::*;

#[derive(Eq, Clone, Copy, Debug)]
pub struct SubBoard {
    pieces: [BitBoard; NUM_PIECES],
    occupied_co: [BitBoard; NUM_COLORS],
    occupied: BitBoard,
    side_to_move: Color,
    castle_rights: [CastleRights; NUM_COLORS],
    en_passant: Option<Square>,
    pinned: BitBoard,
    checkers: BitBoard,
    transposition_key: u64,
    halfmove_number: u8,
    fullmove_count: u16,
}

impl PartialEq for SubBoard {
    fn eq(&self, other: &Self) -> bool {
        self.transposition_key_components()
            .eq(&other.transposition_key_components())
    }
}

impl SubBoard {
    fn transposition_key_components(
        &self,
    ) -> (
        [bitboard::BitBoard; 6],
        [bitboard::BitBoard; 2],
        color::Color,
        [castle::CastleRights; 2],
        Option<square::Square>,
    ) {
        (
            self.pieces,
            self.occupied_co,
            self.side_to_move,
            self.castle_rights,
            self.en_passant,
        )
    }
}

impl SubBoard {
    fn new_empty() -> Self {
        Self {
            pieces: [EMPTY_BITBOARD; NUM_PIECES],
            occupied_co: [EMPTY_BITBOARD; NUM_COLORS],
            occupied: EMPTY_BITBOARD,
            side_to_move: Color::White,
            castle_rights: [CastleRights::None; NUM_COLORS],
            pinned: EMPTY_BITBOARD,
            checkers: EMPTY_BITBOARD,
            transposition_key: 0,
            en_passant: None,
            halfmove_number: 0,
            fullmove_count: 1,
        }
    }

    // #[inline(always)]
    // pub fn status(&self) -> BoardStatus {
    //     let moves = MoveGen::new_legal(&self).len();
    //     match moves {
    //         0 => {
    //             if self.checkers == EMPTY_BITBOARD {
    //                 BoardStatus::Stalemate
    //             } else {
    //                 BoardStatus::Checkmate
    //             }
    //         }
    //         _ => BoardStatus::Ongoing,
    //     }
    // }

    #[inline(always)]
    pub fn occupied(&self) -> &BitBoard {
        &self.occupied
    }

    #[inline(always)]
    pub fn occupied_co(&self, color: Color) -> &BitBoard {
        unsafe { self.occupied_co.get_unchecked(color.to_index()) }
    }

    #[inline(always)]
    pub fn king_square(&self, color: Color) -> Square {
        (self.pieces(PieceType::King) & self.occupied_co(color)).to_square()
    }

    #[inline(always)]
    pub fn pieces(&self, piece: PieceType) -> &BitBoard {
        unsafe { self.pieces.get_unchecked(piece.to_index()) }
    }

    #[inline(always)]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        unsafe { *self.castle_rights.get_unchecked(color.to_index()) }
    }

    #[inline(always)]
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline(always)]
    pub fn my_castle_rights(&self) -> CastleRights {
        self.castle_rights(self.side_to_move())
    }

    #[inline(always)]
    pub fn their_castle_rights(&self) -> CastleRights {
        self.castle_rights(!self.side_to_move())
    }

    // fn xor(&mut self, piece: PieceType, bb: BitBoard, color: Color) {
    //     unsafe {
    //         *self.pieces.get_unchecked_mut(piece.to_index()) ^= bb;
    //         *self.occupied_co.get_unchecked_mut(color.to_index()) ^= bb;
    //         self.occupied ^= bb;
    //         self.hash ^= Zobrist::piece(piece, bb.to_square(), color);
    //     }
    // }

    // #[inline(always)]
    // pub fn null_move(&self) -> Option<Self> {
    //     if self.checkers != EMPTY_BITBOARD {
    //         None
    //     } else {
    //         let mut result = *self;
    //         result.side_to_move = !result.side_to_move;
    //         result.remove_ep();
    //         result.update_pin_info();
    //         Some(result)
    //     }
    // }

    // pub fn is_sane(&self) -> bool {
    //     // make sure there is no square with multiple pieces on it
    //     for x in ALL_PIECES.iter() {
    //         for y in ALL_PIECES.iter() {
    //             if *x != *y {
    //                 if self.pieces(*x) & self.pieces(*y) != EMPTY_BITBOARD {
    //                     return false;
    //                 }
    //             }
    //         }
    //     }

    //     // make sure the colors don't overlap, either
    //     if self.occupied_co(Color::White) & self.occupied_co(Color::Black) != EMPTY_BITBOARD {
    //         return false;
    //     }

    //     // grab all the pieces by OR'ing together each piece() BitBoard
    //     let occupied = ALL_PIECES
    //         .iter()
    //         .fold(EMPTY_BITBOARD, |cur, next| cur | self.pieces(*next));

    //     // make sure that's equal to the occupied bitboard
    //     if occupied != *self.occupied() {
    //         return false;
    //     }

    //     // make sure there is exactly one white king
    //     if (self.pieces(PieceType::King) & self.occupied_co(Color::White)).popcnt() != 1 {
    //         return false;
    //     }

    //     // make sure there is exactly one black king
    //     if (self.pieces(PieceType::King) & self.occupied_co(Color::Black)).popcnt() != 1 {
    //         return false;
    //     }

    //     // make sure the en_passant square has a pawn on it of the right color
    //     match self.en_passant {
    //         None => {}
    //         Some(x) => {
    //             if self.pieces(PieceType::Pawn)
    //                 & self.occupied_co(!self.side_to_move)
    //                 & BitBoard::from_square(x)
    //                 == EMPTY_BITBOARD
    //             {
    //                 return false;
    //             }
    //         }
    //     }

    //     // make sure my opponent is not currently in check (because that would be illegal)
    //     let mut board_copy = *self;
    //     board_copy.side_to_move = !board_copy.side_to_move;
    //     board_copy.update_pin_info();
    //     if board_copy.checkers != EMPTY_BITBOARD {
    //         return false;
    //     }

    //     // for each color, verify that, if they have castle rights, that they haven't moved their
    //     // rooks or king
    //     for color in ALL_COLORS.iter() {
    //         // get the castle rights
    //         let castle_rights = self.castle_rights(*color);

    //         // the castle rights object will tell us which rooks shouldn't have moved yet.
    //         // verify there are rooks on all those squares
    //         if castle_rights.unmoved_rooks(*color)
    //             & self.pieces(PieceType::Rook)
    //             & self.occupied_co(*color)
    //             != castle_rights.unmoved_rooks(*color)
    //         {
    //             return false;
    //         }
    //         // if we have castle rights, make sure we have a king on the (E, {1,8}) square,
    //         // depending on the color
    //         if castle_rights != CastleRights::None {
    //             if self.pieces(PieceType::King) & self.occupied_co(*color)
    //                 != get_file(File::E) & get_rank(color.to_my_backrank())
    //             {
    //                 return false;
    //             }
    //         }
    //     }

    //     // we must make sure the kings aren't touching
    //     if get_king_moves(self.king_square(Color::White)) & self.pieces(PieceType::King) != EMPTY_BITBOARD {
    //         return false;
    //     }

    //     // it checks out
    //     return true;
    // }

    // #[inline(always)]
    // pub fn get_hash(&self) -> u64 {
    //     self.hash
    //         ^ if let Some(ep) = self.en_passant {
    //             Zobrist::en_passant(ep.get_file(), !self.side_to_move)
    //         } else {
    //             0
    //         }
    //         ^ Zobrist::castles(
    //             self.castle_rights[self.side_to_move.to_index()],
    //             self.side_to_move,
    //         )
    //         ^ Zobrist::castles(
    //             self.castle_rights[(!self.side_to_move).to_index()],
    //             !self.side_to_move,
    //         )
    //         ^ if self.side_to_move == Color::Black {
    //             Zobrist::color()
    //         } else {
    //             0
    //         }
    // }

    #[inline(always)]
    pub fn get_pawn_hash(&self) -> u64 {
        todo!()
    }

    #[inline(always)]
    pub fn piece_on(&self, square: Square) -> Option<PieceType> {
        // TODO: check speed
        let opp = BitBoard::from_square(square);
        if self.occupied() & opp == EMPTY_BITBOARD {
            None
        } else {
            //naive algorithm
            /*
            for p in ALL_PIECES {
                if self.pieces(*p) & opp {
                    return p;
                }
            } */
            if (self.pieces(PieceType::Pawn)
                ^ self.pieces(PieceType::Knight)
                ^ self.pieces(PieceType::Bishop))
                & opp
                != EMPTY_BITBOARD
            {
                if self.pieces(PieceType::Pawn) & opp != EMPTY_BITBOARD {
                    Some(PieceType::Pawn)
                } else if self.pieces(PieceType::Knight) & opp != EMPTY_BITBOARD {
                    Some(PieceType::Knight)
                } else {
                    Some(PieceType::Bishop)
                }
            } else {
                if self.pieces(PieceType::Rook) & opp != EMPTY_BITBOARD {
                    Some(PieceType::Rook)
                } else if self.pieces(PieceType::Queen) & opp != EMPTY_BITBOARD {
                    Some(PieceType::Queen)
                } else {
                    Some(PieceType::King)
                }
            }
        }
    }

    #[inline(always)]
    pub fn color_on(&self, square: Square) -> Option<Color> {
        if (self.occupied_co(Color::White) & BitBoard::from_square(square)) != EMPTY_BITBOARD {
            Some(Color::White)
        } else if (self.occupied_co(Color::Black) & BitBoard::from_square(square)) != EMPTY_BITBOARD
        {
            Some(Color::Black)
        } else {
            None
        }
    }

    fn remove_ep(&mut self) {
        self.en_passant = None;
    }

    #[inline(always)]
    pub fn en_passant(self) -> Option<Square> {
        self.en_passant
    }

    // fn set_ep(&mut self, sq: Square) {
    //     // Only set self.en_passant if the pawn can actually be captured next move.
    //     if get_adjacent_files(sq.get_file())
    //         & get_rank(sq.get_rank())
    //         & self.pieces(PieceType::Pawn)
    //         & self.occupied_co(!self.side_to_move)
    //         != EMPTY_BITBOARD
    //     {
    //         self.en_passant = Some(sq);
    //     }
    // }

    // #[inline(always)]
    // pub fn legal(&self, m: Move) -> bool {
    //     MoveGen::new_legal(&self).find(|x| *x == m).is_some()
    // }

    // #[inline(always)]
    // pub fn make_move_new(&self, m: Move) -> Self {
    //     let mut result = mem::MaybeUninit::<Self>::uninit();
    //     unsafe {
    //         self.make_move(m, &mut *result.as_mut_ptr());
    //         result.assume_init()
    //     }
    // }

    // #[inline(always)]
    // pub fn make_move(&self, m: Move, result: &mut Self) {
    //     *result = *self;
    //     result.remove_ep();
    //     result.checkers = EMPTY_BITBOARD;
    //     result.pinned = EMPTY_BITBOARD;
    //     let source = m.get_source();
    //     let dest = m.get_dest();

    //     let source_bb = BitBoard::from_square(source);
    //     let dest_bb = BitBoard::from_square(dest);
    //     let move_bb = source_bb ^ dest_bb;
    //     let moved = self.piece_on(source).unwrap();

    //     result.xor(moved, source_bb, self.side_to_move);
    //     result.xor(moved, dest_bb, self.side_to_move);
    //     if let Some(captured) = self.piece_on(dest) {
    //         result.xor(captured, dest_bb, !self.side_to_move);
    //     }

    //     #[allow(deprecated)]
    //     result.remove_their_castle_rights(CastleRights::square_to_castle_rights(
    //         !self.side_to_move,
    //         dest,
    //     ));

    //     #[allow(deprecated)]
    //     result.remove_my_castle_rights(CastleRights::square_to_castle_rights(
    //         self.side_to_move,
    //         source,
    //     ));

    //     let opp_king = result.pieces(PieceType::King) & result.occupied_co(!result.side_to_move);

    //     let castles = moved == PieceType::King && (move_bb & get_castle_moves()) == move_bb;

    //     let ksq = opp_king.to_square();

    //     const CASTLE_ROOK_START: [File; 8] = [
    //         File::A,
    //         File::A,
    //         File::A,
    //         File::A,
    //         File::H,
    //         File::H,
    //         File::H,
    //         File::H,
    //     ];
    //     const CASTLE_ROOK_END: [File; 8] = [
    //         File::D,
    //         File::D,
    //         File::D,
    //         File::D,
    //         File::F,
    //         File::F,
    //         File::F,
    //         File::F,
    //     ];

    //     if moved == PieceType::Knight {
    //         result.checkers ^= get_knight_moves(ksq) & dest_bb;
    //     } else if moved == PieceType::Pawn {
    //         if let Some(PieceType::Knight) = m.get_promotion() {
    //             result.xor(PieceType::Pawn, dest_bb, self.side_to_move);
    //             result.xor(PieceType::Knight, dest_bb, self.side_to_move);
    //             result.checkers ^= get_knight_moves(ksq) & dest_bb;
    //         } else if let Some(promotion) = m.get_promotion() {
    //             result.xor(PieceType::Pawn, dest_bb, self.side_to_move);
    //             result.xor(promotion, dest_bb, self.side_to_move);
    //         } else if (source_bb & get_pawn_source_double_moves()) != EMPTY_BITBOARD
    //             && (dest_bb & get_pawn_dest_double_moves()) != EMPTY_BITBOARD
    //         {
    //             result.set_ep(dest);
    //             result.checkers ^= get_pawn_attacks(ksq, !result.side_to_move, dest_bb);
    //         } else if Some(dest.ubackward(self.side_to_move)) == self.en_passant {
    //             result.xor(
    //                 PieceType::Pawn,
    //                 BitBoard::from_square(dest.ubackward(self.side_to_move)),
    //                 !self.side_to_move,
    //             );
    //             result.checkers ^= get_pawn_attacks(ksq, !result.side_to_move, dest_bb);
    //         } else {
    //             result.checkers ^= get_pawn_attacks(ksq, !result.side_to_move, dest_bb);
    //         }
    //     } else if castles {
    //         let my_backrank = self.side_to_move.to_my_backrank();
    //         let index = dest.get_file().to_index();
    //         let start = BitBoard::set(my_backrank, unsafe {
    //             *CASTLE_ROOK_START.get_unchecked(index)
    //         });
    //         let end = BitBoard::set(my_backrank, unsafe {
    //             *CASTLE_ROOK_END.get_unchecked(index)
    //         });
    //         result.xor(PieceType::Rook, start, self.side_to_move);
    //         result.xor(PieceType::Rook, end, self.side_to_move);
    //     }
    //     // now, lets see if we're in check or pinned
    //     let attackers = result.occupied_co(result.side_to_move)
    //         & ((get_bishop_rays(ksq)
    //             & (result.pieces(PieceType::Bishop) | result.pieces(PieceType::Queen)))
    //             | (get_rook_rays(ksq)
    //                 & (result.pieces(PieceType::Rook) | result.pieces(PieceType::Queen))));

    //     for sq in attackers {
    //         let between = between(sq, ksq) & result.occupied();
    //         if between == EMPTY_BITBOARD {
    //             result.checkers ^= BitBoard::from_square(sq);
    //         } else if between.popcnt() == 1 {
    //             result.pinned ^= between;
    //         }
    //     }

    //     result.side_to_move = !result.side_to_move;
    // }

    // fn update_pin_info(&mut self) {
    //     self.pinned = EMPTY_BITBOARD;
    //     self.checkers = EMPTY_BITBOARD;

    //     let ksq = (self.pieces(PieceType::King) & self.occupied_co(self.side_to_move)).to_square();

    //     let pinners = self.occupied_co(!self.side_to_move)
    //         & ((get_bishop_rays(ksq) & (self.pieces(PieceType::Bishop) | self.pieces(PieceType::Queen)))
    //             | (get_rook_rays(ksq) & (self.pieces(PieceType::Rook) | self.pieces(PieceType::Queen))));

    //     for sq in pinners {
    //         let between = between(sq, ksq) & self.occupied();
    //         if between == EMPTY_BITBOARD {
    //             self.checkers ^= BitBoard::from_square(sq);
    //         } else if between.popcnt() == 1 {
    //             self.pinned ^= between;
    //         }
    //     }

    //     self.checkers ^= get_knight_moves(ksq)
    //         & self.occupied_co(!self.side_to_move)
    //         & self.pieces(PieceType::Knight);

    //     self.checkers ^= get_pawn_attacks(
    //         ksq,
    //         self.side_to_move,
    //         self.occupied_co(!self.side_to_move) & self.pieces(PieceType::Pawn),
    //     );
    // }

    #[inline(always)]
    pub fn pinned(&self) -> &BitBoard {
        &self.pinned
    }

    #[inline(always)]
    pub fn checkers(&self) -> &BitBoard {
        &self.checkers
    }
}

// impl TryFrom<&BoardBuilder> for SubBoard {
//     type Error = ChessError;

//     fn try_from(fen: &BoardBuilder) -> Result<Self, Self::Error> {
//         let mut board = SubBoard::new_empty();

//         for sq in ALL_SQUARES.iter() {
//             if let Some((piece, color)) = fen[*sq] {
//                 board.xor(piece, BitBoard::from_square(*sq), color);
//             }
//         }

//         board.side_to_move = fen.get_side_to_move();

//         if let Some(ep) = fen.get_en_passant() {
//             board.side_to_move = !board.side_to_move;
//             board.set_ep(ep);
//             board.side_to_move = !board.side_to_move;
//         }

//         #[allow(deprecated)]
//         board.add_castle_rights(Color::White, fen.get_castle_rights(Color::White));
//         #[allow(deprecated)]
//         board.add_castle_rights(Color::Black, fen.get_castle_rights(Color::Black));

//         board.update_pin_info();

//         if board.is_sane() {
//             Ok(board)
//         } else {
//             Err(ChessError::InvalidBoard)
//         }
//     }
// }

// impl TryFrom<&mut BoardBuilder> for SubBoard {
//     type Error = ChessError;

//     fn try_from(fen: &mut BoardBuilder) -> Result<Self, Self::Error> {
//         (&*fen).try_into()
//     }
// }

// impl TryFrom<BoardBuilder> for SubBoard {
//     type Error = ChessError;

//     fn try_from(fen: BoardBuilder) -> Result<Self, Self::Error> {
//         (&fen).try_into()
//     }
// }

// impl FromStr for SubBoard {
//     type Err = ChessError;

//     fn from_str(value: &str) -> Result<Self, Self::Err> {
//         Ok(BoardBuilder::from_str(value)?.try_into()?)
//     }
// }

// impl Default for SubBoard {
//     #[inline(always)]
//     fn default() -> Self {
//         Self::from_str(STARTING_BOARD_FEN).unwrap()
//     }
// }
