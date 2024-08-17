use super::*;

trait PieceMoves {
    fn is(piece: PieceType) -> bool;
    fn into_piece() -> PieceType;
    fn pseudo_legals(src: Square, color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard;
    fn legals<T>(move_list: &mut MoveList, mini_board: &MiniBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = mini_board.occupied();
        let color = mini_board.turn();
        let my_pieces = mini_board.occupied_co(color);
        let ksq = mini_board.get_king_square(color);

        let pieces = mini_board.get_piece_mask(Self::into_piece()) & my_pieces;
        let pinned = mini_board.pinned();
        let checkers = mini_board.get_checkers();

        let check_mask = if T::IN_CHECK {
            checkers.to_square().between(ksq) ^ checkers
        } else {
            BB_ALL
        };

        for src in pieces & !pinned {
            let square_and_bitboard_array =
                Self::pseudo_legals(src, color, occupied, mask) & check_mask;
            if !square_and_bitboard_array.is_empty() {
                unsafe {
                    move_list.push_unchecked(SquareAndBitBoard::new(
                        src,
                        square_and_bitboard_array,
                        false,
                    ));
                }
            }
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let square_and_bitboard_array =
                    Self::pseudo_legals(src, color, occupied, mask) & src.line(ksq);
                if !square_and_bitboard_array.is_empty() {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            square_and_bitboard_array,
                            false,
                        ));
                    }
                }
            }
        }
    }
}

struct PawnMoves;
struct BishopMoves;
struct KnightMoves;
struct RookMoves;
struct QueenMoves;
struct KingMoves;

trait CheckMoves {
    const IN_CHECK: bool;
}

struct InCheckMoves;
struct NotInCheckMoves;

impl CheckMoves for InCheckMoves {
    const IN_CHECK: bool = true;
}

impl CheckMoves for NotInCheckMoves {
    const IN_CHECK: bool = false;
}

impl PawnMoves {
    fn legal_ep_move(mini_board: &MiniBoard, source: Square, dest: Square) -> bool {
        let occupied = mini_board.occupied()
            ^ mini_board
                .ep_square()
                .unwrap()
                .wrapping_backward(mini_board.turn())
                .to_bitboard()
            ^ source.to_bitboard()
            ^ dest.to_bitboard();

        let ksq = (mini_board.get_piece_mask(King) & mini_board.self_occupied()).to_square();

        let rooks = (mini_board.get_piece_mask(Rook) ^ mini_board.get_piece_mask(Queen))
            & mini_board.opponent_occupied();

        if !(ksq.get_rook_rays_bb() & rooks).is_empty()
            && !(get_rook_moves(ksq, occupied) & rooks).is_empty()
        {
            return false;
        }

        let bishops = (mini_board.get_piece_mask(Bishop) ^ mini_board.get_piece_mask(Queen))
            & mini_board.opponent_occupied();

        if !(ksq.get_bishop_rays_bb() & bishops).is_empty()
            && !(get_bishop_moves(ksq, occupied) & bishops).is_empty()
        {
            return false;
        }

        true
    }
}

impl PieceMoves for PawnMoves {
    fn is(piece: PieceType) -> bool {
        piece == Pawn
    }

    fn into_piece() -> PieceType {
        Pawn
    }

    #[inline]
    fn pseudo_legals(src: Square, color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_pawn_moves(src, color, occupied) & mask
    }

    #[inline]
    fn legals<T>(move_list: &mut MoveList, mini_board: &MiniBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = mini_board.occupied();
        let color = mini_board.turn();
        let my_pieces = mini_board.occupied_co(color);
        let ksq = mini_board.get_king_square(color);

        let pieces = mini_board.get_piece_mask(Self::into_piece()) & my_pieces;
        let pinned = mini_board.pinned();
        let checkers = mini_board.get_checkers();

        let check_mask = if T::IN_CHECK {
            checkers.to_square().between(ksq) ^ checkers
        } else {
            BB_ALL
        };

        for src in pieces & !pinned {
            let square_and_bitboard_array =
                Self::pseudo_legals(src, color, occupied, mask) & check_mask;
            if !square_and_bitboard_array.is_empty() {
                unsafe {
                    move_list.push_unchecked(SquareAndBitBoard::new(
                        src,
                        square_and_bitboard_array,
                        src.get_rank() == color.to_seventh_rank(),
                    ));
                }
            }
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let square_and_bitboard_array =
                    Self::pseudo_legals(src, color, occupied, mask) & ksq.line(src);
                if !square_and_bitboard_array.is_empty() {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            square_and_bitboard_array,
                            src.get_rank() == color.to_seventh_rank(),
                        ));
                    }
                }
            }
        }

        if let Some(dest) = mini_board.ep_square() {
            let dest_rank = dest.get_rank();
            let rank_bb = if dest_rank.to_int() > 3 {
                dest_rank.wrapping_down().to_bitboard()
            } else {
                dest_rank.wrapping_up().to_bitboard()
            };
            let files_bb = dest.get_file().get_adjacent_files_bb();
            for src in rank_bb & files_bb & pieces {
                if PawnMoves::legal_ep_move(mini_board, src, dest) {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            dest.to_bitboard(),
                            false,
                        ));
                    }
                }
            }
        }
    }
}

