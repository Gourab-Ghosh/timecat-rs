use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoardStatus {
    Ongoing,
    Stalemate,
    Checkmate,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq)]
pub struct BoardPosition {
    _piece_masks: [BitBoard; NUM_PIECE_TYPES],
    _occupied_color: [BitBoard; NUM_COLORS],
    _occupied: BitBoard,
    _turn: Color,
    _castle_rights: [CastleRights; NUM_COLORS],
    _ep_square: Option<Square>,
    _pinned: BitBoard,
    _checkers: BitBoard,
    _pawn_transposition_hash: u64,
    _non_pawn_transposition_hash: u64,
    // _transposition_hash: u64,
    _halfmove_clock: u8,
    _fullmove_number: NumMoves,
    _material_scores: [Score; 2],
}

impl UniqueIdentifier for BoardPosition {
    fn unique_identifier(&self) -> impl PartialEq + Hash {
        (
            self._piece_masks,
            self._occupied_color,
            self._castle_rights,
            self.turn(),
            self.ep_square(),
        )
    }
}

impl PartialEq for BoardPosition {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.unique_identifier()
            .eq(&other.unique_identifier())
    }
}

impl BoardPosition {
    #[inline]
    fn new_empty() -> Self {
        Self {
            _piece_masks: [BitBoard::EMPTY; NUM_PIECE_TYPES],
            _occupied_color: [BitBoard::EMPTY; NUM_COLORS],
            _occupied: BitBoard::EMPTY,
            _turn: White,
            _castle_rights: [CastleRights::None; NUM_COLORS],
            _pinned: BitBoard::EMPTY,
            _checkers: BitBoard::EMPTY,
            _pawn_transposition_hash: 0,
            _non_pawn_transposition_hash: 0,
            // _transposition_hash: 0,
            _ep_square: None,
            _halfmove_clock: 0,
            _fullmove_number: 1,
            _material_scores: [0; 2],
        }
    }

    #[inline]
    pub fn has_legal_moves(&self) -> bool {
        MoveGenerator::has_legal_moves(self)
    }

    #[inline]
    pub fn status(&self) -> BoardStatus {
        if self.has_legal_moves() {
            BoardStatus::Ongoing
        } else if self.get_checkers() == BitBoard::EMPTY {
            BoardStatus::Stalemate
        } else {
            BoardStatus::Checkmate
        }
    }

    #[inline]
    pub fn occupied(&self) -> BitBoard {
        self._occupied
    }

    #[inline]
    pub fn get_num_pieces(&self) -> u32 {
        self.occupied().popcnt()
    }

    #[inline]
    pub fn occupied_color(&self, color: Color) -> BitBoard {
        *get_item_unchecked!(self._occupied_color, color.to_index())
    }

    #[inline]
    pub fn self_occupied(&self) -> BitBoard {
        self.occupied_color(self.turn())
    }

    #[inline]
    pub fn opponent_occupied(&self) -> BitBoard {
        self.occupied_color(!self.turn())
    }

    #[inline]
    pub fn get_black_occupied(&self) -> BitBoard {
        self.occupied_color(Black)
    }

    #[inline]
    pub fn get_white_occupied(&self) -> BitBoard {
        self.occupied_color(White)
    }

    #[inline]
    pub fn get_king_square(&self, color: Color) -> Square {
        self.get_colored_piece_mask(King, color)
            .to_square_unchecked()
    }

    #[inline]
    pub fn get_all_piece_masks(&self) -> &[BitBoard] {
        &self._piece_masks
    }

    #[inline]
    pub fn get_piece_mask(&self, piece_type: PieceType) -> BitBoard {
        *get_item_unchecked!(self._piece_masks, piece_type.to_index())
    }

    #[inline]
    pub fn get_colored_piece_mask(&self, piece_type: PieceType, color: Color) -> BitBoard {
        self.get_piece_mask(piece_type) & self.occupied_color(color)
    }

    pub fn has_insufficient_material(&self, color: Color) -> bool {
        let occupied = self.occupied_color(color);
        match occupied.popcnt() {
            1 => true,
            2 => ((self.get_piece_mask(Rook)
                ^ self.get_piece_mask(Queen)
                ^ self.get_piece_mask(Pawn))
                & occupied)
                .is_empty(),
            _ => false,
        }
    }

    #[inline]
    pub fn has_non_pawn_material(&self) -> bool {
        (self.get_piece_mask(Pawn) ^ self.get_piece_mask(King)) != self.occupied()
    }

    #[inline]
    pub fn get_non_king_pieces_mask(&self) -> BitBoard {
        self.occupied() ^ self.get_piece_mask(King)
    }

    pub fn has_only_same_colored_bishop(&self) -> bool {
        let non_king_pieces_mask = self.get_non_king_pieces_mask();
        if non_king_pieces_mask.popcnt() > 32 {
            return false;
        }
        let bishop_bitboard = self.get_piece_mask(Bishop);
        if non_king_pieces_mask != bishop_bitboard {
            return false;
        }
        non_king_pieces_mask & BB_LIGHT_SQUARES == bishop_bitboard
            || non_king_pieces_mask & BB_DARK_SQUARES == bishop_bitboard
    }

    #[inline]
    pub fn is_insufficient_material(&self) -> bool {
        match self.occupied().popcnt() {
            2 => true,
            3 => const { [Pawn, Rook, Queen] }
                .into_iter()
                .all(|piece| self.get_piece_mask(piece).is_empty()),
            _ => self.has_only_same_colored_bishop(),
        }
    }

    #[inline]
    pub fn castle_rights(&self, color: Color) -> CastleRights {
        *get_item_unchecked!(self._castle_rights, color.to_index())
    }

