use super::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum ChessError {
    InvalidRank,
    InvalidFile,
    InvalidSquare,
    InvalidUciMove,
    InvalidFen { fen: String },
    InvalidBoard,
}
