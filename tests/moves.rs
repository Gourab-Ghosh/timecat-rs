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
fn test_uci_parsing() {
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