impl PieceMoves for BishopMoves {
    fn is(piece: PieceType) -> bool {
        piece == Bishop
    }

    fn into_piece() -> PieceType {
        Bishop
    }

    #[inline]
    fn pseudo_legals(src: Square, _color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_bishop_moves(src, occupied) & mask
    }
}

impl PieceMoves for KnightMoves {
    fn is(piece: PieceType) -> bool {
        piece == Knight
    }

    fn into_piece() -> PieceType {
        Knight
    }

    #[inline]
    fn pseudo_legals(src: Square, _color: Color, _occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_knight_moves(src) & mask
    }

    #[inline]
    fn legals<T>(move_list: &mut MoveList, mini_board: &MiniBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = mini_board.occupied();
        let color = mini_board.turn();
        let my_pieces = mini_board.occupied_co(color);
        let ksq = mini_board.get_king_square(color);

        let pieces = mini_board.get_piece_mask(Self::into_piece()) & my_pieces;
        let pinned = mini_board.pinned();
        let checkers = mini_board.get_checkers();

        if T::IN_CHECK {
            let check_mask = checkers.to_square().between(ksq) ^ checkers;

            for src in pieces & !pinned {
                let square_and_bitboard_array =
                    Self::pseudo_legals(src, color, occupied, mask & check_mask);
                if !square_and_bitboard_array.is_empty() {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            square_and_bitboard_array,
                            false,
                        ));
                    }
                }
            }
        } else {
            for src in pieces & !pinned {
                let square_and_bitboard_array = Self::pseudo_legals(src, color, occupied, mask);
                if !square_and_bitboard_array.is_empty() {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            square_and_bitboard_array,
                            false,
                        ));
                    }
                }
            }
        };
    }
}

impl PieceMoves for RookMoves {
    fn is(piece: PieceType) -> bool {
        piece == Rook
    }

    fn into_piece() -> PieceType {
        Rook
    }

    #[inline]
    fn pseudo_legals(src: Square, _color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_rook_moves(src, occupied) & mask
    }
}

impl PieceMoves for QueenMoves {
    fn is(piece: PieceType) -> bool {
        piece == Queen
    }

    fn into_piece() -> PieceType {
        Queen
    }

    #[inline]
    fn pseudo_legals(src: Square, _color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard {
        (get_rook_moves(src, occupied) ^ get_bishop_moves(src, occupied)) & mask
    }
}

impl KingMoves {
    #[inline]
    fn legal_king_move(mini_board: &MiniBoard, dest: Square) -> bool {
        let occupied = mini_board.occupied()
            ^ (mini_board.get_piece_mask(King) & mini_board.self_occupied())
            | dest.to_bitboard();

        let rooks = (mini_board.get_piece_mask(Rook) ^ mini_board.get_piece_mask(Queen))
            & mini_board.opponent_occupied();

        let mut attackers = get_rook_moves(dest, occupied) & rooks;

        let bishops = (mini_board.get_piece_mask(Bishop) ^ mini_board.get_piece_mask(Queen))
            & mini_board.opponent_occupied();

        attackers |= get_bishop_moves(dest, occupied) & bishops;

        let knight_rays = get_knight_moves(dest);

        // Using ^ because knight square_and_bitboard_array bitboard do not collide with rook and bishop square_and_bitboard_array bitboard
        attackers ^=
            knight_rays & mini_board.get_piece_mask(Knight) & mini_board.opponent_occupied();

        let king_rays = get_king_moves(dest);
        attackers |= king_rays & mini_board.get_piece_mask(King) & mini_board.opponent_occupied();

        attackers |= get_pawn_attacks(
            dest,
            mini_board.turn(),
            mini_board.get_piece_mask(Pawn) & mini_board.opponent_occupied(),
        );

        attackers.is_empty()
    }
}

impl PieceMoves for KingMoves {
    fn is(piece: PieceType) -> bool {
        piece == King
    }

    fn into_piece() -> PieceType {
        King
    }

