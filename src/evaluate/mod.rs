use super::*;

#[cfg(feature = "nnue_reader")]
pub mod evaluate_nnue;
pub mod evaluate_non_nnue;

#[cfg(feature = "nnue_reader")]
pub use evaluate_nnue::*;
pub use evaluate_non_nnue::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Default, Debug)]
pub struct Evaluator {
    #[cfg(feature = "inbuilt_nnue")]
    inner_evaluator: EvaluatorNNUE,
    #[cfg(not(feature = "inbuilt_nnue"))]
    inner_evaluator: EvaluatorNonNNUE,
}

impl Evaluator {
    pub fn new(minimum_board: &MinimumBoard) -> Self {
        Self {
            #[cfg(feature = "inbuilt_nnue")]
            inner_evaluator: EvaluatorNNUE::new(minimum_board),
            #[cfg(not(feature = "inbuilt_nnue"))]
            inner_evaluator: EvaluatorNonNNUE::new(minimum_board),
        }
    }

    pub fn slow_evaluate(minimum_board: &MinimumBoard) -> Score {
        #[cfg(feature = "inbuilt_nnue")]
        {
            EvaluatorNNUE::slow_evaluate(minimum_board)
        }
        #[cfg(not(feature = "inbuilt_nnue"))]
        {
            EvaluatorNonNNUE::slow_evaluate(minimum_board)
        }
    }
}

impl PositionEvaluation for Evaluator {
    fn evaluate(&mut self, minimum_board: &MinimumBoard) -> Score {
        self.inner_evaluator.evaluate(minimum_board)
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

    fn evaluate_flipped(&mut self, minimum_board: &MinimumBoard) -> Score {
        self.inner_evaluator.evaluate_flipped(minimum_board)
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
