use rand::seq::IndexedRandom;
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
                squares[0][0] = Some(Piece::new(PieceType::Rook, Color::White));
                squares[0][1] = Some(Piece::new(PieceType::Knight, Color::White));
                squares[0][2] = Some(Piece::new(PieceType::Bishop, Color::White));
                squares[0][3] = Some(Piece::new(PieceType::King, Color::White));
                squares[0][4] = Some(Piece::new(PieceType::Queen, Color::White));
                squares[0][5] = Some(Piece::new(PieceType::Bishop, Color::White));
                squares[0][6] = Some(Piece::new(PieceType::Knight, Color::White));
                squares[0][7] = Some(Piece::new(PieceType::Rook, Color::White));

                // White pawns
                for col in 0..8 {
                    squares[1][col] = Some(Piece::new(PieceType::Pawn, Color::White));
                }

                // Black pawns
                for col in 0..8 {
                    squares[6][col] = Some(Piece::new(PieceType::Pawn, Color::Black));
                }

                // Black major pieces
                squares[7][0] = Some(Piece::new(PieceType::Rook, Color::Black));
                squares[7][1] = Some(Piece::new(PieceType::Knight, Color::Black));
                squares[7][2] = Some(Piece::new(PieceType::Bishop, Color::Black));
                squares[7][3] = Some(Piece::new(PieceType::King, Color::Black));
                squares[7][4] = Some(Piece::new(PieceType::Queen, Color::Black));
                squares[7][5] = Some(Piece::new(PieceType::Bishop, Color::Black));
                squares[7][6] = Some(Piece::new(PieceType::Knight, Color::Black));
                squares[7][7] = Some(Piece::new(PieceType::Rook, Color::Black));

                Board { squares }
            }
            Self::Random(dimensions) => {
                let mut squares = vec![vec![None; dimensions.files]; dimensions.ranks];
                let mut rng = rand::rng();

                let mut available_pieces = vec![
                    PieceType::Rook,
                    PieceType::Knight,
                    PieceType::Bishop,
                    PieceType::Queen,
                ];

                let mut generate_row = |piece_types: &Vec<PieceType>| {
                    (0..dimensions.files)
                        .map(|_| *piece_types.choose(&mut rng).unwrap())
                        .collect::<Vec<PieceType>>()
                };

                let outer_row = generate_row(&available_pieces);
                available_pieces.push(PieceType::Pawn);
                let inner_row = generate_row(&available_pieces);

                let king_file = dimensions.files / 2;

                // First and last ranks
                for (i, piece_type) in outer_row.into_iter().enumerate() {
                    squares[0][i] = Some(Piece::new(piece_type, Color::White));
                    squares[dimensions.ranks - 1][i] = Some(Piece::new(piece_type, Color::Black));
                }

                // Second and second-to-last ranks
                for (i, piece_type) in inner_row.into_iter().enumerate() {
                    squares[1][i] = Some(Piece::new(piece_type, Color::White));
                    squares[dimensions.ranks - 2][i] = Some(Piece::new(piece_type, Color::Black));
                }

                // Place kings
                squares[0][king_file] = Some(Piece::new(PieceType::King, Color::White));
                squares[dimensions.ranks - 1][king_file] =
                    Some(Piece::new(PieceType::King, Color::Black));

                Board { squares }
            }
            Self::KingAndOneOtherPiece(dimensions) => {
                let mut squares = vec![vec![None; dimensions.files]; dimensions.ranks];
                let mut rng = rand::rng();

                let available_pieces = vec![
                    PieceType::Rook,
                    PieceType::Knight,
                    PieceType::Bishop,
                    PieceType::Queen,
                    PieceType::Pawn,
                ];

                let chosen_piece = available_pieces.choose(&mut rng).unwrap().clone();
                let king_file = dimensions.files / 2;

                // Place the random piece
                for i in 0..dimensions.files {
                    squares[0][i] = Some(Piece::new(chosen_piece, Color::White));
                    squares[1][i] = Some(Piece::new(chosen_piece, Color::White));

                    squares[dimensions.ranks - 2][i] = Some(Piece::new(chosen_piece, Color::Black));
                    squares[dimensions.ranks - 1][i] = Some(Piece::new(chosen_piece, Color::Black));
                }

                // Place kings
                squares[0][king_file] = Some(Piece::new(PieceType::King, Color::White));
                squares[dimensions.ranks - 1][king_file] =
                    Some(Piece::new(PieceType::King, Color::Black));

                Board { squares }
            }
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
