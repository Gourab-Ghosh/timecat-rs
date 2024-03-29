use super::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

#[derive(Eq, Clone, Debug)]
pub struct SubBoard {
    _pieces: [BitBoard; NUM_PIECE_TYPES],
    _occupied_co: [BitBoard; NUM_COLORS],
    _occupied: BitBoard,
    _turn: Color,
    _castle_rights: [CastleRights; NUM_COLORS],
    _ep_square: Option<Square>,
    _pinned: BitBoard,
    _checkers: BitBoard,
    _transposition_key: u64,
    _halfmove_clock: u8,
    _fullmove_number: NumMoves,
    _white_material_score: Score,
    _black_material_score: Score,
}

impl PartialEq for SubBoard {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.transposition_key_components()
            .eq(&other.transposition_key_components())
    }
}

impl SubBoard {
    #[inline(always)]
    fn transposition_key_components(
        &self,
    ) -> (
        [BitBoard; 6],
        [BitBoard; 2],
        Color,
        [CastleRights; 2],
        Option<Square>,
    ) {
        (
            self._pieces,
            self._occupied_co,
            self.turn(),
            self._castle_rights,
            self.ep_square(),
        )
    }

    #[inline(always)]
    fn new_empty() -> Self {
        Self {
            _pieces: [BB_EMPTY; NUM_PIECE_TYPES],
            _occupied_co: [BB_EMPTY; NUM_COLORS],
            _occupied: BB_EMPTY,
            _turn: White,
            _castle_rights: [CastleRights::None; NUM_COLORS],
            _pinned: BB_EMPTY,
            _checkers: BB_EMPTY,
            _transposition_key: 0,
            _ep_square: None,
            _halfmove_clock: 0,
            _fullmove_number: 1,
            _white_material_score: 0,
            _black_material_score: 0,
        }
    }

    #[inline(always)]
    pub fn status(&self) -> BoardStatus {
        let moves = MoveGenerator::new_legal(self).len();
        match moves {
            0 => {
                if self.checkers().is_empty() {
                    BoardStatus::Stalemate
                } else {
                    BoardStatus::Checkmate
                }
            }
            _ => BoardStatus::Ongoing,
        }
    }

    #[inline(always)]
    pub fn occupied(&self) -> BitBoard {
        self._occupied
    }

    #[inline(always)]
    pub fn occupied_co(&self, color: Color) -> BitBoard {
        *get_item_unchecked!(self._occupied_co, color.to_index())
    }

    #[inline(always)]
    pub fn king_square(&self, color: Color) -> Square {
        (self.get_piece_mask(King) & self.occupied_co(color)).to_square()
    }

    #[inline(always)]
    pub fn get_piece_mask(&self, piece: PieceType) -> BitBoard {
        *get_item_unchecked!(self._pieces, piece.to_index())
    }

