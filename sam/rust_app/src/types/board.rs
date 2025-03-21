use crate::types::pieces::Piece;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
}

pub fn is_on_board(row: isize, col: isize) -> bool {
    row >= 0 && row < 8 && col >= 0 && col < 8
}
