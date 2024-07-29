use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Default, Debug)]
pub struct EvaluatorNonNNUE;

impl EvaluatorNonNNUE {
    pub fn new(_: &SubBoard) -> Self {
        Self
    }

    pub fn slow_evaluate(sub_board: &SubBoard) -> Score {
        Self::default().evaluate(sub_board)
    }
}

impl PositionEvaluation for EvaluatorNonNNUE {
    fn evaluate(&mut self, sub_board: &SubBoard) -> Score {
        let material_score = sub_board.get_material_score();
        let mut score = material_score;
        for (piece, square) in sub_board.iter() {
            let psqt_score = get_psqt_score(piece, square, sub_board.get_material_score_abs());
            score += if piece.get_color() == White {
                psqt_score
            } else {
                -psqt_score
            } as Score;
        }
        score
    }
}
