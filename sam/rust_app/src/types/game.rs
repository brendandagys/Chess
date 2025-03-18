use crate::types::board::{Board, Position};
use crate::types::pieces::{Color, Piece, PieceType};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GameEnding {
    Checkmate(Color),
    Resignation(Color),
    OutOfTime(Color),
    Stalemate,
    DrawByThreefoldRepetition,
    DrawByFiftyMoveRule,
    DrawByInsufficientMaterial,
    DrawByMutualAgreement,
}

#[derive(Serialize, Deserialize)]
pub enum State {
    NotStarted,
    InProgress,
    Finished(GameEnding),
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub game_id: String,
    pub state: State,
    pub current_turn: Color,
    pub board: Board,
    pub move_history: Vec<(Position, Position)>,
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
            game_id,
            state: State::NotStarted,
            current_turn: Color::White,
            board: Board { squares },
            move_history: Vec::new(),
        }
    }
}