    #[inline(always)]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        *get_item_unchecked!(self._castle_rights, color.to_index())
    }

    #[inline(always)]
    fn add_castle_rights(&mut self, color: Color, add: CastleRights) {
        *get_item_unchecked_mut!(self._castle_rights, color.to_index()) =
            self.castle_rights(color).add(add);
    }

    #[inline(always)]
    fn remove_castle_rights(&mut self, color: Color, remove: CastleRights) {
        *get_item_unchecked_mut!(self._castle_rights, color.to_index()) =
            self.castle_rights(color).remove(remove);
    }

    #[inline(always)]
    pub fn turn(&self) -> Color {
        self._turn
    }

    #[inline(always)]
    pub fn my_castle_rights(&self) -> CastleRights {
        self.castle_rights(self.turn())
    }

    #[inline(always)]
    fn add_my_castle_rights(&mut self, add: CastleRights) {
        self.add_castle_rights(self.turn(), add);
    }

    #[inline(always)]
    fn remove_my_castle_rights(&mut self, remove: CastleRights) {
        self.remove_castle_rights(self.turn(), remove);
    }

    #[inline(always)]
    pub fn their_castle_rights(&self) -> CastleRights {
        self.castle_rights(!self.turn())
    }

    #[inline(always)]
    fn add_their_castle_rights(&mut self, add: CastleRights) {
        self.add_castle_rights(!self.turn(), add)
    }

    #[inline(always)]
    fn remove_their_castle_rights(&mut self, remove: CastleRights) {
        self.remove_castle_rights(!self.turn(), remove);
    }

    fn xor(&mut self, piece_type: PieceType, bb: BitBoard, color: Color) {
        *get_item_unchecked_mut!(self._pieces, piece_type.to_index()) ^= bb;
        let colored_piece_mask = get_item_unchecked_mut!(self._occupied_co, color.to_index());
        let colored_piece_mask_before = *colored_piece_mask;
        *colored_piece_mask ^= bb;
        self._occupied ^= bb;
        self._transposition_key ^= Zobrist::piece(piece_type, bb.to_square(), color);
        if piece_type != King {
            let score_change =
                if colored_piece_mask.get_mask() > colored_piece_mask_before.get_mask() {
                    piece_type.evaluate()
                } else {
                    -piece_type.evaluate()
                };
            match color {
                White => self._white_material_score += score_change,
                Black => self._black_material_score += score_change,
            }
        }
    }

    #[inline(always)]
    pub fn get_white_material_score(&self) -> Score {
        self._white_material_score
    }

    #[inline(always)]
    pub fn get_black_material_score(&self) -> Score {
        self._black_material_score
    }

    #[inline(always)]
    pub fn null_move(&self) -> Option<Self> {
        if !self.checkers().is_empty() {
            None
        } else {
            let mut result = self.to_owned();
            result._turn = !result.turn();
            result.remove_ep();
            result._halfmove_clock += 1;
            result._fullmove_number += 1;
            result.update_pin_and_checkers_info();
            Some(result)
        }
    }

    pub fn is_sane(&self) -> bool {
        // make sure there is no square with multiple pieces on it
        for x in ALL_PIECE_TYPES {
            for y in ALL_PIECE_TYPES {
                if x != y && !(self.get_piece_mask(x) & self.get_piece_mask(y)).is_empty() {
                    return false;
                }
            }
        }

        // make sure the colors don't overlap, either
        if !(self.occupied_co(White) & self.occupied_co(Black)).is_empty() {
            return false;
        }

        // grab all the pieces by OR'ing together each piece() BitBoard
        let occupied = ALL_PIECE_TYPES
            .iter()
            .fold(BB_EMPTY, |cur, &next| cur | self.get_piece_mask(next));

        // make sure that's equal to the occupied bitboard
        if occupied != self.occupied() {
            return false;
        }

        // make sure there is exactly one white king
        if (self.get_piece_mask(King) & self.occupied_co(White)).popcnt() != 1 {
            return false;
        }

        // make sure there is exactly one black king
        if (self.get_piece_mask(King) & self.occupied_co(Black)).popcnt() != 1 {
            return false;
        }

        // make sure the en passant square has a pawn on it of the right color
        match self.ep_square() {
            None => {}
            Some(x) => {
                let mut square_bb = x.to_bitboard();
                if self.turn() == White {
                    square_bb >>= 8;
                } else {
                    square_bb <<= 8;
                }
                if (self.get_piece_mask(Pawn) & self.occupied_co(!self.turn()) & square_bb)
                    .is_empty()
                {
                    return false;
                }
            }
        }

        // make sure my opponent is not currently in check (because that would be illegal)
        let mut board_copy = self.to_owned();
        board_copy._turn = !board_copy.turn();
        board_copy.update_pin_and_checkers_info();
        if !board_copy.checkers().is_empty() {
            return false;
        }

        // for each color, verify that, if they have castle rights, that they haven't moved their
        // rooks or king
        for color in ALL_COLORS {
            // get the castle rights
            let castle_rights = self.castle_rights(color);

            // the castle rights object will tell us which rooks shouldn't have moved yet.
            // verify there are rooks on all those squares
            if castle_rights.unmoved_rooks(color)
                & self.get_piece_mask(Rook)
                & self.occupied_co(color)
                != castle_rights.unmoved_rooks(color)
            {
                return false;
            }
            // if we have castle rights, make sure we have a king on the (E, {1,8}) square,
            // depending on the color
            if castle_rights != CastleRights::None
                && self.get_piece_mask(King) & self.occupied_co(color)
                    != get_file_bb(File::E) & get_rank_bb(color.to_my_backrank())
            {
                return false;
            }
        }

        // we must make sure the kings aren't touching
        if !(get_king_moves(self.king_square(White)) & self.get_piece_mask(King)).is_empty() {
            return false;
        }

        // it checks out
        true
    }

    #[inline(always)]
    pub fn get_hash(&self) -> u64 {
        self._transposition_key
            ^ if let Some(ep) = self.ep_square() {
                Zobrist::en_passant(ep.get_file(), !self.turn())
            } else {
                0
            }
            ^ Zobrist::castles(
                *get_item_unchecked!(self._castle_rights, self.turn().to_index()),
                self.turn(),
            )
            ^ Zobrist::castles(
                *get_item_unchecked!(self._castle_rights, (!self.turn()).to_index()),
                !self.turn(),
            )
            ^ if self.turn() == Black {
                Zobrist::color()
            } else {
                0
            }
    }

    #[inline(always)]
    pub fn get_pawn_hash(&self) -> u64 {
        todo!()
    }

    #[inline(always)]
    pub fn piece_type_at(&self, square: Square) -> Option<PieceType> {
        // TODO: check speed on Naive Algorithm
        let opp = square.to_bitboard();
        if (self.occupied() & opp).is_empty() {
            None
        } else {
            //naive algorithm
            /*
            for &p in ALL_PIECE_TYPES {
                if self.get_piece_mask(p) & opp {
                    return p;
                }
            } */
            if !((self.get_piece_mask(Pawn)
                ^ self.get_piece_mask(Knight)
                ^ self.get_piece_mask(Bishop))
                & opp)
                .is_empty()
            {
                if !(self.get_piece_mask(Pawn) & opp).is_empty() {
                    Some(Pawn)
                } else if !(self.get_piece_mask(Knight) & opp).is_empty() {
                    Some(Knight)
                } else {
                    Some(Bishop)
                }
            } else if !(self.get_piece_mask(Rook) & opp).is_empty() {
                Some(Rook)
            } else if !(self.get_piece_mask(Queen) & opp).is_empty() {
                Some(Queen)
            } else {
                Some(King)
            }
        }
    }

    #[inline(always)]
    pub fn color_at(&self, square: Square) -> Option<Color> {
        if !(self.occupied_co(White) & square.to_bitboard()).is_empty() {
            Some(White)
        } else if !(self.occupied_co(Black) & square.to_bitboard()).is_empty() {
            Some(Black)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        Some(Piece::new(
            self.piece_type_at(square)?,
            self.color_at(square)?,
        ))
    }

    fn remove_ep(&mut self) {
        self._ep_square = None;
    }

    #[inline(always)]
    pub fn ep_square(&self) -> Option<Square> {
        self._ep_square
    }

    #[inline(always)]
    pub fn get_halfmove_clock(&self) -> u8 {
        self._halfmove_clock
    }

    #[inline(always)]
    pub fn get_fullmove_number(&self) -> NumMoves {
        self._fullmove_number
    }

    fn set_ep(&mut self, square: Square) {
        // Only set self._ep_square if the pawn can actually be captured next move.
        let mut rank = square.get_rank();
        rank = if rank.to_int() > 3 {
            rank.down()
        } else {
            rank.up()
        };
        if !(get_adjacent_files(square.get_file())
            & get_rank_bb(rank)
            & self.get_piece_mask(Pawn)
            & self.occupied_co(!self.turn()))
        .is_empty()
        {
            self._ep_square = Some(square);
        }
    }

    #[inline(always)]
    pub fn legal(&self, move_: Move) -> bool {
        MoveGenerator::new_legal(self).contains(&move_)
    }

    pub fn make_move_new(&self, move_: Move) -> Self {
        let mut result = mem::MaybeUninit::<Self>::uninit();
        unsafe {
            self.make_move(move_, &mut *result.as_mut_ptr());
            result.assume_init()
        }
    }

    pub fn is_zeroing(&self, move_: Move) -> bool {
        let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
        !(touched & self.get_piece_mask(Pawn)).is_empty()
            || !(touched & self.occupied_co(!self.turn())).is_empty()
    }

    pub fn make_move(&self, move_: Move, result: &mut Self) {
        *result = self.to_owned();

        if result.is_zeroing(move_) {
            result._halfmove_clock = 0;
        } else {
            result._halfmove_clock += 1;
        }
        if result.turn() == Black {
            result._fullmove_number += 1;
        }

        result.remove_ep();
        result._checkers = BB_EMPTY;
        result._pinned = BB_EMPTY;
        let source = move_.get_source();
        let dest = move_.get_dest();

        let source_bb = source.to_bitboard();
        let dest_bb = dest.to_bitboard();
        let move_bb = source_bb ^ dest_bb;
        let moved = self.piece_type_at(source).unwrap();

        result.xor(moved, source_bb, self.turn());
        result.xor(moved, dest_bb, self.turn());
        if let Some(captured) = self.piece_type_at(dest) {
            result.xor(captured, dest_bb, !self.turn());
        }

        result
            .remove_their_castle_rights(CastleRights::square_to_castle_rights(!self.turn(), dest));

        result.remove_my_castle_rights(CastleRights::square_to_castle_rights(self.turn(), source));

        let opp_king = result.get_piece_mask(King) & result.occupied_co(!result.turn());

        let castles = moved == King && (move_bb & get_castle_moves()) == move_bb;

        let ksq = opp_king.to_square();

        const CASTLE_ROOK_START: [File; 8] = [
            File::A,
            File::A,
            File::A,
            File::A,
            File::H,
            File::H,
            File::H,
            File::H,
        ];
        const CASTLE_ROOK_END: [File; 8] = [
            File::D,
            File::D,
            File::D,
            File::D,
            File::F,
            File::F,
            File::F,
            File::F,
        ];

        if moved == Knight {
            result._checkers ^= get_knight_moves(ksq) & dest_bb;
        } else if moved == Pawn {
            if let Some(Knight) = move_.get_promotion() {
                result.xor(Pawn, dest_bb, self.turn());
                result.xor(Knight, dest_bb, self.turn());
                result._checkers ^= get_knight_moves(ksq) & dest_bb;
            } else if let Some(promotion) = move_.get_promotion() {
                result.xor(Pawn, dest_bb, self.turn());
                result.xor(promotion, dest_bb, self.turn());
            } else if !(source_bb & get_pawn_source_double_moves()).is_empty()
                && !(dest_bb & get_pawn_dest_double_moves()).is_empty()
            {
                result.set_ep(dest.wrapping_backward(result.turn()));
                result._checkers ^= get_pawn_attacks(ksq, !result.turn(), dest_bb);
            } else if Some(dest) == self.ep_square() {
                result.xor(
                    Pawn,
                    dest.wrapping_backward(self.turn()).to_bitboard(),
                    !self.turn(),
                );
                result._checkers ^= get_pawn_attacks(ksq, !result.turn(), dest_bb);
            } else {
                result._checkers ^= get_pawn_attacks(ksq, !result.turn(), dest_bb);
            }
        } else if castles {
            let my_backrank = self.turn().to_my_backrank();
            let index = dest.get_file().to_index();
            let start = BitBoard::from_rank_and_file(
                my_backrank,
                *get_item_unchecked!(CASTLE_ROOK_START, index),
            );
            let end = BitBoard::from_rank_and_file(
                my_backrank,
                *get_item_unchecked!(CASTLE_ROOK_END, index),
            );
            result.xor(Rook, start, self.turn());
            result.xor(Rook, end, self.turn());
        }
        // now, lets see if we're in check or pinned
        let attackers = result.occupied_co(result.turn())
            & ((get_bishop_rays(ksq)
                & (result.get_piece_mask(Bishop) | result.get_piece_mask(Queen)))
                | (get_rook_rays(ksq)
                    & (result.get_piece_mask(Rook) | result.get_piece_mask(Queen))));

        for square in attackers {
            let between = between(square, ksq) & result.occupied();
            if between.is_empty() {
                result._checkers ^= square.to_bitboard();
            } else if between.popcnt() == 1 {
                result._pinned ^= between;
            }
        }

        result._turn = !result.turn();
    }

    pub fn flip_vertical(&mut self) {
        // TODO: Change Transposition Key
        self._pieces
            .iter_mut()
            .for_each(|bb| *bb = bb.flip_vertical());
        self._occupied_co
            .iter_mut()
            .for_each(|bb| *bb = bb.flip_vertical());
        self._occupied = self._occupied.flip_vertical();
        self._turn = !self._turn;
        self._castle_rights = [CastleRights::None; NUM_COLORS];
        self.update_pin_and_checkers_info();
        // self._transposition_key = self._transposition_key;
        self._ep_square = self._ep_square.map(|square| square.mirror());
    }

    fn update_pin_and_checkers_info(&mut self) {
        self._pinned = BB_EMPTY;
        self._checkers = BB_EMPTY;

        let ksq = (self.get_piece_mask(King) & self.occupied_co(self.turn())).to_square();

        let pinners = self.occupied_co(!self.turn())
            & ((get_bishop_rays(ksq) & (self.get_piece_mask(Bishop) | self.get_piece_mask(Queen)))
                | (get_rook_rays(ksq) & (self.get_piece_mask(Rook) | self.get_piece_mask(Queen))));

        for square in pinners {
            let between = between(square, ksq) & self.occupied();
            if between.is_empty() {
                self._checkers ^= square.to_bitboard();
            } else if between.popcnt() == 1 {
                self._pinned ^= between;
            }
        }

        self._checkers ^=
            get_knight_moves(ksq) & self.occupied_co(!self.turn()) & self.get_piece_mask(Knight);

        self._checkers ^= get_pawn_attacks(
            ksq,
            self.turn(),
            self.occupied_co(!self.turn()) & self.get_piece_mask(Pawn),
        );
    }

    #[inline(always)]
    pub fn pinned(&self) -> BitBoard {
        self._pinned
    }

    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self._checkers
    }

    #[inline(always)]
    pub fn is_check(&self) -> bool {
        !self._checkers.is_empty()
    }
}

