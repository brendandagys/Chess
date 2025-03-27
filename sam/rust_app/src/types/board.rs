use serde::{Deserialize, Serialize};

use super::{
    game::PlayerMove,
    piece::{Color, Piece, PieceType},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rank(pub usize);

impl Rank {
    /// Converts the rank to a 0-based index
    pub fn to_index(&self) -> usize {
        self.0 - 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File(pub usize);

impl File {
    /// Converts the file to a 0-based index
    pub fn to_index(&self) -> usize {
        self.0 - 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub rank: Rank,
    pub file: File,
}

#[derive(Deserialize)]
pub struct BoardSetupDimensions {
    pub ranks: usize,
    pub files: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoardSetup {
    Standard,
    Random(BoardSetupDimensions),
    KingAndOneOtherPiece(BoardSetupDimensions),
}

impl BoardSetup {
    pub fn setup_board(&self) -> Board {
        match self {
            Self::Standard => {
                let mut squares = vec![vec![None; 8]; 8];

                // White major pieces
                squares[0][0] = Some(Piece {
                    piece_type: PieceType::Rook,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][1] = Some(Piece {
                    piece_type: PieceType::Knight,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][2] = Some(Piece {
                    piece_type: PieceType::Bishop,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][3] = Some(Piece {
                    piece_type: PieceType::King,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][4] = Some(Piece {
                    piece_type: PieceType::Queen,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][5] = Some(Piece {
                    piece_type: PieceType::Bishop,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][6] = Some(Piece {
                    piece_type: PieceType::Knight,
                    color: Color::White,
                    has_moved: false,
                });
                squares[0][7] = Some(Piece {
                    piece_type: PieceType::Rook,
                    color: Color::White,
                    has_moved: false,
                });

                // White pawns
                for col in 0..8 {
                    squares[1][col] = Some(Piece {
                        piece_type: PieceType::Pawn,
                        color: Color::White,
                        has_moved: false,
                    });
                }

                // Black pawns
                for col in 0..8 {
                    squares[6][col] = Some(Piece {
                        piece_type: PieceType::Pawn,
                        color: Color::Black,
                        has_moved: false,
                    });
                }

                // Black major pieces
                squares[7][0] = Some(Piece {
                    piece_type: PieceType::Rook,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][1] = Some(Piece {
                    piece_type: PieceType::Knight,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][2] = Some(Piece {
                    piece_type: PieceType::Bishop,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][3] = Some(Piece {
                    piece_type: PieceType::King,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][4] = Some(Piece {
                    piece_type: PieceType::Queen,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][5] = Some(Piece {
                    piece_type: PieceType::Bishop,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][6] = Some(Piece {
                    piece_type: PieceType::Knight,
                    color: Color::Black,
                    has_moved: false,
                });
                squares[7][7] = Some(Piece {
                    piece_type: PieceType::Rook,
                    color: Color::Black,
                    has_moved: false,
                });

                Board { squares }
            }
            // TODO
            Self::Random(dimensions) => Board {
                squares: vec![vec![None; dimensions.files]; dimensions.ranks],
            },
            Self::KingAndOneOtherPiece(dimensions) => Board {
                squares: vec![vec![None; dimensions.files]; dimensions.ranks],
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub squares: Vec<Vec<Option<Piece>>>,
    // Indexes follow the chess board labels, minus one
}

impl Board {
    pub fn new(board_setup: &BoardSetup) -> Self {
        board_setup.setup_board()
    }

    /// Returns all pieces on the board, optionally filtered to a specific color
    pub fn get_all_pieces(&self, color: Option<&Color>) -> Vec<(Piece, Position)> {
        self.squares
            .iter()
            .enumerate()
            .flat_map(|(rank_index, row)| {
                row.iter()
                    .enumerate()
                    .filter_map(move |(file_index, square)| {
                        square.as_ref().and_then(|piece| {
                            let piece_and_position = Some((
                                piece.clone(),
                                Position {
                                    rank: Rank(rank_index + 1),
                                    file: File(file_index + 1),
                                },
                            ));

                            match color {
                                Some(c) if piece.color == *c => piece_and_position,
                                None => piece_and_position,
                                _ => None,
                            }
                        })
                    })
            })
            .collect()
    }

    pub fn get_piece_at_position(&self, position: &Position) -> Option<&Piece> {
        if !self.is_valid_board_position(position) {
            return None;
        }

        let rank_index = position.rank.to_index();
        let file_index = position.file.to_index();
        self.squares[rank_index][file_index].as_ref()
    }

    pub fn set_piece_at_position(&mut self, position: &Position, piece: Option<Piece>) {
        let rank_index = position.rank.to_index();
        let file_index = position.file.to_index();
        self.squares[rank_index][file_index] = piece;
    }

    pub fn is_valid_board_position(&self, position: &Position) -> bool {
        position.rank.to_index() < self.squares.len()
            && position.file.to_index() < self.squares[0].len()
    }

    /// Helper function for King and Knight pieces, whose on-board moves can only
    /// be impeded by the presence of a friendly piece at the destination square.
    /// The move can not be impeded by any pieces **on the way** to that square.
    pub fn is_valid_position_for_king_or_knight_in_game(
        &self,
        position: &Position,
        piece: &Piece,
    ) -> bool {
        if !self.is_valid_board_position(position) {
            return false;
        }

        if let Some(other_piece_at_position) = self.get_piece_at_position(position) {
            return other_piece_at_position.color != piece.color;
        }

        true
    }

    /// These pieces can move in straight lines, and their moves can be impeded
    /// by the presence of a friendly piece at the destination square, or by
    /// any pieces on the way to that square.
    pub fn get_valid_positions_for_bishop_or_rook_or_queen(
        &self,
        rank_index: isize,
        file_index: isize,
        color: &Color,
        offsets: &[(isize, isize)],
    ) -> Vec<Position> {
        offsets
            .iter()
            .fold(Vec::new(), |mut acc, (offset_r, offset_f)| {
                let mut rank_index_ = rank_index;
                let mut file_index_ = file_index;

                loop {
                    rank_index_ += offset_r;
                    file_index_ += offset_f;

                    let tentative_position = Position {
                        rank: Rank(rank_index_ as usize),
                        file: File(file_index_ as usize),
                    };

                    if !self.is_valid_board_position(&tentative_position) {
                        break;
                    }

                    if let Some(other_piece_at_position) =
                        self.get_piece_at_position(&tentative_position)
                    {
                        if other_piece_at_position.color != *color {
                            acc.push(tentative_position);
                        }

                        break;
                    } else {
                        acc.push(tentative_position);
                    }
                }

                acc
            })
    }

    pub fn is_king_in_check(&self, color: &Color) -> bool {
        let pieces_and_positions_for_color = self.get_all_pieces(Some(color));

        let (_king, king_position) = pieces_and_positions_for_color
            .iter()
            .find(|(piece, _position)| piece.piece_type == PieceType::King)
            .unwrap();

        let opponent_attacking_squares: Vec<Position> = self
            .get_all_pieces(Some(&color.opponent_color()))
            .iter()
            .fold(Vec::new(), |mut acc, (piece, position)| {
                acc.extend(piece.possible_moves(self, position));
                acc
            });

        opponent_attacking_squares.contains(king_position)
    }

    /// This function assumes the move has been validated
    pub fn apply_move(&mut self, player_move: &PlayerMove) {
        self.set_piece_at_position(
            &player_move.to,
            self.get_piece_at_position(&player_move.from).cloned(),
        );

        self.set_piece_at_position(&player_move.from, None);
    }

    // TODO: pub fn unapply_move()
}
