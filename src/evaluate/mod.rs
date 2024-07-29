use super::*;

pub mod evaluate;
#[cfg(feature = "nnue_reader")]
pub mod evaluate_nnue;
pub mod evaluate_non_nnue;

#[cfg(feature = "nnue_reader")]
pub use evaluate_nnue::*;
pub use evaluate_non_nnue::*;

pub use evaluate::*;
