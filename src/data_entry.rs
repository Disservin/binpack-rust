use crate::chess::{position::Position, r#move::Move};

/// A single training data entry.
#[derive(Debug, Clone, Copy)]
pub struct TrainingDataEntry {
    /// The position of the board.
    pub pos: Position,
    /// The which will be played on this position.
    pub mv: Move,
    /// The score of the position.
    pub score: i16,
    /// The game ply of the position.
    pub ply: u16,
    /// The game result of the position.
    /// 1, 0, -1 for white win, draw, white loss respectively.
    pub result: i16,
}