    pub fn clean_castling_rights(&self) -> BitBoard {
        let white_castling_rights = get_item_unchecked!(
            const
            [
                BitBoard::EMPTY,
                BB_H1,
                BB_A1,
                BitBoard::new(BB_A1.into_inner() ^ BB_H1.into_inner()),
            ],
            self.castle_rights(White).to_index(),
        );
        let black_castling_rights = get_item_unchecked!(
            const
            [
                BitBoard::EMPTY,
                BB_H8,
                BB_A8,
                BitBoard::new(BB_A8.into_inner() ^ BB_H8.into_inner()),
            ],
            self.castle_rights(Black).to_index(),
        );
        white_castling_rights ^ black_castling_rights
    }

    #[inline]
    fn add_castle_rights(&mut self, color: Color, add: CastleRights) {
        *get_item_unchecked_mut!(self._castle_rights, color.to_index()) =
            self.castle_rights(color).add(add);
    }

    #[inline]
    fn remove_castle_rights(&mut self, color: Color, remove: CastleRights) {
        *get_item_unchecked_mut!(self._castle_rights, color.to_index()) =
            self.castle_rights(color).remove(remove);
    }

    #[inline]
    pub fn turn(&self) -> Color {
        self._turn
    }

    #[inline]
    pub fn my_castle_rights(&self) -> CastleRights {
        self.castle_rights(self.turn())
    }

    #[inline]
    fn add_my_castle_rights(&mut self, add: CastleRights) {
        self.add_castle_rights(self.turn(), add);
    }

    #[inline]
    fn remove_my_castle_rights(&mut self, remove: CastleRights) {
        self.remove_castle_rights(self.turn(), remove);
    }

    #[inline]
    pub fn their_castle_rights(&self) -> CastleRights {
        self.castle_rights(!self.turn())
    }

    #[inline]
    fn add_their_castle_rights(&mut self, add: CastleRights) {
        self.add_castle_rights(!self.turn(), add)
    }

    #[inline]
    fn remove_their_castle_rights(&mut self, remove: CastleRights) {
        self.remove_castle_rights(!self.turn(), remove);
    }

    // fn update_transposition_hash(&mut self) {
    //     self._transposition_hash = self.get_pawn_hash()
    //         ^ self.get_non_pawn_hash()
    //         ^ Zobrist::castle(self.castle_rights(self.turn()), self.turn())
    //         ^ Zobrist::castle(self.castle_rights(!self.turn()), !self.turn());
    //     if let Some(ep) = self.ep_square() {
    //         self._transposition_hash ^= Zobrist::en_passant(ep.get_file());
    //     }
    //     if self.turn() == Black {
    //         self._transposition_hash ^= Zobrist::color();
    //     }
    // }

    fn xor(&mut self, piece_type: PieceType, bb: BitBoard, color: Color) {
        *get_item_unchecked_mut!(self._piece_masks, piece_type.to_index()) ^= bb;
        let colored_piece_mask = get_item_unchecked_mut!(self._occupied_color, color.to_index());
        let colored_piece_mask_before = *colored_piece_mask;
        *colored_piece_mask ^= bb;
        self._occupied ^= bb;
        if piece_type == Pawn {
            self._pawn_transposition_hash ^=
                Zobrist::piece(piece_type, bb.to_square_unchecked(), color);
        } else {
            self._non_pawn_transposition_hash ^=
                Zobrist::piece(piece_type, bb.to_square_unchecked(), color);
        }
        if piece_type != King {
            *get_item_unchecked_mut!(self._material_scores, color.to_index()) +=
                if *colored_piece_mask > colored_piece_mask_before {
                    piece_type.evaluate()
                } else {
                    -piece_type.evaluate()
                };
        }
    }

    #[inline]
    pub fn get_pawn_hash(&self) -> u64 {
        self._pawn_transposition_hash
    }

    #[inline]
    pub fn get_non_pawn_hash(&self) -> u64 {
        self._non_pawn_transposition_hash
    }

    /// The hash function is defined according to the polyglot hash function.
    #[inline]
    pub fn get_hash(&self) -> u64 {
        self.get_pawn_hash()
            ^ self.get_non_pawn_hash()
            ^ Zobrist::castle(self.castle_rights(White), self.castle_rights(Black))
            ^ self
                .ep_square()
                .map_or(0, |ep| Zobrist::en_passant(ep.get_file()))
            ^ Zobrist::color(self.turn())
    }

    #[inline]
    pub fn get_white_material_score(&self) -> Score {
        *get_item_unchecked!(self._material_scores, 0)
    }

    #[inline]
    pub fn get_black_material_score(&self) -> Score {
        *get_item_unchecked!(self._material_scores, 1)
    }

    #[inline]
    pub fn gives_check(&self, move_: Move) -> bool {
        // TODO: Scope for improvement ig?
        self.make_move_new(move_).is_check()
    }

    #[inline]
    pub fn gives_checkmate(&self, move_: Move) -> bool {
        self.make_move_new(move_).status() == BoardStatus::Checkmate
    }

    pub fn null_move_unchecked(&self) -> Self {
        let mut result = self.to_owned();
        result.flip_turn_unchecked();
        result.remove_ep();
        result._halfmove_clock += 1;
        result._fullmove_number += 1;
        result.update_pin_and_checkers_info();
        result
    }

