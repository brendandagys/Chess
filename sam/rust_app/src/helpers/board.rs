use crate::{
    helpers::generic::{base64_to_bytes, bytes_to_base64},
    types::{
        game::GameStateAtPointInTime,
        piece::{Color, Piece, PieceType},
    },
};

const NUM_PIECE_TYPES: usize = 12; // 6 piece types * 2 colors

/// Get a byte representation of a piece type for bitboard encoding.
/// We use the piece type + color value to index into the correct bitboard.
pub fn encode_piece(piece: &Piece) -> u8 {
    let base = piece.piece_type as u8;
    base + if piece.color == Color::Black { 6 } else { 0 }
}

/// Decode a piece type from its byte representation/bitboard index
pub fn decode_piece(piece_type_byte: usize) -> Option<Piece> {
    if piece_type_byte >= NUM_PIECE_TYPES {
        return None;
    }

    let color = if piece_type_byte >= 6 {
        Color::Black
    } else {
        Color::White
    };

    let piece_type = PieceType::from(piece_type_byte % 6);

    Some(Piece {
        piece_type,
        color,
        last_game_move: None,
    })
}

/// Represents a collection of bitboards for different piece types on a board.
/// A Vec<u64> is used for each bitboard to represent the squares occupied by that piece type.
/// Since we support board dimensions up to 12x12, we must use a Vec<u64> with some wrapper logic.
pub struct Bitboards {
    pub piece_bitboards: [Vec<u64>; NUM_PIECE_TYPES],
    pub rank_count: usize,
    pub file_count: usize,
}

impl Bitboards {
    pub fn new(rank_count: usize, file_count: usize) -> Self {
        let square_count = rank_count * file_count;
        let u64_count = (square_count + 63) / 64;

        Self {
            piece_bitboards: std::array::from_fn(|_| vec![0; u64_count]),
            rank_count,
            file_count,
        }
    }

    /// Convert 0-based rank and file to a single index for the bitboard
    fn square_index(&self, rank: usize, file: usize) -> usize {
        rank * self.file_count + file
    }

    /// Set a piece on the bitboard
    pub fn set_piece(&mut self, rank: usize, file: usize, piece: &Piece) {
        let index = self.square_index(rank, file);
        let (chunk, bit) = (index / 64, index % 64);
        let piece_type_index = encode_piece(piece);
        self.piece_bitboards[piece_type_index as usize][chunk] |= 1 << bit;
    }

    /// Clear a square on the bitboard
    pub fn clear_square(&mut self, rank: usize, file: usize) {
        let index = self.square_index(rank, file);
        let (chunk, bit) = (index / 64, index % 64);

        for board in &mut self.piece_bitboards {
            board[chunk] &= !(1 << bit);
        }
    }

    /// Get the piece at a specific rank and file
    pub fn get_piece(&self, rank: usize, file: usize) -> Option<Piece> {
        let index = self.square_index(rank, file);
        let (chunk, bit) = (index / 64, index % 64);

        for (i, board) in self.piece_bitboards.iter().enumerate() {
            if (board[chunk] >> bit) & 1 == 1 {
                return decode_piece(i);
            }
        }

        None
    }

    /// Create a new bitboard from a decoded board representation
    pub fn from_board(board: Vec<Vec<Option<Piece>>>) -> Self {
        let rank_count = board.len();
        let file_count = board.first().map_or(0, |r| r.len());

        let mut bitboards = Self::new(rank_count, file_count);

        for (row_index, row) in board.iter().enumerate() {
            for (file, square) in row.iter().enumerate() {
                if let Some(piece) = square {
                    bitboards.set_piece(rank_count - 1 - row_index, file, piece);
                }
            }
        }

        bitboards
    }

    /// Convert the bitboard to a decoded board representation
    pub fn to_board(&self) -> Vec<Vec<Option<Piece>>> {
        let mut board = vec![vec![None; self.file_count]; self.rank_count];

        for (row_index, row) in board.iter_mut().enumerate() {
            for (file, square) in row.iter_mut().enumerate() {
                *square = self.get_piece(self.rank_count - 1 - row_index, file);
            }
        }

        board
    }

