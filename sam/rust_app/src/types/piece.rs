use serde::{Deserialize, Serialize};

use crate::types::game::PlayerMove;

use super::board::{Board, File, Position, Rank};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opponent_color(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "white"),
            Color::Black => write!(f, "black"),
        }
    }
}

impl std::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "white" => Ok(Color::White),
            "black" => Ok(Color::Black),
            _ => Err(format!("'{s}' is not a valid color")),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl From<usize> for PieceType {
    fn from(value: usize) -> Self {
        match value {
            0 => PieceType::Pawn,
            1 => PieceType::Knight,
            2 => PieceType::Bishop,
            3 => PieceType::Rook,
            4 => PieceType::Queen,
            5 => PieceType::King,
            _ => panic!("Invalid value for PieceType: {}", value),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub last_game_move: Option<usize>,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece {
            piece_type,
            color,
            last_game_move: None,
        }
    }

    pub fn get_point_value(&self) -> u16 {
        match self.piece_type {
            PieceType::King => 0,
            PieceType::Queen => 9,
            PieceType::Rook => 5,
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Pawn => 1,
        }
    }

    fn calculate_offset(&self, position: &Position, offset: (&i32, &i32)) -> Position {
        let new_rank = position.rank.0 + *offset.0 as usize;
        let new_file = position.file.0 + *offset.1 as usize;

        Position {
            rank: Rank(new_rank),
            file: File(new_file),
        }
    }

    fn get_allowed_castling_positions_lower_files(
        &self,
        board: &Board,
        king_position: &Position,
    ) -> Vec<Position> {
        if board.is_king_in_check(&self.color) {
            return vec![];
        }

        let no_pieces_between_king_and_rook_and_not_moving_through_check =
            (2..king_position.file.0).all(|file| {
                let position_to_check = Position {
                    rank: king_position.rank.clone(),
                    file: File(file),
                };

                board.get_piece_at_position(&position_to_check).is_none()
                    && (file < king_position.file.0 - 2 || {
                        let mut hypothetical_board = board.clone();
                        hypothetical_board.apply_move(&PlayerMove {
                            from: king_position.clone(),
                            to: position_to_check.clone(),
                        });
                        !hypothetical_board.is_king_in_check(&self.color)
                    })
            });

        if no_pieces_between_king_and_rook_and_not_moving_through_check {
            let rook_position = Position {
                rank: king_position.rank.clone(),
                file: File(1),
            };

            if let Some(piece) = board.get_piece_at_position(&rook_position) {
                if piece.piece_type == PieceType::Rook && piece.last_game_move.is_none() {
                    let mut hypothetical_board = board.clone();

                    let tentative_new_king_position = Position {
                        rank: king_position.rank.clone(),
                        file: File(king_position.file.0 - 2),
                    };

                    hypothetical_board.apply_move(&PlayerMove {
                        from: king_position.clone(),
                        to: tentative_new_king_position,
                    });

                    hypothetical_board.apply_move(&PlayerMove {
                        from: rook_position.clone(),
                        to: Position {
                            rank: king_position.rank.clone(),
                            file: File(king_position.file.0 - 1),
                        },
                    });

                    if !hypothetical_board.is_king_in_check(&self.color) {
                        return std::iter::once(rook_position)
                            .chain((2..king_position.file.0 - 1).map(|file| Position {
                                rank: king_position.rank.clone(),
                                file: File(file),
                            }))
                            .collect();
                    }
                }
            }
        }

        vec![]
    }

    fn get_allowed_castling_positions_upper_files(
        &self,
        board: &Board,
        king_position: &Position,
    ) -> Vec<Position> {
        if board.is_king_in_check(&self.color) {
            return vec![];
        }

        let no_pieces_between_king_and_rook_and_not_moving_through_check =
            (king_position.file.0 + 1..board.squares[0].len()).all(|file| {
                let position_to_check = Position {
                    rank: king_position.rank.clone(),
                    file: File(file),
                };

                board.get_piece_at_position(&position_to_check).is_none()
                    && (file > king_position.file.0 + 2 || {
                        let mut hypothetical_board = board.clone();
                        hypothetical_board.apply_move(&PlayerMove {
                            from: king_position.clone(),
                            to: position_to_check.clone(),
                        });
                        !hypothetical_board.is_king_in_check(&self.color)
                    })
            });

        if no_pieces_between_king_and_rook_and_not_moving_through_check {
            let rook_position = Position {
                rank: king_position.rank.clone(),
                file: File(board.squares[0].len()),
            };

            if let Some(piece) = board.get_piece_at_position(&rook_position) {
                if piece.piece_type == PieceType::Rook && piece.last_game_move.is_none() {
                    let mut hypothetical_board = board.clone();

                    let tentative_new_king_position = Position {
                        rank: king_position.rank.clone(),
                        file: File(king_position.file.0 + 2),
                    };

                    hypothetical_board.apply_move(&PlayerMove {
                        from: king_position.clone(),
                        to: tentative_new_king_position,
                    });

                    hypothetical_board.apply_move(&PlayerMove {
                        from: rook_position.clone(),
                        to: Position {
                            rank: king_position.rank.clone(),
                            file: File(king_position.file.0 + 1),
                        },
                    });

                    if !hypothetical_board.is_king_in_check(&self.color) {
                        return std::iter::once(rook_position)
                            .chain(
                                (king_position.file.0 + 2..board.squares[0].len()).map(|file| {
                                    Position {
                                        rank: king_position.rank.clone(),
                                        file: File(file),
                                    }
                                }),
                            )
                            .collect();
                    }
                }
            }
        }

        vec![]
    }

    fn get_allowed_castling_positions(
        &self,
        board: &Board,
        king_position: &Position,
    ) -> Vec<Position> {
        if self.piece_type != PieceType::King || self.last_game_move.is_some() {
            return vec![];
        }

        [
            self.get_allowed_castling_positions_lower_files(board, king_position),
            self.get_allowed_castling_positions_upper_files(board, king_position),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn possible_moves(&self, board: &Board, position: &Position) -> Vec<Position> {
        match self.piece_type {
            PieceType::King => {
                let standard_king_moves = [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, -1),
                    (0, 1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                ]
                .iter()
                .filter_map(|(offset_r, offset_f)| {
                    let tentative_position = self.calculate_offset(position, (offset_r, offset_f));

                    match board
                        .is_valid_position_for_king_or_knight_in_game(&tentative_position, self)
                    {
                        true => Some(tentative_position),
                        false => None,
                    }
                })
                .collect::<Vec<Position>>();

                standard_king_moves
                    .into_iter()
                    .chain(self.get_allowed_castling_positions(board, position))
                    .collect::<Vec<Position>>()
            }

            PieceType::Knight => [
                (-2, -1),
                (-2, 1),
                (-1, -2),
                (-1, 2),
                (1, -2),
                (1, 2),
                (2, -1),
                (2, 1),
            ]
            .iter()
            .filter_map(|(offset_r, offset_f)| {
                let tentative_position = self.calculate_offset(position, (offset_r, offset_f));

                match board.is_valid_position_for_king_or_knight_in_game(&tentative_position, self)
                {
                    true => Some(tentative_position),
                    false => None,
                }
            })
            .collect::<Vec<Position>>(),

            PieceType::Pawn => {
                let mut moves = Vec::new();

                // Single-square forward
                let new_single_jump_rank = position.rank.0 as isize
                    + match self.color {
                        Color::White => 1isize,
                        Color::Black => -1isize,
                    };

                let tentative_single_jump_position = Position {
                    rank: Rank(new_single_jump_rank as usize),
                    file: File(position.file.0),
                };

                if board.is_valid_board_position(&tentative_single_jump_position)
                    && board
                        .get_piece_at_position(&tentative_single_jump_position)
                        .is_none()
                {
                    moves.push(tentative_single_jump_position);

                    // Double-square forward; single-jump must also be valid
                    if self.last_game_move.is_none() {
                        let new_double_jump_rank = position.rank.0 as isize
                            + match self.color {
                                Color::White => 2isize,
                                Color::Black => -2isize,
                            };

                        let tentative_double_jump_position = Position {
                            rank: Rank(new_double_jump_rank as usize),
                            file: File(position.file.0),
                        };

                        if board.is_valid_board_position(&tentative_double_jump_position)
                            && board
                                .get_piece_at_position(&tentative_double_jump_position)
                                .is_none()
                        {
                            moves.push(tentative_double_jump_position);
                        }
                    }
                }

                // Regular capture
                for file_offset in &[-1, 1] {
                    let capture_file = position.file.0 as isize + file_offset;

                    let tentative_capture_position = Position {
                        rank: Rank(new_single_jump_rank as usize),
                        file: File(capture_file as usize),
                    };

                    if board.is_valid_board_position(&tentative_capture_position) {
                        if let Some(target_piece) =
                            board.get_piece_at_position(&tentative_capture_position)
                        {
                            if target_piece.color != self.color {
                                moves.push(tentative_capture_position);
                            }
                        }
                    }
                }

                // En passant capture
                if self.color == Color::White && position.rank.0 == board.squares.len() - 3 {
                    let left_position = Position {
                        rank: Rank(position.rank.0),
                        file: File(position.file.0 - 1),
                    };

                    let right_position = Position {
                        rank: Rank(position.rank.0),
                        file: File(position.file.0 + 1),
                    };

                    if let Some(piece) = board.get_piece_at_position(&left_position) {
                        if piece.piece_type == PieceType::Pawn
                            && piece.color == Color::Black
                            && piece.last_game_move == Some(board.move_count)
                        {
                            moves.push(Position {
                                rank: Rank(position.rank.0 + 1),
                                file: File(position.file.0 - 1),
                            });
                        }
                    }

                    if let Some(piece) = board.get_piece_at_position(&right_position) {
                        if piece.piece_type == PieceType::Pawn
                            && piece.color == Color::Black
                            && piece.last_game_move == Some(board.move_count)
                        {
                            moves.push(Position {
                                rank: Rank(position.rank.0 + 1),
                                file: File(position.file.0 + 1),
                            });
                        }
                    }
                }

                if self.color == Color::Black && position.rank.0 == 4 {
                    let left_position = Position {
                        rank: Rank(position.rank.0),
                        file: File(position.file.0 + 1),
                    };

                    let right_position = Position {
                        rank: Rank(position.rank.0),
                        file: File(position.file.0 - 1),
                    };

                    if let Some(piece) = board.get_piece_at_position(&left_position) {
                        if piece.piece_type == PieceType::Pawn
                            && piece.color == Color::White
                            && piece.last_game_move == Some(board.move_count)
                        {
                            moves.push(Position {
                                rank: Rank(position.rank.0 - 1),
                                file: File(position.file.0 + 1),
                            });
                        }
                    }

                    if let Some(piece) = board.get_piece_at_position(&right_position) {
                        if piece.piece_type == PieceType::Pawn
                            && piece.color == Color::White
                            && piece.last_game_move == Some(board.move_count)
                        {
                            moves.push(Position {
                                rank: Rank(position.rank.0 - 1),
                                file: File(position.file.0 - 1),
                            });
                        }
                    }
                }

                moves
            }

            PieceType::Bishop => board.get_valid_positions_for_bishop_or_rook_or_queen(
                position,
                &self.color,
                &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
            ),

            PieceType::Rook => board.get_valid_positions_for_bishop_or_rook_or_queen(
                position,
                &self.color,
                &[(-1, 0), (1, 0), (0, -1), (0, 1)],
            ),

            PieceType::Queen => board.get_valid_positions_for_bishop_or_rook_or_queen(
                position,
                &self.color,
                &[
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                ],
            ),
        }
    }
}
