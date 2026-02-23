use timecat::*;

#[test]
fn test_piece_compression() {
    for piece_type in ALL_PIECE_TYPES {
        let compressed = piece_type.compress();
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(
            piece_type, decompressed,
            "Failed to compress and decompress piece type {:?}",
            piece_type
        );
    }

    for optional_piece_type in ALL_PIECE_TYPES
        .into_iter()
        .map(Some)
        .chain(std::iter::once(None))
    {
        let compressed = optional_piece_type.compress();
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(
            optional_piece_type, decompressed,
            "Failed to compress and decompress piece type {:?}",
            optional_piece_type
        );
    }
}

#[test]
fn test_square_compression() {
    for square in ALL_SQUARES {
        let compressed = square.compress();
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(
            square, decompressed,
            "Failed to compress and decompress square {:?}",
            square
        );
    }
}

#[test]
fn test_move_compression() {
    let all_moves = iproduct!(
        ALL_SQUARES.iter().copied(),
        ALL_SQUARES.iter().copied(),
        ALL_PIECE_TYPES
            .iter()
            .copied()
            .map(Some)
            .chain(std::iter::once(None)),
    )
    .map(|(from, to, piece_type)| unsafe { Move::new_unchecked(from, to, piece_type) })
    .collect_vec();
    for &move_ in &all_moves {
        let compressed = move_.compress();
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(
            move_, decompressed,
            "Failed to compress and decompress move {:?}",
            move_
        );
    }

    for optional_move in all_moves.into_iter().map(Some).chain(std::iter::once(None)) {
        let compressed = optional_move.compress();
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(
            optional_move, decompressed,
            "Failed to compress and decompress move {:?}",
            optional_move
        );
        let valid_move_or_null_move = ValidOrNullMove::from(optional_move);
        let compressed = optional_move.compress();
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(
            valid_move_or_null_move, decompressed,
            "Failed to compress and decompress move {:?}",
            valid_move_or_null_move
        );
    }
}