    /// Serialize the board into a compact Vec<u8> using 4 bits per square (2 squares per byte).
    /// Each piece is encoded as 0-11, 0xF (15) for empty.
    pub fn to_compact_bytes(&self) -> Vec<u8> {
        let num_squares = self.rank_count * self.file_count;

        let get_nibble = |idx| {
            let rank = idx / self.file_count;
            let file = idx % self.file_count;
            self.get_piece(rank, file)
                .map_or(0xF, |p| encode_piece(&p) & 0xF)
        };

        (0..num_squares)
            .step_by(2)
            .map(|i| (get_nibble(i) << 4) | get_nibble(i + 1))
            .collect()
    }

    /// Deserialize from compact bytes (bitpacked: 4 bits per square, 2 squares per byte)
    pub fn from_compact_bytes(bytes: &[u8], rank_count: usize, file_count: usize) -> Self {
        let num_squares = rank_count * file_count;

        let mut board = vec![vec![None; file_count]; rank_count];

        for idx in 0..num_squares {
            let byte = bytes[idx / 2];
            let nibble = (if idx % 2 == 0 { byte >> 4 } else { byte }) & 0xF;

            if nibble != 0xF {
                if let Some(piece) = decode_piece(nibble as usize) {
                    let rank = idx / file_count;
                    let file = idx % file_count;
                    board[rank_count - 1 - rank][file] = Some(piece);
                }
            }
        }

        Self::from_board(board)
    }

    /// Serialize the bitboard into a base64 string
    pub fn to_base64(&self) -> String {
        bytes_to_base64(&self.to_compact_bytes())
    }

    /// Deserialize into a bitboard from a base64 string
    pub fn from_base64(s: &str, rank_count: usize, file_count: usize) -> Self {
        let bytes = base64_to_bytes(s).expect("Invalid base64 board");
        Self::from_compact_bytes(&bytes, rank_count, file_count)
    }
}

/// Generate a FEN (Forsyth-Edwards Notation) string from a game state.
/// This function only supports standard 8x8 boards.
///
/// FEN format: [piece placement] [active color] [castling] [en passant] [halfmove] [fullmove]
pub fn game_state_to_fen(game_state: &GameStateAtPointInTime) -> String {
    let board = &game_state.board;

    // Validate 8x8 board
    if board.squares.len() != 8 || board.squares[0].len() != 8 {
        panic!("FEN generation is only supported for 8x8 boards");
    }

    let mut fen_parts = Vec::new();

    // 1. Piece placement (from rank 8 to rank 1)
    let piece_placement = generate_piece_placement(&board.squares);
    fen_parts.push(piece_placement);

    // 2. Active color
    let active_color = match game_state.current_turn {
        Color::White => "w",
        Color::Black => "b",
    };
    fen_parts.push(active_color.to_string());

    // 3. Castling availability
    let castling = generate_castling_rights(&board.squares);
    fen_parts.push(castling);

    // 4. En passant target square
    let en_passant = generate_en_passant_target(&board.squares, board.move_count);
    fen_parts.push(en_passant);

    // 5. Halfmove clock (moves since last capture or pawn move)
    // TODO: Implement proper halfmove clock tracking
    fen_parts.push("0".to_string());

    // 6. Fullmove number
    let fullmove = (board.move_count / 2) + 1;
    fen_parts.push(fullmove.to_string());

    fen_parts.join(" ")
}

/// Convert a piece to its FEN character representation
fn piece_to_fen_char(piece: &Piece) -> char {
    let base_char = match piece.piece_type {
        PieceType::Pawn => 'p',
        PieceType::Knight => 'n',
        PieceType::Bishop => 'b',
        PieceType::Rook => 'r',
        PieceType::Queen => 'q',
        PieceType::King => 'k',
    };

    match piece.color {
        Color::White => base_char.to_ascii_uppercase(),
        Color::Black => base_char,
    }
}

