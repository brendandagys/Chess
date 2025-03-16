use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn possible_moves(
        &self,
        position: &crate::types::board::Position,
        board: &crate::types::board::Board,
    ) -> Vec<crate::types::board::Position> {
        match self.piece_type {
            PieceType::Pawn => {
                let mut moves = Vec::new();
                let direction = match self.color {
                    Color::White => -1isize,
                    Color::Black => 1isize,
                };
                let new_row = position.row as isize + direction;
                if crate::types::board::is_on_board(new_row, position.col as isize) {
                    if board.squares[new_row as usize][position.col].is_none() {
                        moves.push(crate::types::board::Position {
                            row: new_row as usize,
                            col: position.col,
                        });
                    }
                }
                for col_offset in &[-1, 1] {
                    let capture_col = position.col as isize + col_offset;
                    if crate::types::board::is_on_board(new_row, capture_col) {
                        if let Some(target_piece) =
                            board.squares[new_row as usize][capture_col as usize]
                        {
                            if target_piece.color != self.color {
                                moves.push(crate::types::board::Position {
                                    row: new_row as usize,
                                    col: capture_col as usize,
                                });
                            }
                        }
                    }
                }
                moves
            }
            PieceType::Knight => {
                let offsets = [
                    (-2, -1),
                    (-2, 1),
                    (-1, -2),
                    (-1, 2),
                    (1, -2),
                    (1, 2),
                    (2, -1),
                    (2, 1),
                ];
                let mut moves = Vec::new();
                for (dr, dc) in offsets.iter() {
                    let new_row = position.row as isize + dr;
                    let new_col = position.col as isize + dc;
                    if crate::types::board::is_on_board(new_row, new_col) {
                        if let Some(p) = board.squares[new_row as usize][new_col as usize] {
                            if p.color != self.color {
                                moves.push(crate::types::board::Position {
                                    row: new_row as usize,
                                    col: new_col as usize,
                                });
                            }
                        } else {
                            moves.push(crate::types::board::Position {
                                row: new_row as usize,
                                col: new_col as usize,
                            });
                        }
                    }
                }
                moves
            }
            PieceType::Bishop => {
                let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
                let mut moves = Vec::new();
                for (dr, dc) in directions.iter() {
                    let mut r = position.row as isize;
                    let mut c = position.col as isize;
                    loop {
                        r += dr;
                        c += dc;
                        if !crate::types::board::is_on_board(r, c) {
                            break;
                        }
                        if let Some(p) = board.squares[r as usize][c as usize] {
                            if p.color != self.color {
                                moves.push(crate::types::board::Position {
                                    row: r as usize,
                                    col: c as usize,
                                });
                            }
                            break;
                        } else {
                            moves.push(crate::types::board::Position {
                                row: r as usize,
                                col: c as usize,
                            });
                        }
                    }
                }
                moves
            }
            PieceType::Rook => {
                let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                let mut moves = Vec::new();
                for (dr, dc) in directions.iter() {
                    let mut r = position.row as isize;
                    let mut c = position.col as isize;
                    loop {
                        r += dr;
                        c += dc;
                        if !crate::types::board::is_on_board(r, c) {
                            break;
                        }
                        if let Some(p) = board.squares[r as usize][c as usize] {
                            if p.color != self.color {
                                moves.push(crate::types::board::Position {
                                    row: r as usize,
                                    col: c as usize,
                                });
                            }
                            break;
                        } else {
                            moves.push(crate::types::board::Position {
                                row: r as usize,
                                col: c as usize,
                            });
                        }
                    }
                }
                moves
            }
            PieceType::Queen => {
                let directions = [
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                ];
                let mut moves = Vec::new();
                for (dr, dc) in directions.iter() {
                    let mut r = position.row as isize;
                    let mut c = position.col as isize;
                    loop {
                        r += dr;
                        c += dc;
                        if !crate::types::board::is_on_board(r, c) {
                            break;
                        }
                        if let Some(p) = board.squares[r as usize][c as usize] {
                            if p.color != self.color {
                                moves.push(crate::types::board::Position {
                                    row: r as usize,
                                    col: c as usize,
                                });
                            }
                            break;
                        } else {
                            moves.push(crate::types::board::Position {
                                row: r as usize,
                                col: c as usize,
                            });
                        }
                    }
                }
                moves
            }
            PieceType::King => {
                let offsets = [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ];
                let mut moves = Vec::new();
                for (dr, dc) in offsets.iter() {
                    let new_row = position.row as isize + dr;
                    let new_col = position.col as isize + dc;
                    if crate::types::board::is_on_board(new_row, new_col) {
                        if let Some(p) = board.squares[new_row as usize][new_col as usize] {
                            if p.color != self.color {
                                moves.push(crate::types::board::Position {
                                    row: new_row as usize,
                                    col: new_col as usize,
                                });
                            }
                        } else {
                            moves.push(crate::types::board::Position {
                                row: new_row as usize,
                                col: new_col as usize,
                            });
                        }
                    }
                }
                moves
            }
        }
    }
}