impl TryFrom<&BoardBuilder> for SubBoard {
    type Error = EngineError;

    fn try_from(board_builder: &BoardBuilder) -> Result<Self, Self::Error> {
        let mut board = SubBoard::new_empty();

        for square in ALL_SQUARES {
            if let Some(piece) = board_builder[square] {
                board.xor(
                    piece.get_piece_type(),
                    square.to_bitboard(),
                    piece.get_color(),
                );
            }
        }

        board._turn = board_builder.get_turn();

        if let Some(ep) = board_builder.get_en_passant() {
            board._turn = !board.turn();
            board.set_ep(ep);
            board._turn = !board.turn();
        }

        board.add_castle_rights(White, board_builder.get_castle_rights(White));
        board.add_castle_rights(Black, board_builder.get_castle_rights(Black));

        board._halfmove_clock = board_builder.get_halfmove_clock();
        board._fullmove_number = board_builder.get_fullmove_number();

        board.update_pin_and_checkers_info();

        if board.is_sane() {
            Ok(board)
        } else {
            Err(EngineError::InvalidSubBoard { board })
        }
    }
}

impl TryFrom<BoardBuilder> for SubBoard {
    type Error = EngineError;

    fn try_from(board_builder: BoardBuilder) -> Result<Self, Self::Error> {
        (&board_builder).try_into()
    }
}

impl TryFrom<&mut BoardBuilder> for SubBoard {
    type Error = EngineError;

    fn try_from(board_builder: &mut BoardBuilder) -> Result<Self, Self::Error> {
        (board_builder.to_owned()).try_into()
    }
}

impl FromStr for SubBoard {
    type Err = EngineError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        BoardBuilder::from_str(value)?.try_into()
    }
}

impl Default for SubBoard {
    #[inline(always)]
    fn default() -> Self {
        Self::from_str(STARTING_POSITION_FEN).unwrap()
    }
}

impl fmt::Display for SubBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fen: BoardBuilder = self.into();
        write!(f, "{}", fen)
    }
}