/// Generate the piece placement part of FEN notation
fn generate_piece_placement(squares: &[Vec<Option<Piece>>]) -> String {
    let mut ranks = Vec::new();

    // Iterate from rank 8 to rank 1 (row 0 to row 7)
    for row in squares.iter() {
        let mut rank_str = String::new();
        let mut empty_count = 0;

        for square in row.iter() {
            match square {
                Some(piece) => {
                    if empty_count > 0 {
                        rank_str.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    rank_str.push(piece_to_fen_char(piece));
                }
                None => {
                    empty_count += 1;
                }
            }
        }

        if empty_count > 0 {
            rank_str.push_str(&empty_count.to_string());
        }

        ranks.push(rank_str);
    }

    ranks.join("/")
}

/// Generate castling rights string
/// TODO: Validate moving through check
fn generate_castling_rights(squares: &[Vec<Option<Piece>>]) -> String {
    let mut castling = String::new();

    // Check White castling rights (rank 1, which is row index 7)
    if let Some(white_king) = &squares[7][4] {
        if white_king.piece_type == PieceType::King
            && white_king.color == Color::White
            && white_king.last_game_move.is_none()
        {
            // Check kingside rook (h1 = row 7, col 7)
            if let Some(kingside_rook) = &squares[7][7] {
                if kingside_rook.piece_type == PieceType::Rook
                    && kingside_rook.color == Color::White
                    && kingside_rook.last_game_move.is_none()
                {
                    castling.push('K');
                }
            }

            // Check queenside rook (a1 = row 7, col 0)
            if let Some(queenside_rook) = &squares[7][0] {
                if queenside_rook.piece_type == PieceType::Rook
                    && queenside_rook.color == Color::White
                    && queenside_rook.last_game_move.is_none()
                {
                    castling.push('Q');
                }
            }
        }
    }

    // Check Black castling rights (rank 8, which is row index 0)
    if let Some(black_king) = &squares[0][4] {
        if black_king.piece_type == PieceType::King
            && black_king.color == Color::Black
            && black_king.last_game_move.is_none()
        {
            // Check kingside rook (h8 = row 0, col 7)
            if let Some(kingside_rook) = &squares[0][7] {
                if kingside_rook.piece_type == PieceType::Rook
                    && kingside_rook.color == Color::Black
                    && kingside_rook.last_game_move.is_none()
                {
                    castling.push('k');
                }
            }

            // Check queenside rook (a8 = row 0, col 0)
            if let Some(queenside_rook) = &squares[0][0] {
                if queenside_rook.piece_type == PieceType::Rook
                    && queenside_rook.color == Color::Black
                    && queenside_rook.last_game_move.is_none()
                {
                    castling.push('q');
                }
            }
        }
    }

    if castling.is_empty() {
        "-".to_string()
    } else {
        castling
    }
}

/// Generate en passant target square
fn generate_en_passant_target(squares: &[Vec<Option<Piece>>], move_count: usize) -> String {
    // Check if the last move was a two-square pawn advance
    // We need to check rank 4 (row index 4) for white pawns that just moved from rank 2
    // and rank 5 (row index 3) for black pawns that just moved from rank 7

    // Check white pawns on rank 4 (row 4)
    for (file_index, square) in squares[4].iter().enumerate() {
        if let Some(piece) = square {
            if piece.piece_type == PieceType::Pawn
                && piece.color == Color::White
                && piece.last_game_move == Some(move_count)
            {
                // Check if the square two ranks back is empty (rank 2, row 6)
                if squares[6][file_index].is_none() {
                    // En passant target is rank 3 (behind the pawn)
                    let file_char = (b'a' + file_index as u8) as char;
                    return format!("{file_char}3");
                }
            }
        }
    }

    // Check black pawns on rank 5 (row 3)
    for (file_index, square) in squares[3].iter().enumerate() {
        if let Some(piece) = square {
            if piece.piece_type == PieceType::Pawn
                && piece.color == Color::Black
                && piece.last_game_move == Some(move_count)
            {
                // Check if the square two ranks back is empty (rank 7, row 1)
                if squares[1][file_index].is_none() {
                    // En passant target is rank 6 (behind the pawn)
                    let file_char = (b'a' + file_index as u8) as char;
                    return format!("{file_char}6");
                }
            }
        }
    }

    "-".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        board::{Board, BoardSetup},
        game::State,
    };

    #[test]
    fn test_generate_fen_starting_position() {
        let board = Board::new(&BoardSetup::Standard);
        let game_state = GameStateAtPointInTime {
            state: State::NotStarted,
            current_turn: Color::White,
            in_check: None,
            board,
            captured_pieces: crate::types::game::CapturedPieces {
                white: Vec::new(),
                black: Vec::new(),
                white_points: 0,
                black_points: 0,
            },
        };

        let fen = game_state_to_fen(&game_state);
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
