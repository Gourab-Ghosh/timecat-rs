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
    pub fn new(position: &BoardPosition) -> Self {
        Self {
            #[cfg(feature = "inbuilt_nnue")]
            inner_evaluator: EvaluatorNNUE::new(position),
            #[cfg(not(feature = "inbuilt_nnue"))]
            inner_evaluator: EvaluatorNonNNUE::new(position),
        }
    }

    pub fn slow_evaluate(position: &BoardPosition) -> Score {
        #[cfg(feature = "inbuilt_nnue")]
        {
            EvaluatorNNUE::slow_evaluate(position)
        }
        #[cfg(not(feature = "inbuilt_nnue"))]
        {
            EvaluatorNonNNUE::slow_evaluate(position)
        }
    }
}

impl PositionEvaluation for Evaluator {
    fn evaluate(&mut self, position: &BoardPosition) -> Score {
        self.inner_evaluator.evaluate(position)
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

    fn evaluate_flipped(&mut self, position: &BoardPosition) -> Score {
        self.inner_evaluator.evaluate_flipped(position)
    }

    fn evaluate_checkmate_in(&mut self, mate_distance: Ply) -> Score {
        self.inner_evaluator.evaluate_checkmate_in(mate_distance)
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
