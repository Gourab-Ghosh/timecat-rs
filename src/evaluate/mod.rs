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
    pub fn new(mini_board: &MiniBoard) -> Self {
        Self {
            #[cfg(feature = "inbuilt_nnue")]
            inner_evaluator: EvaluatorNNUE::new(mini_board),
            #[cfg(not(feature = "inbuilt_nnue"))]
            inner_evaluator: EvaluatorNonNNUE::new(mini_board),
        }
    }

    pub fn slow_evaluate(mini_board: &MiniBoard) -> Score {
        #[cfg(feature = "inbuilt_nnue")]
        {
            EvaluatorNNUE::slow_evaluate(mini_board)
        }
        #[cfg(not(feature = "inbuilt_nnue"))]
        {
            EvaluatorNonNNUE::slow_evaluate(mini_board)
        }
    }
}

impl PositionEvaluation for Evaluator {
    fn evaluate(&mut self, mini_board: &MiniBoard) -> Score {
        self.inner_evaluator.evaluate(mini_board)
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

    fn evaluate_flipped(&mut self, mini_board: &MiniBoard) -> Score {
        self.inner_evaluator.evaluate_flipped(mini_board)
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
