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

                // Black major pieces
                squares[0][0] = Some(Piece::new(PieceType::Rook, Color::Black));
                squares[0][1] = Some(Piece::new(PieceType::Knight, Color::Black));
                squares[0][2] = Some(Piece::new(PieceType::Bishop, Color::Black));
                squares[0][3] = Some(Piece::new(PieceType::Queen, Color::Black));
                squares[0][4] = Some(Piece::new(PieceType::King, Color::Black));
                squares[0][5] = Some(Piece::new(PieceType::Bishop, Color::Black));
                squares[0][6] = Some(Piece::new(PieceType::Knight, Color::Black));
                squares[0][7] = Some(Piece::new(PieceType::Rook, Color::Black));

                // Black pawns
                for col in 0..8 {
                    squares[1][col] = Some(Piece::new(PieceType::Pawn, Color::Black));
                }

                // White pawns
                for col in 0..8 {
                    squares[6][col] = Some(Piece::new(PieceType::Pawn, Color::White));
                }

                // White major pieces
                squares[7][0] = Some(Piece::new(PieceType::Rook, Color::White));
                squares[7][1] = Some(Piece::new(PieceType::Knight, Color::White));
                squares[7][2] = Some(Piece::new(PieceType::Bishop, Color::White));
                squares[7][3] = Some(Piece::new(PieceType::Queen, Color::White));
                squares[7][4] = Some(Piece::new(PieceType::King, Color::White));
                squares[7][5] = Some(Piece::new(PieceType::Bishop, Color::White));
                squares[7][6] = Some(Piece::new(PieceType::Knight, Color::White));
                squares[7][7] = Some(Piece::new(PieceType::Rook, Color::White));

                Board {
                    squares,
                    move_count: 0,
                }
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
                    squares[0][i] = Some(Piece::new(piece_type, Color::Black));
                    squares[dimensions.ranks - 1][i] = Some(Piece::new(piece_type, Color::White));
                }

                // Second and second-to-last ranks
                for (i, piece_type) in inner_row.into_iter().enumerate() {
                    squares[1][i] = Some(Piece::new(piece_type, Color::Black));
                    squares[dimensions.ranks - 2][i] = Some(Piece::new(piece_type, Color::White));
                }

                // Place kings
                squares[0][king_file] = Some(Piece::new(PieceType::King, Color::Black));
                squares[dimensions.ranks - 1][king_file] =
                    Some(Piece::new(PieceType::King, Color::White));

                Board {
                    squares,
                    move_count: 0,
                }
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
                    squares[0][i] = Some(Piece::new(chosen_piece, Color::Black));
                    squares[1][i] = Some(Piece::new(chosen_piece, Color::Black));

                    squares[dimensions.ranks - 2][i] = Some(Piece::new(chosen_piece, Color::White));
                    squares[dimensions.ranks - 1][i] = Some(Piece::new(chosen_piece, Color::White));
                }

                // Place kings
                squares[0][king_file] = Some(Piece::new(PieceType::King, Color::Black));
                squares[dimensions.ranks - 1][king_file] =
                    Some(Piece::new(PieceType::King, Color::White));

                Board {
                    squares,
                    move_count: 0,
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    // Indexes follow the chess board labels, minus one
    pub squares: Vec<Vec<Option<Piece>>>,
    pub move_count: usize,
}

impl Board {
    pub fn new(board_setup: &BoardSetup) -> Self {
        board_setup.setup_board()
    }

    /// Returns all pieces on the board, optionally filtered to a specific color
    pub fn get_all_pieces(&self, color: Option<&Color>) -> Vec<(Piece, Position)> {
        let num_ranks = self.squares.len();

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
                                    rank: Rank(num_ranks - rank_index),
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

        let rank_index = self.squares.len() - position.rank.0;
        let file_index = position.file.to_index();
        self.squares[rank_index][file_index].as_ref()
    }

    pub fn set_piece_at_position(&mut self, position: &Position, piece: Option<Piece>) {
        if !self.is_valid_board_position(position) {
            return;
        }

        let rank_index = self.squares.len() - position.rank.0;
        let file_index = position.file.to_index();
        self.squares[rank_index][file_index] = piece;
    }