    #[inline]
    pub fn null_move(&self) -> Result<Self> {
        if self.get_checkers().is_empty() {
            Ok(self.null_move_unchecked())
        } else {
            Err(TimecatError::NullMoveInCheck {
                fen: self.get_fen(),
            })
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
        if !(self.occupied_color(White) & self.occupied_color(Black)).is_empty() {
            return false;
        }

        // grab all the pieces by OR'ing together each piece() BitBoard
        let occupied = ALL_PIECE_TYPES.iter().fold(BitBoard::EMPTY, |cur, &next| {
            cur | self.get_piece_mask(next)
        });

        // make sure that's equal to the occupied bitboard
        if occupied != self.occupied() {
            return false;
        }

        // make sure there is exactly one white king
        if self.get_colored_piece_mask(King, White).popcnt() != 1 {
            return false;
        }

        // make sure there is exactly one black king
        if self.get_colored_piece_mask(King, Black).popcnt() != 1 {
            return false;
        }

        // make sure the en passant square has a pawn on it of the right color
        if let Some(x) = self.ep_square() {
            let mut square_bb = x.to_bitboard();
            if self.turn() == White {
                square_bb >>= 8;
            } else {
                square_bb <<= 8;
            }
            if (self.get_colored_piece_mask(Pawn, !self.turn()) & square_bb).is_empty() {
                return false;
            }
        }

        // make sure my opponent is not currently in check (because that would be illegal)
        let mut board_copy = self.to_owned();
        board_copy.flip_turn_unchecked();
        board_copy.update_pin_and_checkers_info();
        if !board_copy.get_checkers().is_empty() {
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
                & self.occupied_color(color)
                != castle_rights.unmoved_rooks(color)
            {
                return false;
            }
            // if we have castle rights, make sure we have a king on the (E, {1,8}) square,
            // depending on the color
            if castle_rights != CastleRights::None
                && self.get_colored_piece_mask(King, color)
                    != BB_FILE_E & color.to_my_backrank().to_bitboard()
            {
                return false;
            }
        }

        // we must make sure the kings aren't touching
        if !(self.get_king_square(White).get_king_moves() & self.get_piece_mask(King)).is_empty() {
            return false;
        }

        // it checks out
        true
    }

    #[inline]
    pub fn get_piece_type_at(&self, square: Square) -> Option<PieceType> {
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

    #[inline]
    pub fn color_at(&self, square: Square) -> Option<Color> {
        if !(self.occupied_color(White) & square.to_bitboard()).is_empty() {
            Some(White)
        } else if !(self.occupied_color(Black) & square.to_bitboard()).is_empty() {
            Some(Black)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
        Some(Piece::new(
            self.get_piece_type_at(square)?,
            self.color_at(square)?,
        ))
    }

    fn remove_ep(&mut self) {
        self._ep_square = None;
    }

    #[inline]
    pub fn ep_square(&self) -> Option<Square> {
        self._ep_square
    }

    #[inline]
    pub fn get_halfmove_clock(&self) -> u8 {
        self._halfmove_clock
    }

    #[inline]
    pub fn get_fullmove_number(&self) -> NumMoves {
        self._fullmove_number
    }

    #[inline]
    pub fn get_fen(&self) -> String {
        self.to_string()
    }

    #[inline]
    pub fn from_fen(fen: &str) -> Result<Self> {
        Self::from_str(fen)
    }

    pub fn is_good_fen(fen: &str) -> bool {
        let fen = simplify_fen(fen);
        let mut splitted_fen = fen.split(' ');
        if splitted_fen.nth(4).unwrap_or("0").parse().unwrap_or(-1) < 0
            || splitted_fen.next().unwrap_or("1").parse().unwrap_or(-1) < 0
            || splitted_fen.next().is_some()
        {
            return false;
        };
        Self::from_str(&fen).is_ok()
    }

    pub fn set_fen(&mut self, fen: &str) -> Result<()> {
        let fen = simplify_fen(fen);
        if fen == self.get_fen() {
            return Ok(());
        }
        *self = Self::from_str(&fen)?;
        Ok(())
    }

    #[inline]
    pub fn is_en_passant(&self, move_: Move) -> bool {
        self.ep_square().map_or(false, |ep_square| {
            let source = move_.get_source();
            let dest = move_.get_dest();
            ep_square == dest
                && self.get_piece_mask(Pawn).contains(source)
                && [7, 9].contains(&dest.to_int().abs_diff(source.to_int()))
                && !self.occupied().contains(dest)
        })
    }

    pub fn is_passed_pawn(&self, square: Square) -> bool {
        // TODO:: Scope for improvement
        let pawn_mask = self.get_piece_mask(Pawn);
        let Some(self_color) = self.color_at(square) else {
            return false;
        };
        if !(pawn_mask & self.occupied_color(self_color)).contains(square) {
            return false;
        }
        let file = square.get_file();
        (pawn_mask
            & self.occupied_color(!self_color)
            & (file.get_adjacent_files_bb() ^ file.to_bitboard())
            & square.get_rank().get_upper_board_mask(self_color))
        .is_empty()
    }

    pub fn is_capture(&self, move_: Move) -> bool {
        let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
        !(touched & self.opponent_occupied()).is_empty() || self.is_en_passant(move_)
    }

    fn set_ep(&mut self, square: Square) {
        // Only set self._ep_square if the pawn can actually be captured next move.
        let mut rank = square.get_rank();
        rank = if rank.to_int() > 3 {
            rank.wrapping_down()
        } else {
            rank.wrapping_up()
        };
        if !(square.get_file().get_adjacent_files_bb()
            & rank.to_bitboard()
            & self.get_piece_mask(Pawn)
            & self.opponent_occupied())
        .is_empty()
        {
            self._ep_square = Some(square);
        }
    }

    #[inline]
    pub fn generate_legal_moves(&self) -> MoveGenerator {
        MoveGenerator::new_legal(self)
    }

    pub fn generate_masked_legal_moves(
        &self,
        from_bitboard: BitBoard,
        to_bitboard: BitBoard,
    ) -> MoveGenerator {
        // TODO: Scope of improvement
        let mut moves = self.generate_legal_moves();
        moves.set_iterator_masks(from_bitboard, to_bitboard);
        moves
    }

    pub fn generate_legal_captures(&self) -> MoveGenerator {
        let mut targets = self.opponent_occupied();
        if let Some(ep_square) = self.ep_square() {
            targets ^= ep_square.to_bitboard()
        }
        let mut moves = self.generate_legal_moves();
        moves.set_to_bitboard_iterator_mask(targets);
        moves
    }

    #[inline]
    pub fn is_legal(&self, move_: &Move) -> bool {
        MoveGenerator::is_legal(self, move_)
    }

    pub fn is_castling(&self, move_: Move) -> bool {
        if !self.get_piece_mask(King).contains(move_.get_source()) {
            return false;
        }
        let rank_diff = move_
            .get_source()
            .get_file()
            .to_index()
            .abs_diff(move_.get_dest().get_file().to_index());
        rank_diff > 1
            || self
                .get_colored_piece_mask(Rook, self.turn())
                .contains(move_.get_dest())
    }

    pub fn is_zeroing(&self, move_: Move) -> bool {
        let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
        !(touched & self.get_piece_mask(Pawn)).is_empty()
            || !(touched & self.opponent_occupied()).is_empty()
    }

    pub fn flip_vertical(&mut self) {
        // TODO: Change Transposition Keys
        self._piece_masks
            .iter_mut()
            .chain(self._occupied_color.iter_mut())
            .for_each(|bb| *bb = bb.flip_vertical());
        self._occupied = self._occupied.flip_vertical();
        self._castle_rights = [CastleRights::None; NUM_COLORS];
        self.update_pin_and_checkers_info();
        // self._non_pawn_transposition_hash = self._non_pawn_transposition_hash;
        // self._pawn_transposition_hash = self._pawn_transposition_hash;
        self._ep_square = self._ep_square.map(|square| square.horizontal_mirror());
    }

    pub fn flip_horizontal(&mut self) {
        // TODO: Change Transposition Keys
        self._piece_masks
            .iter_mut()
            .chain(self._occupied_color.iter_mut())
            .for_each(|bb| *bb = bb.flip_horizontal());
        self._occupied = self._occupied.flip_horizontal();
        self._castle_rights = [CastleRights::None; NUM_COLORS];
        self.update_pin_and_checkers_info();
        // self._non_pawn_transposition_hash = self._non_pawn_transposition_hash;
        // self._pawn_transposition_hash = self._pawn_transposition_hash;
        self._ep_square = self._ep_square.map(|square| square.vertical_mirror());
    }

    #[inline]
    pub fn set_turn_unchecked(&mut self, turn: Color) {
        self._turn = turn;
    }

    #[inline]
    pub fn flip_turn_unchecked(&mut self) {
        self._turn = !self._turn;
    }

    pub fn flip_vertical_and_flip_turn_unchecked(&mut self) {
        self.flip_vertical();
        self.flip_turn_unchecked();
    }

    fn update_pin_and_checkers_info(&mut self) {
        self._pinned = BitBoard::EMPTY;
        self._checkers = BitBoard::EMPTY;

        let ksq = self.get_king_square(self.turn());
        let pinners = self.opponent_occupied()
            & ((ksq.get_bishop_rays_bb()
                & (self.get_piece_mask(Bishop) | self.get_piece_mask(Queen)))
                | (ksq.get_rook_rays_bb()
                    & (self.get_piece_mask(Rook) | self.get_piece_mask(Queen))));

        for square in pinners {
            let between = square.between(ksq) & self.occupied();
            if between.is_empty() {
                self._checkers ^= square.to_bitboard();
            } else if between.popcnt() == 1 {
                self._pinned ^= between;
            }
        }

        self._checkers ^=
            ksq.get_knight_moves() & self.opponent_occupied() & self.get_piece_mask(Knight);

        self._checkers ^= ksq.get_pawn_attacks(
            self.turn(),
            self.opponent_occupied() & self.get_piece_mask(Pawn),
        );
    }

    #[inline]
    pub fn pinned(&self) -> BitBoard {
        self._pinned
    }

    #[inline]
    pub fn get_checkers(&self) -> BitBoard {
        self._checkers
    }

    pub fn get_custom_attacked_squares_bb<'a>(
        &'a self,
        attacker_piece_types: &'a [PieceType],
        colors: &'a [Color],
        attackers_mask: BitBoard,
    ) -> BitBoard {
        let mut attacked_squares = BitBoard::EMPTY;
        for (piece, square) in self.custom_iter(attacker_piece_types, colors, attackers_mask) {
            attacked_squares |= match piece.get_piece_type() {
                Pawn => square.get_pawn_attacks(piece.get_color(), BB_ALL),
                Knight => square.get_knight_moves(),
                Bishop => get_bishop_moves(square, self.occupied()),
                Rook => get_rook_moves(square, self.occupied()),
                Queen => get_queen_moves(square, self.occupied()),
                King => square.get_king_moves(),
            };
        }
        attacked_squares & !self.self_occupied()
    }

    #[inline]
    pub fn get_attacked_squares_bb(&self) -> BitBoard {
        self.get_custom_attacked_squares_bb(&ALL_PIECE_TYPES, &ALL_COLORS, BB_ALL)
    }

    pub fn get_attackers_mask(
        &self,
        target_square: Square,
        color: impl Into<Option<Color>>,
    ) -> BitBoard {
        let color = color.into();
        let occupied = self.occupied();

        let queens_and_bishops = self.get_piece_mask(Bishop) ^ self.get_piece_mask(Queen);
        let queens_and_rooks = self.get_piece_mask(Rook) ^ self.get_piece_mask(Queen);

        let pawn_attacks = color.map_or(
            target_square.get_pawn_attacks(White, BB_ALL)
                ^ target_square.get_pawn_attacks(Black, BB_ALL),
            |color| target_square.get_pawn_attacks(!color, BB_ALL),
        ) & self.get_piece_mask(Pawn);

        // TODO: Scope for improvement?
        let sliding_attackers = (get_bishop_moves(target_square, occupied) & queens_and_bishops)
            | (get_rook_moves(target_square, occupied) & queens_and_rooks);
        let non_sliding_attackers = pawn_attacks
            ^ (target_square.get_knight_moves() & self.get_piece_mask(Knight))
            ^ (target_square.get_king_moves() & self.get_piece_mask(King));

        let attackers = sliding_attackers ^ non_sliding_attackers;

        color.map_or(attackers, |color| attackers & self.occupied_color(color))
    }

    #[inline]
    pub fn is_attacked_by(&self, square: Square, color: Color) -> bool {
        !self.get_attackers_mask(square, color).is_empty()
    }

    pub fn get_attackers_mask_by_piece_type(
        &self,
        target_square: Square,
        piece_type: PieceType,
        color: impl Into<Option<Color>>,
    ) -> BitBoard {
        let occupied = self.occupied();
        let color = color.into();

        let attackers = match piece_type {
            Pawn => match color {
                Some(color) => target_square.get_pawn_attacks(!color, BB_ALL),
                None => {
                    target_square.get_pawn_attacks(White, BB_ALL)
                        ^ target_square.get_pawn_attacks(Black, BB_ALL)
                }
            },
            Knight => target_square.get_knight_moves(),
            Bishop => get_bishop_moves(target_square, occupied),
            Rook => get_rook_moves(target_square, occupied),
            Queen => get_queen_moves(target_square, occupied),
            King => target_square.get_king_moves(),
        } & self.get_piece_mask(piece_type);

        color.map_or(attackers, |color| attackers & self.occupied_color(color))
    }

    pub fn get_least_attackers_square(
        &self,
        target_square: Square,
        color: impl Into<Option<Color>>,
    ) -> Option<Square> {
        let color = color.into();
        ALL_PIECE_TYPES
            .into_iter()
            .filter_map(|piece_type| {
                self.get_attackers_mask_by_piece_type(target_square, piece_type, color)
                    .to_square()
            })
            .next()
    }

    #[inline]
    pub fn is_check(&self) -> bool {
        !self._checkers.is_empty()
    }

    #[inline]
    pub fn is_checkmate(&self) -> bool {
        self.status() == BoardStatus::Checkmate
    }

    #[inline]
    pub fn piece_symbol_at(&self, square: Square) -> String {
        self.get_piece_at(square)
            .map_or_else(|| EMPTY_SPACE_SYMBOL.to_string(), |piece| piece.to_string())
    }

    pub fn piece_unicode_symbol_at(&self, square: Square, flip_color: bool) -> String {
        if let Some(piece) = self.get_piece_at(square) {
            let piece_index = piece.get_piece_type().to_index();
            let (white_pieces, black_pieces) = match flip_color {
                true => (BLACK_PIECE_UNICODE_SYMBOLS, WHITE_PIECE_UNICODE_SYMBOLS),
                false => (WHITE_PIECE_UNICODE_SYMBOLS, BLACK_PIECE_UNICODE_SYMBOLS),
            };
            return match piece.get_color() {
                White => get_item_unchecked!(white_pieces, piece_index),
                Black => get_item_unchecked!(black_pieces, piece_index),
            }
            .to_string();
        }
        EMPTY_SPACE_UNICODE_SYMBOL.to_string()
    }

    pub fn to_board_string(&self, last_move: ValidOrNullMove, use_unicode: bool) -> String {
        let mut skeleton = get_board_skeleton();
        let checkers = self.get_checkers();
        let king_square = self.get_king_square(self.turn());
        for square in SQUARES_HORIZONTAL_MIRROR {
            let symbol = if use_unicode {
                self.piece_unicode_symbol_at(square, false)
            } else {
                self.piece_symbol_at(square)
            };
            let mut styles = vec![];
            if symbol != " " {
                styles.extend_from_slice(match self.color_at(square).unwrap() {
                    White => WHITE_PIECES_STYLE,
                    Black => BLACK_PIECES_STYLE,
                });
                if square == king_square && !checkers.is_empty() {
                    styles.extend_from_slice(CHECK_STYLE);
                }
            }
            if [last_move.get_source(), last_move.get_dest()].contains(&Some(square)) {
                styles.extend_from_slice(LAST_MOVE_HIGHLIGHT_STYLE);
            }
            styles.dedup();
            skeleton = skeleton.replacen('O', &symbol.colorize(&styles), 1);
        }
        skeleton.push('\n');
        skeleton.push_str(
            &[
                String::new(),
                format_info("Fen", self.get_fen(), true),
                format_info("Transposition Key", self.get_hash().stringify_hash(), true),
                format_info(
                    "Checkers",
                    checkers.into_iter().join(" ").colorize(CHECKERS_STYLE),
                    true,
                ),
            ]
            .join("\n"),
        );
        #[cfg(feature = "inbuilt_nnue")]
        skeleton.push_str(&format!(
            "\n{}",
            format_info("Current Evaluation", self.slow_evaluate().stringify(), true)
        ));
        skeleton
    }

    #[inline]
    pub fn to_unicode_string(&self, last_move: ValidOrNullMove) -> String {
        self.to_board_string(last_move, true)
    }

    #[inline]
    fn is_halfmoves(&self, n: u8) -> bool {
        self.get_halfmove_clock() >= n
    }

    #[inline]
    pub fn is_fifty_moves(&self) -> bool {
        self.is_halfmoves(100)
    }

    #[inline]
    pub fn is_stalemate(&self) -> bool {
        self.status() == BoardStatus::Stalemate
    }

    pub fn is_double_pawn_push(&self, move_: Move) -> bool {
        let source = move_.get_source();
        let dest = move_.get_dest();
        source.get_rank() == self.turn().to_second_rank()
            && source
                .get_rank()
                .to_int()
                .abs_diff(dest.get_rank().to_int())
                == 2
            && !self.get_piece_mask(Pawn).contains(source)
    }

    #[inline]
    pub fn is_quiet(&self, move_: Move) -> bool {
        !(self.is_capture(move_) || self.gives_check(move_))
    }

    #[inline]
    pub fn has_legal_en_passant(&self) -> bool {
        self.ep_square().is_some()
    }

    // fn reduces_castling_rights(&self, move_: Move) -> bool {
    //     let cr = self.clean_castling_rights();
    //     let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
    //     let touched_cr = touched & cr;
    //     let kings = self.get_piece_mask(King);
    //     let touched_kings_cr = touched_cr & kings;
    //     !touched_cr.is_empty()
    //         || !(BB_RANK_1 & touched_kings_cr & self.occupied_color(White)).is_empty()
    //         || !(BB_RANK_8 & touched_kings_cr & self.occupied_color(Black)).is_empty()
    // }

    pub fn reduces_castling_rights(&self, move_: Move) -> bool {
        // TODO: Check Logic
        let cr = self.clean_castling_rights();
        let touched = move_.get_source().to_bitboard() ^ move_.get_dest().to_bitboard();
        let touched_cr = touched & cr;
        let touched_kings_cr_is_empty = (touched_cr & self.get_piece_mask(King)).is_empty();
        !(touched_cr.is_empty()
            && touched_kings_cr_is_empty
            && BB_RANK_1.is_empty()
            && self.occupied_color(White).is_empty()
            && BB_RANK_8.is_empty()
            && self.occupied_color(Black).is_empty())
    }

    #[inline]
    pub fn is_irreversible(&self, move_: Move) -> bool {
        self.has_legal_en_passant() || self.is_zeroing(move_) || self.reduces_castling_rights(move_)
    }

    #[inline]
    pub fn is_endgame(&self) -> bool {
        if self.get_num_pieces() <= ENDGAME_PIECE_THRESHOLD {
            return true;
        }
        match self.get_piece_mask(Queen).popcnt() {
            0 => {
                (self.get_piece_mask(Rook)
                    ^ self.get_piece_mask(Bishop)
                    ^ self.get_piece_mask(Knight))
                .popcnt()
                    <= 4
            }
            1 => {
                self.get_piece_mask(Rook).popcnt() <= 2
                    && (self.get_piece_mask(Bishop) ^ self.get_piece_mask(Knight)).is_empty()
            }
            2 => (self.get_piece_mask(Rook)
                ^ self.get_piece_mask(Bishop)
                ^ self.get_piece_mask(Knight))
            .is_empty(),
            _ => false,
        }
    }

    #[inline]
    pub fn parse_move(&self, move_text: &str) -> Result<ValidOrNullMove> {
        self.parse_uci(move_text)
            .or(self.parse_san(move_text))
            .or(self.parse_lan(move_text))
            .map_err(|_| TimecatError::InvalidMoveString {
                s: move_text.to_string(),
            })
    }

    #[inline]
    pub fn score_flipped(&self, score: Score) -> Score {
        if self.turn() == White {
            score
        } else {
            -score
        }
    }

    #[inline]
    pub fn get_material_score(&self) -> Score {
        self.get_white_material_score() - self.get_black_material_score()
    }

    pub fn get_winning_side(&self) -> Option<Color> {
        let material_score = self.get_material_score();
        if material_score.is_positive() {
            Some(White)
        } else if material_score.is_negative() {
            Some(Black)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_material_score_flipped(&self) -> Score {
        self.score_flipped(self.get_material_score())
    }

    #[inline]
    pub fn get_masked_material_score_abs(&self, mask: BitBoard) -> Score {
        get_item_unchecked!(ALL_PIECE_TYPES, ..5)
            .iter()
            .map(|&piece| piece.evaluate() * (self.get_piece_mask(piece) & mask).popcnt() as Score)
            .sum()
    }

    #[inline]
    pub fn get_material_score_abs(&self) -> Score {
        self.get_white_material_score() + self.get_black_material_score()
    }

    #[inline]
    pub fn get_non_pawn_material_score_abs(&self) -> Score {
        self.get_material_score() - PAWN_VALUE * self.get_piece_mask(Pawn).popcnt() as Score
    }

    #[inline]
    pub fn slow_evaluate(&self) -> Score {
        Evaluator::slow_evaluate(self)
    }

    #[inline]
    pub fn slow_evaluate_flipped(&self) -> Score {
        self.score_flipped(self.slow_evaluate())
    }

    #[inline]
    pub fn custom_iter<'a>(
        &'a self,
        piece_types: &'a [PieceType],
        colors: &'a [Color],
        mask: BitBoard,
    ) -> impl Iterator<Item = (Piece, Square)> + 'a {
        piece_types
            .iter()
            .cartesian_product(colors)
            .flat_map(move |(&piece_type, &color)| {
                (self.get_piece_mask(piece_type) & self.occupied_color(color) & mask)
                    .into_iter()
                    .map(move |square| (Piece::new(piece_type, color), square))
            })
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Piece, Square)> + '_ {
        self.custom_iter(&ALL_PIECE_TYPES, &ALL_COLORS, BB_ALL)
    }

    #[cfg(feature = "pyo3")]
    fn from_py_board(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let pieces_masks = [
            BitBoard::new(ob.getattr("pawns")?.extract()?),
            BitBoard::new(ob.getattr("knights")?.extract()?),
            BitBoard::new(ob.getattr("bishops")?.extract()?),
            BitBoard::new(ob.getattr("rooks")?.extract()?),
            BitBoard::new(ob.getattr("queens")?.extract()?),
            BitBoard::new(ob.getattr("kings")?.extract()?),
        ];
        let (black_occupied, white_occupied) = {
            if let Ok(occupied_color_py_object) = ob.getattr("occupied_color") {
                (
                    occupied_color_py_object
                        .get_item(0)?
                        .extract::<BitBoard>()?,
                    occupied_color_py_object
                        .get_item(1)?
                        .extract::<BitBoard>()?,
                )
            } else {
                (
                    ob.getattr("occupied_b")?.extract::<BitBoard>()?,
                    ob.getattr("occupied_w")?.extract::<BitBoard>()?,
                )
            }
        };
        let (white_castle_rights, black_castle_rights) = {
            let castling_rights_bb = ob.getattr("castling_rights")?.extract::<BitBoard>()?;
            const BB_A1_H1: BitBoard = BitBoard::new(BB_A1.into_inner() ^ BB_H1.into_inner());
            const BB_A8_H8: BitBoard = BitBoard::new(BB_A8.into_inner() ^ BB_H8.into_inner());
            (
                match castling_rights_bb & White.to_my_backrank().to_bitboard() {
                    BitBoard::EMPTY => CastleRights::None,
                    BB_H1 => CastleRights::KingSide,
                    BB_A1 => CastleRights::QueenSide,
                    BB_A1_H1 => CastleRights::Both,
                    _ => {
                        return Err(Pyo3Error::Pyo3TypeConversionError {
                            from: ob.to_string(),
                            to: std::any::type_name::<Self>().to_string(),
                        }
                        .into())
                    }
                },
                match castling_rights_bb & Black.to_my_backrank().to_bitboard() {
                    BitBoard::EMPTY => CastleRights::None,
                    BB_H8 => CastleRights::KingSide,
                    BB_A8 => CastleRights::QueenSide,
                    BB_A8_H8 => CastleRights::Both,
                    _ => {
                        return Err(Pyo3Error::Pyo3TypeConversionError {
                            from: ob.to_string(),
                            to: std::any::type_name::<Self>().to_string(),
                        }
                        .into())
                    }
                },
            )
        };
        Ok(BoardPositionBuilder::setup(
            ALL_PIECE_TYPES
                .iter()
                .zip(pieces_masks)
                .flat_map(|(&piece_type, pieces_mask)| {
                    (pieces_mask & white_occupied)
                        .map(move |square| (square, Piece::new(piece_type, White)))
                        .chain(
                            (pieces_mask & black_occupied)
                                .map(move |square| (square, Piece::new(piece_type, Black))),
                        )
                }),
            ob.getattr("turn")?.extract()?,
            white_castle_rights,
            black_castle_rights,
            ob.getattr("ep_square")?
                .extract::<Option<Square>>()
                .unwrap_or_default()
                .map(|square| square.get_file()),
            ob.getattr("halfmove_clock")?.extract().unwrap_or(0),
            ob.getattr("fullmove_number")?.extract().unwrap_or(1),
        )
        .try_into()?)
    }
}

impl BoardPositionMethodOverload<Move> for BoardPosition {
    #[inline]
    fn parse_san(&self, san: &str) -> Result<Move> {
        Move::from_san(self, san)
    }