    #[inline]
    fn pseudo_legals(src: Square, _color: Color, _occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_king_moves(src) & mask
    }

    #[inline]
    fn legals<T>(move_list: &mut MoveList, mini_board: &MiniBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = mini_board.occupied();
        let color = mini_board.turn();
        let ksq = mini_board.get_king_square(color);

        let mut square_and_bitboard_array = Self::pseudo_legals(ksq, color, occupied, mask);

        let copy = square_and_bitboard_array;
        for dest in copy {
            if !KingMoves::legal_king_move(mini_board, dest) {
                square_and_bitboard_array ^= dest.to_bitboard();
            }
        }

        // If we are not in check, we may be able to castle.
        // We can do so iff:
        //  * the `MiniBoard` structure says we can.
        //  * the squares between my king and my rook are empty.
        //  * no enemy pieces are attacking the squares between the king, and the kings
        //    destination square.
        //  ** This is determined by going to the left or right, and calling
        //     'legal_king_move' for that square.
        if !T::IN_CHECK {
            if mini_board.my_castle_rights().has_kingside()
                && (occupied & mini_board.my_castle_rights().kingside_squares(color)).is_empty()
            {
                let middle = ksq.wrapping_right();
                let right = middle.wrapping_right();
                if KingMoves::legal_king_move(mini_board, middle)
                    && KingMoves::legal_king_move(mini_board, right)
                {
                    square_and_bitboard_array ^= right.to_bitboard();
                }
            }

            if mini_board.my_castle_rights().has_queenside()
                && (occupied & mini_board.my_castle_rights().queenside_squares(color)).is_empty()
            {
                let middle = ksq.wrapping_left();
                let left = middle.wrapping_left();
                if KingMoves::legal_king_move(mini_board, middle)
                    && KingMoves::legal_king_move(mini_board, left)
                {
                    square_and_bitboard_array ^= left.to_bitboard();
                }
            }
        }
        if !square_and_bitboard_array.is_empty() {
            unsafe {
                move_list.push_unchecked(SquareAndBitBoard::new(
                    ksq,
                    square_and_bitboard_array,
                    false,
                ));
            }
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Eq, Debug)]
struct SquareAndBitBoard {
    square: Square,
    bitboard: BitBoard,
    promotion: bool,
}

impl SquareAndBitBoard {
    fn new(square: Square, bb: BitBoard, promotion: bool) -> SquareAndBitBoard {
        SquareAndBitBoard {
            square,
            bitboard: bb,
            promotion,
        }
    }
}

type MoveList = ArrayVec<SquareAndBitBoard, 18>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct MoveGenerator {
    square_and_bitboard_array: MoveList,
    promotion_index: usize,
    from_bitboard_iterator_mask: BitBoard,
    to_bitboard_iterator_mask: BitBoard,
    index: usize,
    last_index: usize,
}

impl MoveGenerator {
    #[inline]
    fn enumerate_moves(mini_board: &MiniBoard) -> MoveList {
        let checkers = mini_board.get_checkers();
        let mask = !mini_board.self_occupied();
        let mut move_list = ArrayVec::new();

        if checkers.is_empty() {
            PawnMoves::legals::<NotInCheckMoves>(&mut move_list, mini_board, mask);
            KnightMoves::legals::<NotInCheckMoves>(&mut move_list, mini_board, mask);
            BishopMoves::legals::<NotInCheckMoves>(&mut move_list, mini_board, mask);
            RookMoves::legals::<NotInCheckMoves>(&mut move_list, mini_board, mask);
            QueenMoves::legals::<NotInCheckMoves>(&mut move_list, mini_board, mask);
            KingMoves::legals::<NotInCheckMoves>(&mut move_list, mini_board, mask);
        } else if checkers.popcnt() == 1 {
            PawnMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
            KnightMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
            BishopMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
            RookMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
            QueenMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
            KingMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
        } else {
            KingMoves::legals::<InCheckMoves>(&mut move_list, mini_board, mask);
        }

        move_list
    }

    #[inline]
    pub fn new_legal(mini_board: &MiniBoard) -> MoveGenerator {
        MoveGenerator {
            square_and_bitboard_array: MoveGenerator::enumerate_moves(mini_board),
            promotion_index: 0,
            from_bitboard_iterator_mask: BB_ALL,
            to_bitboard_iterator_mask: BB_ALL,
            index: 0,
            last_index: usize::MAX,
        }
    }

