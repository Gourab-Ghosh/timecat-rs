use super::*;

trait PieceTypeTrait {
    fn is(piece: PieceType) -> bool;
    fn into_piece() -> PieceType;
    fn pseudo_legals(src: Square, color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard;
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let occupied = board.occupied();
        let color = board.side_to_move();
        let my_pieces = board.occupied_co(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if T::IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !BB_EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *occupied, mask) & check_mask;
            if moves != BB_EMPTY {
                unsafe {
                    move_list.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                }
            }
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let moves = Self::pseudo_legals(src, color, *occupied, mask) & line(src, ksq);
                if moves != BB_EMPTY {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        }
    }
}

struct PawnType;
struct BishopType;
struct KnightType;
struct RookType;
struct QueenType;
struct KingType;

trait CheckType {
    const IN_CHECK: bool;
}

struct InCheckType;
struct NotInCheckType;

impl CheckType for InCheckType {
    const IN_CHECK: bool = true;
}

impl CheckType for NotInCheckType {
    const IN_CHECK: bool = false;
}

impl PawnType {
    fn legal_ep_move(board: &SubBoard, source: Square, dest: Square) -> bool {
        let occupied = board.occupied()
            ^ BitBoard::from_square(board.en_passant().unwrap())
            ^ BitBoard::from_square(source)
            ^ BitBoard::from_square(dest);

        let ksq = (board.pieces(King) & board.occupied_co(board.side_to_move())).to_square();

        let rooks =
            (board.pieces(Rook) | board.pieces(Queen)) & board.occupied_co(!board.side_to_move());

        if (get_rook_rays(ksq) & rooks) != BB_EMPTY && (get_rook_moves(ksq, occupied) & rooks) != BB_EMPTY {
            return false;
        }

        let bishops =
            (board.pieces(Bishop) | board.pieces(Queen)) & board.occupied_co(!board.side_to_move());

        if (get_bishop_rays(ksq) & bishops) != BB_EMPTY && (get_bishop_moves(ksq, occupied) & bishops) != BB_EMPTY {
            return false;
        }

        true
    }
}

impl PieceTypeTrait for PawnType {
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
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let occupied = board.occupied();
        let color = board.side_to_move();
        let my_pieces = board.occupied_co(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        let check_mask = if T::IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !BB_EMPTY
        };

        for src in pieces & !pinned {
            let moves = Self::pseudo_legals(src, color, *occupied, mask) & check_mask;
            if moves != BB_EMPTY {
                unsafe {
                    move_list.push_unchecked(SquareAndBitBoard::new(
                        src,
                        moves,
                        src.get_rank() == color.to_seventh_rank(),
                    ));
                }
            }
        }

        if !T::IN_CHECK {
            for src in pieces & pinned {
                let moves = Self::pseudo_legals(src, color, *occupied, mask) & line(ksq, src);
                if moves != BB_EMPTY {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            moves,
                            src.get_rank() == color.to_seventh_rank(),
                        ));
                    }
                }
            }
        }

        if board.en_passant().is_some() {
            let ep_sq = board.en_passant().unwrap();
            let rank = get_rank_bb(ep_sq.get_rank());
            let files = get_adjacent_files(ep_sq.get_file());
            for src in rank & files & pieces {
                let dest = ep_sq.wrapping_forward(color);
                if PawnType::legal_ep_move(board, src, dest) {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(
                            src,
                            BitBoard::from_square(dest),
                            false,
                        ));
                    }
                }
            }
        }
    }
}

impl PieceTypeTrait for BishopType {
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

impl PieceTypeTrait for KnightType {
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
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let occupied = board.occupied();
        let color = board.side_to_move();
        let my_pieces = board.occupied_co(color);
        let ksq = board.king_square(color);