    #[inline]
    fn parse_lan(&self, lan: &str) -> Result<Move> {
        Move::from_lan(self, lan)
    }

    #[inline]
    fn parse_uci(&self, uci: &str) -> Result<Move> {
        Move::from_str(uci)
    }

    fn make_move_new(&self, move_: Move) -> Self {
        let mut result = self.clone();

        if result.is_zeroing(move_) {
            result._halfmove_clock = 0;
        } else {
            result._halfmove_clock += 1;
        }
        if result.turn() == Black {
            result._fullmove_number += 1;
        }

        result.remove_ep();
        result._checkers = BitBoard::EMPTY;
        result._pinned = BitBoard::EMPTY;
        let source = move_.get_source();
        let dest = move_.get_dest();

        let source_bb = source.to_bitboard();
        let dest_bb = dest.to_bitboard();
        let move_bb = source_bb ^ dest_bb;
        let moved = self.get_piece_type_at(source).unwrap();

        result.xor(moved, source_bb, self.turn());
        result.xor(moved, dest_bb, self.turn());
        if let Some(captured) = self.get_piece_type_at(dest) {
            result.xor(captured, dest_bb, !self.turn());
        }

        result
            .remove_their_castle_rights(CastleRights::square_to_castle_rights(!self.turn(), dest));

        result.remove_my_castle_rights(CastleRights::square_to_castle_rights(self.turn(), source));

        let opp_king = result.get_colored_piece_mask(King, !result.turn());

        let castles = moved == King && (move_bb & get_castle_moves()) == move_bb;

        let ksq = opp_king.to_square_unchecked();

        if moved == Knight {
            result._checkers ^= ksq.get_knight_moves() & dest_bb;
        } else if moved == Pawn {
            if let Some(Knight) = move_.get_promotion() {
                result.xor(Pawn, dest_bb, self.turn());
                result.xor(Knight, dest_bb, self.turn());
                result._checkers ^= ksq.get_knight_moves() & dest_bb;
            } else if let Some(promotion) = move_.get_promotion() {
                result.xor(Pawn, dest_bb, self.turn());
                result.xor(promotion, dest_bb, self.turn());
            } else if !(source_bb & get_pawn_source_double_moves()).is_empty()
                && !(dest_bb & get_pawn_dest_double_moves()).is_empty()
            {
                result.set_ep(dest.wrapping_backward(result.turn()));
                result._checkers ^= ksq.get_pawn_attacks(!result.turn(), dest_bb);
            } else if Some(dest) == self.ep_square() {
                result.xor(
                    Pawn,
                    dest.wrapping_backward(self.turn()).to_bitboard(),
                    !self.turn(),
                );
                result._checkers ^= ksq.get_pawn_attacks(!result.turn(), dest_bb);
            } else {
                result._checkers ^= ksq.get_pawn_attacks(!result.turn(), dest_bb);
            }
        } else if castles {
            let my_backrank = self.turn().to_my_backrank();
            let index = dest.get_file().to_index();
            let start = BitBoard::from_rank_and_file(
                my_backrank,
                *get_item_unchecked!(
                    const
                    [
                        File::A,
                        File::A,
                        File::A,
                        File::A,
                        File::H,
                        File::H,
                        File::H,
                        File::H,
                    ],
                    index,
                ),
            );
            let end = BitBoard::from_rank_and_file(
                my_backrank,
                *get_item_unchecked!(
                    const
                    [
                        File::D,
                        File::D,
                        File::D,
                        File::D,
                        File::F,
                        File::F,
                        File::F,
                        File::F,
                    ],
                    index,
                ),
            );
            result.xor(Rook, start, self.turn());
            result.xor(Rook, end, self.turn());
        }
        // now, lets see if we're in check or pinned
        let attackers = result.occupied_color(result.turn())
            & ((ksq.get_bishop_rays_bb()
                & (result.get_piece_mask(Bishop) | result.get_piece_mask(Queen)))
                | (ksq.get_rook_rays_bb()
                    & (result.get_piece_mask(Rook) | result.get_piece_mask(Queen))));

        for square in attackers {
            let between = square.between(ksq) & result.occupied();
            if between.is_empty() {
                result._checkers ^= square.to_bitboard();
            } else if between.popcnt() == 1 {
                result._pinned ^= between;
            }
        }

        result.flip_turn_unchecked();

        result
    }
}

impl BoardPositionMethodOverload<ValidOrNullMove> for BoardPosition {
    #[inline]
    fn parse_san(&self, san: &str) -> Result<ValidOrNullMove> {
        ValidOrNullMove::from_san(self, san)
    }

