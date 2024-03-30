use super::*;

trait PieceMoves {
    fn is(piece: PieceType) -> bool;
    fn into_piece() -> PieceType;
    fn pseudo_legals(src: Square, color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard;
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = board.occupied();
        let color = board.turn();
        let my_pieces = board.occupied_co(color);
        let ksq = board.get_king_square(color);

        let pieces = board.get_piece_mask(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.get_checkers();

        let check_mask = if T::IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !BB_EMPTY
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
                    Self::pseudo_legals(src, color, occupied, mask) & line(src, ksq);
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
    fn legal_ep_move(board: &SubBoard, source: Square, dest: Square) -> bool {
        let occupied = board.occupied()
            ^ board
                .ep_square()
                .unwrap()
                .wrapping_backward(board.turn())
                .to_bitboard()
            ^ source.to_bitboard()
            ^ dest.to_bitboard();

        let ksq = (board.get_piece_mask(King) & board.occupied_co(board.turn())).to_square();

        let rooks = (board.get_piece_mask(Rook) ^ board.get_piece_mask(Queen))
            & board.occupied_co(!board.turn());

        if !(get_rook_rays(ksq) & rooks).is_empty()
            && !(get_rook_moves(ksq, occupied) & rooks).is_empty()
        {
            return false;
        }

        let bishops = (board.get_piece_mask(Bishop) ^ board.get_piece_mask(Queen))
            & board.occupied_co(!board.turn());

        if !(get_bishop_rays(ksq) & bishops).is_empty()
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

    #[inline(always)]
    fn pseudo_legals(src: Square, color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_pawn_moves(src, color, occupied) & mask
    }

    #[inline(always)]
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = board.occupied();
        let color = board.turn();
        let my_pieces = board.occupied_co(color);
        let ksq = board.get_king_square(color);

        let pieces = board.get_piece_mask(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.get_checkers();

        let check_mask = if T::IN_CHECK {
            between(checkers.to_square(), ksq) ^ checkers
        } else {
            !BB_EMPTY
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
                    Self::pseudo_legals(src, color, occupied, mask) & line(ksq, src);
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

        if let Some(dest) = board.ep_square() {
            let dest_rank = dest.get_rank();
            let rank_bb = get_rank_bb(if dest_rank.to_int() > 3 {
                dest_rank.down()
            } else {
                dest_rank.up()
            });
            let files_bb = get_adjacent_files(dest.get_file());
            for src in rank_bb & files_bb & pieces {
                if PawnMoves::legal_ep_move(board, src, dest) {
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

    #[inline(always)]
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

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_knight_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = board.occupied();
        let color = board.turn();
        let my_pieces = board.occupied_co(color);
        let ksq = board.get_king_square(color);

        let pieces = board.get_piece_mask(Self::into_piece()) & my_pieces;
        let pinned = board.pinned();
        let checkers = board.get_checkers();

        if T::IN_CHECK {
            let check_mask = between(checkers.to_square(), ksq) ^ checkers;

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

    #[inline(always)]
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

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, occupied: BitBoard, mask: BitBoard) -> BitBoard {
        (get_rook_moves(src, occupied) ^ get_bishop_moves(src, occupied)) & mask
    }
}

impl KingMoves {
    #[inline(always)]
    fn legal_king_move(board: &SubBoard, dest: Square) -> bool {
        let occupied = board.occupied()
            ^ (board.get_piece_mask(King) & board.occupied_co(board.turn()))
            | dest.to_bitboard();

        let rooks = (board.get_piece_mask(Rook) ^ board.get_piece_mask(Queen))
            & board.occupied_co(!board.turn());

        let mut attackers = get_rook_moves(dest, occupied) & rooks;

        let bishops = (board.get_piece_mask(Bishop) ^ board.get_piece_mask(Queen))
            & board.occupied_co(!board.turn());

        attackers |= get_bishop_moves(dest, occupied) & bishops;

        let knight_rays = get_knight_moves(dest);

        // Using ^ because knight square_and_bitboard_array bitboard do not collide with rook and bishop square_and_bitboard_array bitboard
        attackers ^= knight_rays & board.get_piece_mask(Knight) & board.occupied_co(!board.turn());

        let king_rays = get_king_moves(dest);
        attackers |= king_rays & board.get_piece_mask(King) & board.occupied_co(!board.turn());

        attackers |= get_pawn_attacks(
            dest,
            board.turn(),
            board.get_piece_mask(Pawn) & board.occupied_co(!board.turn()),
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

    #[inline(always)]
    fn pseudo_legals(src: Square, _color: Color, _occupied: BitBoard, mask: BitBoard) -> BitBoard {
        get_king_moves(src) & mask
    }

    #[inline(always)]
    fn legals<T>(move_list: &mut MoveList, board: &SubBoard, mask: BitBoard)
    where
        T: CheckMoves,
    {
        let occupied = board.occupied();
        let color = board.turn();
        let ksq = board.get_king_square(color);

        let mut square_and_bitboard_array = Self::pseudo_legals(ksq, color, occupied, mask);

        let copy = square_and_bitboard_array;
        for dest in copy {
            if !KingMoves::legal_king_move(board, dest) {
                square_and_bitboard_array ^= dest.to_bitboard();
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
                && (occupied & board.my_castle_rights().kingside_squares(color)).is_empty()
            {
                let middle = ksq.wrapping_right();
                let right = middle.wrapping_right();
                if KingMoves::legal_king_move(board, middle)
                    && KingMoves::legal_king_move(board, right)
                {
                    square_and_bitboard_array ^= right.to_bitboard();
                }
            }

            if board.my_castle_rights().has_queenside()
                && (occupied & board.my_castle_rights().queenside_squares(color)).is_empty()
            {
                let middle = ksq.wrapping_left();
                let left = middle.wrapping_left();
                if KingMoves::legal_king_move(board, middle)
                    && KingMoves::legal_king_move(board, left)
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

pub struct MoveGenerator {
    square_and_bitboard_array: MoveList,
    promotion_index: usize,
    iterator_mask: BitBoard,
    index: usize,
}

impl MoveGenerator {
    #[inline(always)]
    fn enumerate_moves(board: &SubBoard) -> MoveList {
        let checkers = board.get_checkers();
        let mask = !board.occupied_co(board.turn());
        let mut move_list = ArrayVec::new();

        if checkers.is_empty() {
            PawnMoves::legals::<NotInCheckMoves>(&mut move_list, board, mask);
            KnightMoves::legals::<NotInCheckMoves>(&mut move_list, board, mask);
            BishopMoves::legals::<NotInCheckMoves>(&mut move_list, board, mask);
            RookMoves::legals::<NotInCheckMoves>(&mut move_list, board, mask);
            QueenMoves::legals::<NotInCheckMoves>(&mut move_list, board, mask);
            KingMoves::legals::<NotInCheckMoves>(&mut move_list, board, mask);
        } else if checkers.popcnt() == 1 {
            PawnMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
            KnightMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
            BishopMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
            RookMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
            QueenMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
            KingMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
        } else {
            KingMoves::legals::<InCheckMoves>(&mut move_list, board, mask);
        }

        move_list
    }

    #[inline(always)]
    pub fn new_legal(board: &SubBoard) -> MoveGenerator {
        MoveGenerator {
            square_and_bitboard_array: MoveGenerator::enumerate_moves(board),
            promotion_index: 0,
            iterator_mask: !BB_EMPTY,
            index: 0,
        }
    }

    pub fn remove_mask(&mut self, mask: BitBoard) {
        for x in 0..self.square_and_bitboard_array.len() {
            get_item_unchecked_mut!(self.square_and_bitboard_array, x).bitboard &= !mask;
        }
    }

    pub fn remove_move(&mut self, move_: Move) -> bool {
        for x in 0..self.square_and_bitboard_array.len() {
            let square_and_bitboard = get_item_unchecked_mut!(self.square_and_bitboard_array, x);
            if square_and_bitboard.square == move_.get_source() {
                square_and_bitboard.bitboard &= !move_.get_dest().to_bitboard();
                return true;
            }
        }
        false
    }

    pub fn get_iterator_mask(&self) -> BitBoard {
        self.iterator_mask
    }

    pub fn set_iterator_mask(&mut self, mask: BitBoard) {
        self.iterator_mask = mask;
        self.index = 0;

        // the iterator portion of this struct relies on the invariant that
        // the bitboards at the beginning of the square_and_bitboard_array[] array are the only
        // ones used.  As a result, we must partition the list such that the
        // assumption is true.

        // first, find the first non-used square_and_bitboard_array index, and store that in i
        let mut i = 0;
        while i < self.square_and_bitboard_array.len()
            && !(get_item_unchecked!(self.square_and_bitboard_array, i).bitboard
                & self.iterator_mask)
                .is_empty()
        {
            i += 1;
        }

        // next, find each element past i where the square_and_bitboard_array are used, and store
        // that in i.  Then, increment i to point to a new unused slot.
        for j in (i + 1)..self.square_and_bitboard_array.len() {
            if !(get_item_unchecked!(self.square_and_bitboard_array, j).bitboard
                & self.iterator_mask)
                .is_empty()
            {
                // unsafe { self.square_and_bitboard_array.swap_unchecked(i, j) };
                self.square_and_bitboard_array.swap(i, j);
                i += 1;
            }
        }
    }

    pub fn is_legal_quick(board: &SubBoard, move_: Move) -> bool {
        let piece = board.piece_type_at(move_.get_source()).unwrap();
        match piece {
            Rook => true,
            Bishop => true,
            Knight => true,
            Queen => true,
            Pawn => {
                if move_.get_source().get_file() != move_.get_dest().get_file()
                    && board.piece_type_at(move_.get_dest()).is_none()
                {
                    // en-passant
                    PawnMoves::legal_ep_move(board, move_.get_source(), move_.get_dest())
                } else {
                    true
                }
            }
            King => {
                let bb = between(move_.get_source(), move_.get_dest());
                if bb.popcnt() == 1 {
                    // castles
                    if !KingMoves::legal_king_move(board, bb.to_square()) {
                        false
                    } else {
                        KingMoves::legal_king_move(board, move_.get_dest())
                    }
                } else {
                    KingMoves::legal_king_move(board, move_.get_dest())
                }
            }
        }
    }

    pub fn perft_test(board: &SubBoard, depth: usize) -> usize {
        let iterable = MoveGenerator::new_legal(board);

        let mut result: usize = 0;
        if depth == 1 {
            iterable.len()
        } else {
            for m in iterable {
                let board_result = board.make_move_new(m);
                result += MoveGenerator::perft_test(&board_result, depth - 1);
            }
            result
        }
    }

    pub fn perft_test_piecewise(board: &SubBoard, depth: usize) -> usize {
        let mut iterable = MoveGenerator::new_legal(board);

        let targets = board.occupied_co(!board.turn());
        let mut result: usize = 0;

        if depth == 1 {
            iterable.set_iterator_mask(targets);
            result += iterable.len();
            iterable.set_iterator_mask(!targets);
            result += iterable.len();
            result
        } else {
            iterable.set_iterator_mask(targets);
            for x in &mut iterable {
                let mut board_result = mem::MaybeUninit::<SubBoard>::uninit();
                unsafe {
                    board.make_move(x, &mut *board_result.as_mut_ptr());
                    result += MoveGenerator::perft_test(&*board_result.as_ptr(), depth - 1);
                }
            }
            iterable.set_iterator_mask(!BB_EMPTY);
            for x in &mut iterable {
                let mut board_result = mem::MaybeUninit::<SubBoard>::uninit();
                unsafe {
                    board.make_move(x, &mut *board_result.as_mut_ptr());
                    result += MoveGenerator::perft_test(&*board_result.as_ptr(), depth - 1);
                }
            }
            result
        }
    }
}

impl ExactSizeIterator for MoveGenerator {
    fn len(&self) -> usize {
        let mut result = 0;
        for i in 0..self.square_and_bitboard_array.len() {
            let square_and_bitboard = get_item_unchecked!(self.square_and_bitboard_array, i);
            let bitboard_and_iterator_mask = square_and_bitboard.bitboard & self.iterator_mask;
            if bitboard_and_iterator_mask.is_empty() {
                break;
            }
            if square_and_bitboard.promotion {
                result += (bitboard_and_iterator_mask.popcnt() as usize) * NUM_PROMOTION_PIECES;
            } else {
                result += bitboard_and_iterator_mask.popcnt() as usize;
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
        let square_and_bitboard =
            get_item_unchecked_mut!(self.square_and_bitboard_array, self.index);

        if self.index >= square_and_bitboard_array_len
            || (square_and_bitboard.bitboard & self.iterator_mask).is_empty()
        {
            // are we done?
            None
        } else if square_and_bitboard.promotion {
            let dest = (square_and_bitboard.bitboard & self.iterator_mask).to_square();

            // deal with potential promotions for this pawn
            let result = Move::new(
                square_and_bitboard.square,
                dest,
                Some(*get_item_unchecked!(PROMOTION_PIECES, self.promotion_index)),
            );
            self.promotion_index += 1;
            if self.promotion_index >= NUM_PROMOTION_PIECES {
                square_and_bitboard.bitboard ^= dest.to_bitboard();
                self.promotion_index = 0;
                if (square_and_bitboard.bitboard & self.iterator_mask).is_empty() {
                    self.index += 1;
                }
            }
            Some(result)
        } else {
            // not a promotion move, so its a 'normal' move as far as this function is concerned
            let dest = (square_and_bitboard.bitboard & self.iterator_mask).to_square();

            square_and_bitboard.bitboard ^= dest.to_bitboard();
            if (square_and_bitboard.bitboard & self.iterator_mask).is_empty() {
                self.index += 1;
            }
            Some(Move::new(square_and_bitboard.square, dest, None))
        }
    }
}