    /// The iterator portion of this struct relies on the invariant that
    /// the bitboards at the beginning of the square_and_bitboard_array[] array are the only
    /// ones used.  As a result, we must partition the list such that the
    /// assumption is true.
    pub fn reorganize_square_and_bitboard_array(&mut self) {
        // first, find the first non-used square_and_bitboard_array index, and store that in i
        let mut i = 0;
        while i < self.square_and_bitboard_array.len()
            && !(get_item_unchecked!(self.square_and_bitboard_array, i).bitboard
                & self.to_bitboard_iterator_mask)
                .is_empty()
        {
            i += 1;
        }

        // next, find each element past i where the square_and_bitboard_array are used, and store
        // that in i.  Then, increment i to point to a new unused slot.
        for j in (i + 1)..self.square_and_bitboard_array.len() {
            if !(get_item_unchecked!(self.square_and_bitboard_array, j).bitboard
                & self.to_bitboard_iterator_mask)
                .is_empty()
            {
                // unsafe { self.square_and_bitboard_array.swap_unchecked(i, j) };
                self.square_and_bitboard_array.swap(i, j);
                i += 1;
            }
        }
    }

    pub fn remove_mask(&mut self, mask: BitBoard) {
        self.square_and_bitboard_array
            .iter_mut()
            .for_each(|square_and_bitboard| square_and_bitboard.bitboard &= !mask);
        self.reorganize_square_and_bitboard_array();
    }

    pub fn remove_move(&mut self, move_: Move) -> bool {
        for square_and_bitboard in self.square_and_bitboard_array.iter_mut() {
            if square_and_bitboard.square == move_.get_source() {
                square_and_bitboard.bitboard &= !move_.get_dest().to_bitboard();
                self.reorganize_square_and_bitboard_array();
                return true;
            }
        }
        self.reorganize_square_and_bitboard_array();
        false
    }

    pub fn reset_indices(&mut self) {
        self.index = 0;
        self.last_index = usize::MAX;
    }

    pub fn get_from_bitboard_iterator_mask(&self) -> BitBoard {
        self.from_bitboard_iterator_mask
    }

    pub fn set_from_bitboard_iterator_mask(&mut self, mask: BitBoard) {
        self.reset_indices();
        self.from_bitboard_iterator_mask = mask;
        self.reorganize_square_and_bitboard_array();
    }

    pub fn get_to_bitboard_iterator_mask(&self) -> BitBoard {
        self.to_bitboard_iterator_mask
    }

    pub fn set_to_bitboard_iterator_mask(&mut self, mask: BitBoard) {
        self.reset_indices();
        self.to_bitboard_iterator_mask = mask;
        self.reorganize_square_and_bitboard_array();
    }

    pub fn set_iterator_masks(&mut self, from_bitboard: BitBoard, to_bitboard: BitBoard) {
        self.reset_indices();
        self.from_bitboard_iterator_mask = from_bitboard;
        self.to_bitboard_iterator_mask = to_bitboard;
        self.reorganize_square_and_bitboard_array();
    }

    pub fn is_legal_quick(mini_board: &MiniBoard, move_: Move) -> bool {
        let piece = mini_board.get_piece_type_at(move_.get_source()).unwrap();
        match piece {
            Rook => true,
            Bishop => true,
            Knight => true,
            Queen => true,
            Pawn => {
                if move_.get_source().get_file() != move_.get_dest().get_file()
                    && mini_board.get_piece_type_at(move_.get_dest()).is_none()
                {
                    // en-passant
                    PawnMoves::legal_ep_move(mini_board, move_.get_source(), move_.get_dest())
                } else {
                    true
                }
            }
            King => {
                let bb = move_.get_source().between(move_.get_dest());
                if bb.popcnt() == 1 {
                    // castles
                    if !KingMoves::legal_king_move(mini_board, bb.to_square()) {
                        false
                    } else {
                        KingMoves::legal_king_move(mini_board, move_.get_dest())
                    }
                } else {
                    KingMoves::legal_king_move(mini_board, move_.get_dest())
                }
            }
        }
    }

    pub fn perft_test(mini_board: &MiniBoard, depth: usize) -> usize {
        let iterable = mini_board.generate_legal_moves();

        let mut result: usize = 0;
        if depth == 1 {
            iterable.len()
        } else {
            for m in iterable {
                let board_result = mini_board.make_move_new(m);
                result += MoveGenerator::perft_test(&board_result, depth - 1);
            }
            result
        }
    }