    #[inline]
    fn parse_lan(&self, lan: &str) -> Result<ValidOrNullMove> {
        ValidOrNullMove::from_lan(self, lan)
    }

    #[inline]
    fn parse_uci(&self, uci: &str) -> Result<ValidOrNullMove> {
        ValidOrNullMove::from_str(uci)
    }

    fn make_move_new(&self, valid_or_null_move: ValidOrNullMove) -> Self {
        if let Some(move_) = *valid_or_null_move {
            self.make_move_new(move_)
        } else {
            self.null_move().unwrap()
        }
    }
}

impl TryFrom<&BoardPositionBuilder> for BoardPosition {
    type Error = TimecatError;

    fn try_from(position_builder: &BoardPositionBuilder) -> Result<Self> {
        let mut position = BoardPosition::new_empty();

        for square in ALL_SQUARES {
            if let Some(piece) = position_builder[square] {
                position.xor(
                    piece.get_piece_type(),
                    square.to_bitboard(),
                    piece.get_color(),
                );
            }
        }

        position.set_turn_unchecked(position_builder.get_turn());

        if let Some(ep) = position_builder.get_en_passant() {
            position._turn = !position.turn();
            position.set_ep(ep);
            position._turn = !position.turn();
        }

        position.add_castle_rights(White, position_builder.get_castle_rights(White));
        position.add_castle_rights(Black, position_builder.get_castle_rights(Black));

        position._halfmove_clock = position_builder.get_halfmove_clock();
        position._fullmove_number = position_builder.get_fullmove_number();

        position.update_pin_and_checkers_info();

        if position.is_sane() {
            Ok(position)
        } else {
            Err(TimecatError::InvalidBoardPosition { position })
        }
    }
}

impl TryFrom<BoardPositionBuilder> for BoardPosition {
    type Error = TimecatError;

