use serde::{Deserialize, Serialize};

// Color of a chess piece
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

// Possible chess piece types
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

// Represents a chess piece
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

// Board coordinates
#[derive(Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

// Represents the board as an 8x8 array of optional pieces
#[derive(Serialize, Deserialize)]
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
}

// Tracks the state of a game
#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub current_turn: Color,
    pub move_history: Vec<(Position, Position)>,
    pub is_over: bool,
    pub winner: Option<Color>,
    pub game_id: String,
}

pub fn is_on_board(row: isize, col: isize) -> bool {
    row >= 0 && row < 8 && col >= 0 && col < 8
}

impl GameState {
    pub fn new(game_id: String) -> Self {
        let mut squares = [[None; 8]; 8];

        // Black major pieces
        squares[0][0] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        });
        squares[0][1] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::Black,
        });
        squares[0][2] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::Black,
        });
        squares[0][3] = Some(Piece {
            piece_type: PieceType::Queen,
            color: Color::Black,
        });
        squares[0][4] = Some(Piece {
            piece_type: PieceType::King,
            color: Color::Black,
        });
        squares[0][5] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::Black,
        });
        squares[0][6] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::Black,
        });
        squares[0][7] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        });

        // Black pawns
        for col in 0..8 {
            squares[1][col] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
            });
        }

        // White pawns
        for col in 0..8 {
            squares[6][col] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
            });
        }

        // White major pieces
        squares[7][0] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::White,
        });
        squares[7][1] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::White,
        });
        squares[7][2] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::White,
        });
        squares[7][3] = Some(Piece {
            piece_type: PieceType::Queen,
            color: Color::White,
        });
        squares[7][4] = Some(Piece {
            piece_type: PieceType::King,
            color: Color::White,
        });
        squares[7][5] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::White,
        });
        squares[7][6] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::White,
        });
        squares[7][7] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::White,
        });

        GameState {
            board: Board { squares },
            current_turn: Color::White,
            move_history: Vec::new(),
            is_over: false,
            winner: None,
            game_id,
        }
    }
}

impl Piece {
    fn possible_moves(&self, position: &Position, board: &Board) -> Vec<Position> {
        match self.piece_type {
            PieceType::Pawn => {
                let mut moves = Vec::new();
                let direction = match self.color {
                    Color::White => -1isize,
                    Color::Black => 1isize,
                };
                let new_row = position.row as isize + direction;
                // One-step forward
                if is_on_board(new_row, position.col as isize) {
                    if board.squares[new_row as usize][position.col].is_none() {
                        moves.push(Position {
                            row: new_row as usize,
                            col: position.col,
                        });
                    }
                }
                // Simple capture moves (diagonals)
                for col_offset in &[-1, 1] {
                    let capture_col = position.col as isize + col_offset;
                    if is_on_board(new_row, capture_col) {
                        if let Some(target_piece) =
                            board.squares[new_row as usize][capture_col as usize]
                        {
                            if target_piece.color != self.color {
                                moves.push(Position {
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
                    if is_on_board(new_row, new_col) {
                        if let Some(p) = board.squares[new_row as usize][new_col as usize] {
                            if p.color != self.color {
                                moves.push(Position {
                                    row: new_row as usize,
                                    col: new_col as usize,
                                });
                            }
                        } else {
                            moves.push(Position {
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
                        if !is_on_board(r, c) {
                            break;
                        }
                        if let Some(p) = board.squares[r as usize][c as usize] {
                            if p.color != self.color {
                                moves.push(Position {
                                    row: r as usize,
                                    col: c as usize,
                                });
                            }
                            break;
                        } else {
                            moves.push(Position {
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
                        if !is_on_board(r, c) {
                            break;
                        }
                        if let Some(p) = board.squares[r as usize][c as usize] {
                            if p.color != self.color {
                                moves.push(Position {
                                    row: r as usize,
                                    col: c as usize,
                                });
                            }
                            break;
                        } else {
                            moves.push(Position {
                                row: r as usize,
                                col: c as usize,
                            });
                        }
                    }
                }
                moves
            }
            PieceType::Queen => {
                // Rook + Bishop combined
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
                        if !is_on_board(r, c) {
                            break;
                        }
                        if let Some(p) = board.squares[r as usize][c as usize] {
                            if p.color != self.color {
                                moves.push(Position {
                                    row: r as usize,
                                    col: c as usize,
                                });
                            }
                            break;
                        } else {
                            moves.push(Position {
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
                    if is_on_board(new_row, new_col) {
                        if let Some(p) = board.squares[new_row as usize][new_col as usize] {
                            if p.color != self.color {
                                moves.push(Position {
                                    row: new_row as usize,
                                    col: new_col as usize,
                                });
                            }
                        } else {
                            moves.push(Position {
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
