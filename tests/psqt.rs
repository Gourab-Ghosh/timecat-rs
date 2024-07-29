use timecat::*;

#[test]
fn check_index_choosing() {
    for piece in ALL_PIECES {
        for is_endgame in [false, true] {
            for mut square in ALL_SQUARES {
                let output = if is_endgame {
                    get_psqt_score_index_endgame(piece, square)
                } else {
                    get_psqt_score_index_opening(piece, square)
                };

                if piece.get_color() == White {
                    square = square.horizontal_mirror();
                }
                let mut expected = 128 * piece.get_piece_type().to_index() + square.to_index();
                if is_endgame {
                    expected += 64;
                }

                assert_eq!(
                    output, expected,
                    "Expected {expected} for Piece {piece} and Square {square} but got {output}"
                );
            }
        }
    }
}