    fn try_from(position_builder: BoardPositionBuilder) -> Result<Self> {
        (&position_builder).try_into()
    }
}

impl TryFrom<&mut BoardPositionBuilder> for BoardPosition {
    type Error = TimecatError;

    fn try_from(position_builder: &mut BoardPositionBuilder) -> Result<Self> {
        (position_builder.to_owned()).try_into()
    }
}

impl FromStr for BoardPosition {
    type Err = TimecatError;

    fn from_str(value: &str) -> Result<Self> {
        BoardPositionBuilder::from_str(value)?.try_into()
    }
}

impl Default for BoardPosition {
    #[inline]
    fn default() -> Self {
        Self::from_str(STARTING_POSITION_FEN).unwrap()
    }
}

impl fmt::Display for BoardPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fen: BoardPositionBuilder = self.into();
        write!(f, "{}", fen)
    }
}

impl Hash for BoardPosition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.get_hash())
    }
}

#[cfg(feature = "pyo3")]
impl<'source> FromPyObject<'source> for BoardPosition {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> PyResult<Self> {
        if let Ok(fen) = ob.extract::<&str>() {
            if let Ok(position) = Self::from_str(fen) {
                return Ok(position);
            }
        }
        if let Ok(position) = BoardPosition::from_py_board(ob) {
            return Ok(position);
        }
        Err(Pyo3Error::Pyo3TypeConversionError {
            from: ob.to_string(),
            to: std::any::type_name::<Self>().to_string(),
        }
        .into())
    }
}