        let pieces = board.pieces(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.checkers();

        if T::IN_CHECK {
            let check_mask = between(checkers.to_square(), ksq) ^ checkers;

            for src in pieces & !pinned {
                let moves = Self::pseudo_legals(src, color, *occupied, mask & check_mask);
                if moves != BB_EMPTY {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        } else {
            for src in pieces & !pinned {
                let moves = Self::pseudo_legals(src, color, *occupied, mask);
                if moves != BB_EMPTY {
                    unsafe {
                        move_list.push_unchecked(SquareAndBitBoard::new(src, moves, false));
                    }
                }
            }
        };
    }
}

impl PieceTypeTrait for RookType {
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

impl PieceTypeTrait for QueenType {
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

impl KingType {
    #[inline]
    fn legal_king_move(board: &SubBoard, dest: Square) -> bool {
        let occupied = board.occupied()
            ^ (board.pieces(King) & board.occupied_co(board.side_to_move()))
            | BitBoard::from_square(dest);

        let mut attackers = BB_EMPTY;

        let rooks =
            (board.pieces(Rook) | board.pieces(Queen)) & board.occupied_co(!board.side_to_move());

        attackers |= get_rook_moves(dest, occupied) & rooks;

        let bishops =
            (board.pieces(Bishop) | board.pieces(Queen)) & board.occupied_co(!board.side_to_move());

        attackers |= get_bishop_moves(dest, occupied) & bishops;

        let knight_rays = get_knight_moves(dest);
        attackers |= knight_rays & board.pieces(Knight) & board.occupied_co(!board.side_to_move());

        let king_rays = get_king_moves(dest);
        attackers |= king_rays & board.pieces(King) & board.occupied_co(!board.side_to_move());

        attackers |= get_pawn_attacks(
            dest,
            board.side_to_move(),
            board.pieces(Pawn) & board.occupied_co(!board.side_to_move()),
        );

        attackers == BB_EMPTY
    }
}

impl PieceTypeTrait for KingType {
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
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckType,
    {
        let occupied = board.occupied();
        let color = board.side_to_move();
        let ksq = board.king_square(color);

        let mut moves = Self::pseudo_legals(ksq, color, *occupied, mask);

        let copy = moves;
        for dest in copy {
            if !KingType::legal_king_move(board, dest) {
                moves ^= BitBoard::from_square(dest);
            }
        }

        // If we are not in check, we may be able to castle.
        // We can do so iff:
        //  * the `SubBoard` structure says we can.
        //  * the squares between my king and my rook are empty.
        //  * no enemy pieces are attacking the squares between the king, and the kings
        //    destination square.
        //  ** This is determined by going to the left or right, and calling
        //     'legal_king_move' for that square.
        if !T::IN_CHECK {
            if board.my_castle_rights().has_kingside()
                && (occupied & board.my_castle_rights().kingside_squares(color)) == BB_EMPTY
            {
                let middle = ksq.wrapping_right();
                let right = middle.wrapping_right();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, right)
                {
                    moves ^= BitBoard::from_square(right);
                }
            }

            if board.my_castle_rights().has_queenside()
                && (occupied & board.my_castle_rights().queenside_squares(color)) == BB_EMPTY
            {
                let middle = ksq.wrapping_left();
                let left = middle.wrapping_left();
                if KingType::legal_king_move(board, middle)
                    && KingType::legal_king_move(board, left)
                {
                    moves ^= BitBoard::from_square(left);
                }
            }
        }
        if moves != BB_EMPTY {
            unsafe {
                move_list.push_unchecked(SquareAndBitBoard::new(ksq, moves, false));
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SquareAndBitBoard {
    square: Square,
    bitboard: BitBoard,
    promotion: bool,
}

impl SquareAndBitBoard {
    fn new(sq: Square, bb: BitBoard, promotion: bool) -> SquareAndBitBoard {
        SquareAndBitBoard {
            square: sq,
            bitboard: bb,
            promotion,
        }
    }
}

pub type MoveList = NoDrop<ArrayVec<SquareAndBitBoard, 18>>;

pub struct MoveGen {
    moves: MoveList,
    promotion_index: usize,
    iterator_mask: BitBoard,
    index: usize,
}

impl MoveGen {
    #[inline]
    fn enumerate_moves(board: &SubBoard) -> MoveList {
        let checkers = *board.checkers();
        let mask = !board.occupied_co(board.side_to_move());
        let mut movelist = NoDrop::new(ArrayVec::<SquareAndBitBoard, 18>::new());

        if checkers == BB_EMPTY {
            PawnType::legals::<NotInCheckType>(&mut movelist, board, mask);
            KnightType::legals::<NotInCheckType>(&mut movelist, board, mask);
            BishopType::legals::<NotInCheckType>(&mut movelist, board, mask);
            RookType::legals::<NotInCheckType>(&mut movelist, board, mask);
            QueenType::legals::<NotInCheckType>(&mut movelist, board, mask);
            KingType::legals::<NotInCheckType>(&mut movelist, board, mask);
        } else if checkers.popcnt() == 1 {
            PawnType::legals::<InCheckType>(&mut movelist, board, mask);
            KnightType::legals::<InCheckType>(&mut movelist, board, mask);
            BishopType::legals::<InCheckType>(&mut movelist, board, mask);
            RookType::legals::<InCheckType>(&mut movelist, board, mask);
            QueenType::legals::<InCheckType>(&mut movelist, board, mask);
            KingType::legals::<InCheckType>(&mut movelist, board, mask);
        } else {
            KingType::legals::<InCheckType>(&mut movelist, board, mask);
        }

        movelist
    }

    #[inline]
    pub fn new_legal(board: &SubBoard) -> MoveGen {
        MoveGen {
            moves: MoveGen::enumerate_moves(board),
            promotion_index: 0,
            iterator_mask: !BB_EMPTY,
            index: 0,
        }
    }

    pub fn remove_mask(&mut self, mask: BitBoard) {
        for x in 0..self.moves.len() {
            self.moves[x].bitboard &= !mask;
        }
    }

    pub fn remove_move(&mut self, chess_move: Move) -> bool {
        for x in 0..self.moves.len() {
            if self.moves[x].square == chess_move.get_source() {
                self.moves[x].bitboard &= !BitBoard::from_square(chess_move.get_dest());
                return true;
            }
        }
        false
    }

    pub fn set_iterator_mask(&mut self, mask: BitBoard) {
        self.iterator_mask = mask;
        self.index = 0;

        // the iterator portion of this struct relies on the invariant that
        // the bitboards at the beginning of the moves[] array are the only
        // ones used.  As a result, we must partition the list such that the
        // assumption is true.

        // first, find the first non-used moves index, and store that in i
        let mut i = 0;
        while i < self.moves.len() && self.moves[i].bitboard & self.iterator_mask != BB_EMPTY {
            i += 1;
        }

        // next, find each element past i where the moves are used, and store
        // that in i.  Then, increment i to point to a new unused slot.
        for j in (i + 1)..self.moves.len() {
            if self.moves[j].bitboard & self.iterator_mask != BB_EMPTY {
                let backup = self.moves[i];
                self.moves[i] = self.moves[j];
                self.moves[j] = backup;
                i += 1;
            }
        }
    }

    pub fn legal_quick(board: &SubBoard, chess_move: Move) -> bool {
        let piece = board.piece_type_at(chess_move.get_source()).unwrap();
        match piece {
            Rook => true,
            Bishop => true,
            Knight => true,
            Queen => true,
            Pawn => {
                if chess_move.get_source().get_file() != chess_move.get_dest().get_file()
                    && board.piece_type_at(chess_move.get_dest()).is_none()
                {
                    // en-passant
                    PawnType::legal_ep_move(board, chess_move.get_source(), chess_move.get_dest())
                } else {
                    true
                }
            }
            King => {
                let bb = between(chess_move.get_source(), chess_move.get_dest());
                if bb.popcnt() == 1 {
                    // castles
                    if !KingType::legal_king_move(board, bb.to_square()) {
                        false
                    } else {
                        KingType::legal_king_move(board, chess_move.get_dest())
                    }
                } else {
                    KingType::legal_king_move(board, chess_move.get_dest())
                }
            }
        }
    }

    pub fn perft_test(board: &SubBoard, depth: usize) -> usize {
        let iterable = MoveGen::new_legal(board);

        let mut result: usize = 0;
        if depth == 1 {
            iterable.len()
        } else {
            for m in iterable {
                let bresult = board.make_move_new(m);
                result += MoveGen::perft_test(&bresult, depth - 1);
            }
            result
        }
    }

    pub fn perft_test_piecewise(board: &SubBoard, depth: usize) -> usize {
        let mut iterable = MoveGen::new_legal(board);
    
        let targets = board.occupied_co(!board.side_to_move());
        let mut result: usize = 0;
    
        if depth == 1 {
            iterable.set_iterator_mask(*targets);
            result += iterable.len();
            iterable.set_iterator_mask(!targets);
            result += iterable.len();
            result
        } else {
            iterable.set_iterator_mask(*targets);
            for x in &mut iterable {
                let mut bresult = mem::MaybeUninit::<SubBoard>::uninit();
                unsafe {
                    board.make_move(x, &mut *bresult.as_mut_ptr());
                    result += MoveGen::perft_test(&*bresult.as_ptr(), depth - 1);
                }
            }
            iterable.set_iterator_mask(!BB_EMPTY);
            for x in &mut iterable {
                let mut bresult = mem::MaybeUninit::<SubBoard>::uninit();
                unsafe {
                    board.make_move(x, &mut *bresult.as_mut_ptr());
                    result += MoveGen::perft_test(&*bresult.as_ptr(), depth - 1);
                }
            }
            result
        }
    }
}

impl ExactSizeIterator for MoveGen {
    fn len(&self) -> usize {
        let mut result = 0;
        for i in 0..self.moves.len() {
            if self.moves[i].bitboard & self.iterator_mask == BB_EMPTY {
                break;
            }
            if self.moves[i].promotion {
                result += ((self.moves[i].bitboard & self.iterator_mask).popcnt() as usize)
                    * NUM_PROMOTION_PIECES;
            } else {
                result += (self.moves[i].bitboard & self.iterator_mask).popcnt() as usize;
            }
        }
        result
    }
}

impl Iterator for MoveGen {
    type Item = Move;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn next(&mut self) -> Option<Move> {
        if self.index >= self.moves.len()
            || self.moves[self.index].bitboard & self.iterator_mask == BB_EMPTY
        {
            // are we done?
            None
        } else if self.moves[self.index].promotion {
            let moves = &mut self.moves[self.index];

            let dest = (moves.bitboard & self.iterator_mask).to_square();

            // deal with potential promotions for this pawn
            let result = Move::new(
                moves.square,
                dest,
                Some(PROMOTION_PIECES[self.promotion_index]),
            );
            self.promotion_index += 1;
            if self.promotion_index >= NUM_PROMOTION_PIECES {
                moves.bitboard ^= BitBoard::from_square(dest);
                self.promotion_index = 0;
                if moves.bitboard & self.iterator_mask == BB_EMPTY {
                    self.index += 1;
                }
            }
            Some(result)
        } else {
            // not a promotion move, so its a 'normal' move as far as this function is concerned
            let moves = &mut self.moves[self.index];
            let dest = (moves.bitboard & self.iterator_mask).to_square();

            moves.bitboard ^= BitBoard::from_square(dest);
            if moves.bitboard & self.iterator_mask == BB_EMPTY {
                self.index += 1;
            }
            Some(Move::new(moves.square, dest, None))
        }
    }
}
