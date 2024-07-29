use super::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Default, Debug)]
pub struct Evaluator {
    #[cfg(feature = "inbuilt_nnue")]
    inner_evaluator: EvaluatorNNUE,
    #[cfg(not(feature = "inbuilt_nnue"))]
    inner_evaluator: EvaluatorNonNNUE,
}

impl Evaluator {
    pub fn new(sub_board: &SubBoard) -> Self {
        Self {
            #[cfg(feature = "inbuilt_nnue")]
            inner_evaluator: EvaluatorNNUE::new(sub_board),
            #[cfg(not(feature = "inbuilt_nnue"))]
            inner_evaluator: EvaluatorNonNNUE::new(sub_board),
        }
    }

    pub fn slow_evaluate(sub_board: &SubBoard) -> Score {
        #[cfg(feature = "inbuilt_nnue")]
        {
            EvaluatorNNUE::slow_evaluate(sub_board)
        }
        #[cfg(not(feature = "inbuilt_nnue"))]
        {
            EvaluatorNonNNUE::slow_evaluate(sub_board)
        }
    }
}

impl PositionEvaluation for Evaluator {
    fn evaluate(&mut self, sub_board: &SubBoard) -> Score {
        self.inner_evaluator.evaluate(sub_board)
    }

    fn reset_variables(&mut self) {
        self.inner_evaluator.reset_variables()
    }

    fn clear(&mut self) {
        self.inner_evaluator.clear()
    }

    fn print_info(&self) {
        self.inner_evaluator.print_info()
    }

    fn evaluate_flipped(&mut self, sub_board: &SubBoard) -> Score {
        self.inner_evaluator.evaluate_flipped(sub_board)
    }

    fn evaluate_checkmate(&mut self, mate_distance: usize) -> Score {
        self.inner_evaluator.evaluate_checkmate(mate_distance)
    }
}

impl Deref for Evaluator {
    #[cfg(feature = "inbuilt_nnue")]
    type Target = EvaluatorNNUE;
    #[cfg(not(feature = "inbuilt_nnue"))]
    type Target = EvaluatorNonNNUE;

    fn deref(&self) -> &Self::Target {
        &self.inner_evaluator
    }
}

impl DerefMut for Evaluator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_evaluator
    }
}