    pub fn perft_test_piecewise(mini_board: &MiniBoard, depth: usize) -> usize {
        let mut iterable = mini_board.generate_legal_moves();

        let targets = mini_board.opponent_occupied();
        let mut result: usize = 0;

        for &piece_mask in mini_board.get_all_piece_masks() {
            iterable.set_from_bitboard_iterator_mask(piece_mask);
            if depth == 1 {
                iterable.set_to_bitboard_iterator_mask(targets);
                result += iterable.len();
                iterable.set_to_bitboard_iterator_mask(!targets);
                result += iterable.len();
            } else {
                iterable.set_to_bitboard_iterator_mask(targets);
                for x in &mut iterable {
                    result += MoveGenerator::perft_test(&mini_board.make_move_new(x), depth - 1);
                }
                iterable.set_to_bitboard_iterator_mask(!targets);
                for x in &mut iterable {
                    result += MoveGenerator::perft_test(&mini_board.make_move_new(x), depth - 1);
                }
            }
        }
        result
    }

    pub fn iter(&self) -> impl Iterator<Item = Move> + '_ {
        self.square_and_bitboard_array
            .iter()
            .filter(|square_and_bitboard| {
                self.from_bitboard_iterator_mask
                    .contains(square_and_bitboard.square)
            })
            .take_while(|square_and_bitboard| {
                !(square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).is_empty()
            })
            .flat_map(move |square_and_bitboard| {
                let promotion_pieces: &[Option<PieceType>] = if square_and_bitboard.promotion {
                    const { &[Some(Queen), Some(Knight), Some(Rook), Some(Bishop)] }
                } else {
                    const { &[None] }
                };
                promotion_pieces.iter().flat_map(move |&promotion| {
                    (square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).map(
                        move |dest| {
                            Move::new_unchecked(square_and_bitboard.square, dest, promotion)
                        },
                    )
                })
            })
    }

    pub fn contains(&self, move_: &Move) -> bool {
        self.square_and_bitboard_array
            .iter()
            .find(|square_and_bitboard| square_and_bitboard.square == move_.get_source())
            .map(|square_and_bitboard| square_and_bitboard.bitboard.contains(move_.get_dest()))
            .is_some()
    }
}

impl ExactSizeIterator for MoveGenerator {
    fn len(&self) -> usize {
        let mut result = 0;
        for square_and_bitboard in &self.square_and_bitboard_array {
            let bitboard_and_to_bitboard_iterator_mask =
                square_and_bitboard.bitboard & self.to_bitboard_iterator_mask;
            if !self
                .from_bitboard_iterator_mask
                .contains(square_and_bitboard.square)
                || bitboard_and_to_bitboard_iterator_mask.is_empty()
            {
                break;
            }
            if square_and_bitboard.promotion {
                result += (bitboard_and_to_bitboard_iterator_mask.popcnt() as usize)
                    * NUM_PROMOTION_PIECES;
            } else {
                result += bitboard_and_to_bitboard_iterator_mask.popcnt() as usize;
            }
        }
        result
    }
}

impl Iterator for MoveGenerator {
    type Item = Move;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Move> {
        let square_and_bitboard_array_len = self.square_and_bitboard_array.len();
        if self.index >= square_and_bitboard_array_len {
            return None;
        }
        if self.index != self.last_index {
            while !self.from_bitboard_iterator_mask.contains(
                get_item_unchecked_mut!(self.square_and_bitboard_array, self.index).square,
            ) {
                self.index += 1;
                if self.index >= square_and_bitboard_array_len {
                    return None;
                }
            }
            self.last_index = self.index;
        }
        let square_and_bitboard =
            get_item_unchecked_mut!(self.square_and_bitboard_array, self.index);

        if !self
            .from_bitboard_iterator_mask
            .contains(square_and_bitboard.square)
            || (square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).is_empty()
        {
            // are we done?
            None
        } else if square_and_bitboard.promotion {
            let dest = (square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).to_square();

            // deal with potential promotions for this pawn
            let result = Move::new_unchecked(
                square_and_bitboard.square,
                dest,
                Some(*get_item_unchecked!(PROMOTION_PIECES, self.promotion_index)),
            );
            self.promotion_index += 1;
            if self.promotion_index >= NUM_PROMOTION_PIECES {
                square_and_bitboard.bitboard ^= dest.to_bitboard();
                self.promotion_index = 0;
                if (square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).is_empty() {
                    self.index += 1;
                }
            }
            Some(result)
        } else {
            // not a promotion move, so its a 'normal' move as far as this function is concerned
            let dest = (square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).to_square();

            square_and_bitboard.bitboard ^= dest.to_bitboard();
            if (square_and_bitboard.bitboard & self.to_bitboard_iterator_mask).is_empty() {
                self.index += 1;
            }
            Some(Move::new_unchecked(square_and_bitboard.square, dest, None))
        }
    }
}