    pub fn is_valid_board_position(&self, position: &Position) -> bool {
        position.rank.0 > 0
            && position.file.0 > 0
            && position.rank.0 <= self.squares.len()
            && position.file.0 <= self.squares[0].len()
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
        position: &Position,
        color: &Color,
        offsets: &[(isize, isize)],
    ) -> Vec<Position> {
        offsets
            .iter()
            .fold(Vec::new(), |mut acc, (offset_r, offset_f)| {
                let mut new_rank = position.rank.to_index() as isize;
                let mut new_file = position.file.to_index() as isize;

                loop {
                    new_rank += *offset_r;
                    new_file += *offset_f;

                    let tentative_position = Position {
                        rank: Rank((new_rank + 1) as usize),
                        file: File((new_file + 1) as usize),
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
            .expect(&format!(
                "Did not find {color} king when checking for check"
            ));

        let opponent_pieces = self.get_all_pieces(Some(&color.opponent_color()));

        for (piece, position) in opponent_pieces {
            for move_position in piece.possible_moves(self, &position) {
                if move_position == *king_position {
                    return true;
                }
            }
        }

        false
    }

    fn check_for_pawn_promotion(&self, piece: &mut Piece, player_move: &PlayerMove) {
        if piece.piece_type == PieceType::Pawn
            && piece.color == Color::Black
            && player_move.to.rank.0 - 1 == 0
        {
            piece.piece_type = PieceType::Queen;
        } else if piece.piece_type == PieceType::Pawn
            && piece.color == Color::White
            && player_move.to.rank.0 == self.squares.len()
        {
            piece.piece_type = PieceType::Queen;
        }
    }

    /// Only called if the destination square is empty
    fn check_for_en_passant_pawn_capture(
        &mut self,
        player_piece: &Piece,
        player_move: &PlayerMove,
    ) -> Option<Piece> {
        if player_piece.piece_type == PieceType::Pawn
            && player_move.from.file != player_move.to.file
        {
            let captured_pawn_position = Position {
                rank: player_move.from.rank.clone(),
                file: player_move.to.file.clone(),
            };

            let captured_pawn = self.get_piece_at_position(&captured_pawn_position).cloned();
            self.set_piece_at_position(&captured_pawn_position, None);

            return captured_pawn;
        }

        None
    }

    fn check_for_captured_piece(
        &mut self,
        player_piece: &Piece,
        player_move: &PlayerMove,
    ) -> Option<Piece> {
        let captured_piece_at_destination = self.get_piece_at_position(&player_move.to).cloned();

        match captured_piece_at_destination {
            Some(captured_piece) => Some(captured_piece),
            None => self.check_for_en_passant_pawn_capture(&player_piece, player_move),
        }
    }

    fn check_for_castling(&mut self, player_piece: &Piece, player_move: &PlayerMove) -> bool {
        if player_piece.piece_type == PieceType::King
            && (player_move.to.file.0 == 1 || player_move.to.file.0 == self.squares[0].len())
            && (player_move.from.file.0 as isize - player_move.to.file.0 as isize).abs() > 1
        {
            let (old_rook_file, new_rook_file, new_king_file) =
                if player_move.from.file.0 < player_move.to.file.0 {
                    (
                        self.squares[0].len(),
                        player_move.from.file.0 + 1,
                        player_move.from.file.0 + 2,
                    )
                } else {
                    (1, player_move.from.file.0 - 1, player_move.from.file.0 - 2)
                };

            let old_rook_position = Position {
                rank: player_move.from.rank.clone(),
                file: File(old_rook_file),
            };

            let new_rook_position = Position {
                rank: player_move.from.rank.clone(),
                file: File(new_rook_file),
            };

            let new_king_position = Position {
                rank: player_move.from.rank.clone(),
                file: File(new_king_file),
            };

            let rook = self
                .get_piece_at_position(&old_rook_position)
                .cloned()
                .expect(&format!(
                    "Did not find rook at position when castling: {:?}",
                    old_rook_position
                ));

            self.set_piece_at_position(&new_rook_position, Some(rook));
            self.set_piece_at_position(&old_rook_position, None);

            let king = self
                .get_piece_at_position(&player_move.from)
                .cloned()
                .expect(&format!(
                    "Did not find king at position when castling: {:?}",
                    player_move.from
                ));

            self.set_piece_at_position(&new_king_position, Some(king));
            self.set_piece_at_position(&player_move.from, None);

            return true;
        }

        false
    }

    /// This function assumes the move has been validated
    pub fn apply_move(&mut self, player_move: &PlayerMove) -> Option<Piece> {
        let mut player_piece = self
            .get_piece_at_position(&player_move.from)
            .cloned()
            .expect(&format!(
                "Did not find any piece to move from {:?} to {:?}",
                player_move.from, player_move.to
            ));

        self.move_count += 1;
        player_piece.last_game_move = Some(self.move_count);

        self.check_for_pawn_promotion(&mut player_piece, player_move);

        match self.check_for_castling(&player_piece, player_move) {
            true => None,
            false => {
                let captured_piece = self.check_for_captured_piece(&player_piece, player_move);

                self.set_piece_at_position(&player_move.to, Some(player_piece));
                self.set_piece_at_position(&player_move.from, None);

                captured_piece
            }
        }
    }

    // TODO: pub fn unapply_move()
}
