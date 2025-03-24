use serde::{Deserialize, Serialize};

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
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub has_moved: bool,
}

impl Piece {
    fn get_tentative_position(
        &self,
        rank_index: isize,
        file_index: isize,
        offset: (&i32, &i32),
    ) -> Position {
        let new_rank = rank_index + *offset.0 as isize;
        let new_file = file_index + *offset.1 as isize;

        Position {
            rank: Rank(new_rank as usize),
            file: File(new_file as usize),
        }
    }

    pub fn possible_moves(&self, board: &Board, position: &Position) -> Vec<Position> {
        let rank_index = position.rank.to_index() as isize;
        let file_index = position.file.to_index() as isize;

        match self.piece_type {
            PieceType::King => [
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
                let tentative_position =
                    self.get_tentative_position(rank_index, file_index, (offset_r, offset_f));

                match board.is_valid_position_for_king_or_knight_in_game(position, self) {
                    true => Some(tentative_position),
                    false => None,
                }
            })
            .collect::<Vec<Position>>(),

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
                let tentative_position =
                    self.get_tentative_position(rank_index, file_index, (offset_r, offset_f));

                match board.is_valid_position_for_king_or_knight_in_game(position, self) {
                    true => Some(tentative_position),
                    false => None,
                }
            })
            .collect::<Vec<Position>>(),

            PieceType::Pawn => {
                let mut moves = Vec::new();

                // Single-square forward
                let new_single_jump_rank = rank_index
                    + match self.color {
                        Color::White => 1isize,
                        Color::Black => -1isize,
                    };

                let tentative_single_jump_position = Position {
                    rank: Rank(new_single_jump_rank as usize),
                    file: File(file_index as usize),
                };

                if board.is_valid_board_position(&tentative_single_jump_position)
                    && board
                        .get_piece_at_position(&tentative_single_jump_position)
                        .is_none()
                {
                    moves.push(tentative_single_jump_position);

                    // Double-square forward; single-jump must also be valid
                    if !self.has_moved {
                        let new_double_jump_rank = rank_index
                            + match self.color {
                                Color::White => 2isize,
                                Color::Black => -2isize,
                            };

                        let tentative_double_jump_position = Position {
                            rank: Rank(new_double_jump_rank as usize),
                            file: File(file_index as usize),
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
                    let capture_file = file_index + file_offset;

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

                // TODO: En passant capture
                // Store move COUNT on each piece and last game move (#),
                // to validate we are capturing a pawn that JUST moved forward two squares,
                // and that we are capturing on the very next move

                moves
            }

            PieceType::Bishop => board.get_valid_positions_for_bishop_or_rook_or_queen(
                rank_index,
                file_index,
                &self.color,
                &[(-1, -1), (-1, 1), (1, -1), (1, 1)],
            ),

            PieceType::Rook => board.get_valid_positions_for_bishop_or_rook_or_queen(
                rank_index,
                file_index,
                &self.color,
                &[(-1, 0), (1, 0), (0, -1), (0, 1)],
            ),

            PieceType::Queen => board.get_valid_positions_for_bishop_or_rook_or_queen(
                rank_index,
                file_index,
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
