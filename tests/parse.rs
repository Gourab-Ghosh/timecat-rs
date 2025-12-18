use itertools::*;
use std::str::FromStr;
use timecat::*;

macro_rules! assert_error {
    ($uci: expr, $error: pat) => {
        for uci in [$uci.to_lowercase(), $uci.to_uppercase()] {
            assert!(
                matches!(Move::from_str(&uci).unwrap_err(), $error),
                "assert_error failed for {}",
                $uci
            );
        }
    };
}

#[test]
fn test_move_uci_parsing() {
    izip!(
        ALL_SQUARES,
        ALL_SQUARES,
        ALL_PIECE_TYPES
            .iter()
            .copied()
            .map(Some)
            .chain(std::iter::once(None))
    )
    .for_each(|(source, dest, promotion)| {
        if source == dest {
            return;
        }
        for piece_type in [PieceType::Pawn, PieceType::King] {
            assert_error!(
                format!("{}{}{}", source, dest, piece_type).to_lowercase(),
                TimecatError::InvalidPromotion { .. }
            );
        }
        let move_ = Move::new(source, dest, promotion).unwrap();
        let uci_string = move_.uci();
        let parsed_move = Move::from_str(&uci_string).unwrap();
        assert_eq!(
            move_, parsed_move,
            "Failed to parse UCI string {} back to move",
            uci_string
        );
    });

    izip!(ALL_SQUARES, ALL_SQUARES, [King, Pawn],).for_each(|(source, dest, promotion)| {
        if source == dest {
            return;
        }
        assert_error!(
            format!("{}{}{}", source, dest, promotion).to_lowercase(),
            TimecatError::InvalidPromotion { .. }
        );
    });

    assert_eq!(ValidOrNullMove::NullMove.uci(), "0000");
    assert_eq!(
        ValidOrNullMove::NullMove,
        ValidOrNullMove::from_str("0000").unwrap()
    );
    assert_eq!(
        ValidOrNullMove::NullMove,
        ValidOrNullMove::from_str("--").unwrap()
    );

    for square in ALL_SQUARES {
        assert_error!(
            format!("{}{}", square, square).to_lowercase(),
            TimecatError::SameSourceAndDestination { .. }
        );
        for promotion in ALL_PIECE_TYPES {
            assert_error!(
                format!("{}{}{}", square, square, promotion).to_lowercase(),
                TimecatError::SameSourceAndDestination { .. }
            );
        }
    }

    assert_error!("Hello World!", TimecatError::InvalidUciMoveString { .. });
}

macro_rules! generate_parsing_functions {
    ($func_name:ident, $all_values:expr) => {
        #[test]
        fn $func_name() {
            for item in $all_values {
                let item_str = item.to_string();
                let parsed_item = item_str.parse().unwrap();
                assert_eq!(
                    item,
                    parsed_item,
                    "Failed to parse {} back to {}",
                    item_str,
                    stringify!($func_name)
                );
            }
        }
    };
}

generate_parsing_functions!(test_file_parsing, ALL_FILES);
generate_parsing_functions!(test_rank_parsing, ALL_RANKS);
generate_parsing_functions!(test_square_parsing, ALL_SQUARES);
generate_parsing_functions!(test_piece_type_parsing, ALL_PIECE_TYPES);
generate_parsing_functions!(test_piece_parsing, ALL_PIECES);
generate_parsing_functions!(test_color_parsing, ALL_COLORS);

#[test]
fn test_castling_right_parsing() {
    for color in ALL_COLORS {
        for castle_right in [
            CastleRights::None,
            CastleRights::KingSide,
            CastleRights::QueenSide,
            CastleRights::Both,
        ] {
            let castle_right_str = castle_right.to_string(color);
            assert!(
                if color == Color::White {
                    castle_right_str.chars().all(|c| c.is_uppercase())
                } else {
                    castle_right_str.chars().all(|c| c.is_lowercase())
                },
                "Castle right string {} has incorrect casing for color {}",
                castle_right_str,
                color
            );
            let parsed_castle_right = castle_right_str.parse().unwrap();
            assert_eq!(
                castle_right,
                parsed_castle_right,
                "Failed to parse {} back to {}",
                castle_right_str,
                stringify!($func_name)
            );
        }
    }
}
