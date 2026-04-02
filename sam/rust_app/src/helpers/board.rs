use crate::{
    helpers::generic::{base64_to_bytes, bytes_to_base64},
    types::{
        board::Board,
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
        move_count: 0,
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

/// Result of parsing a FEN string, containing the board and metadata.
pub struct FenParseResult {
    pub board: Board,
    pub active_color: Color,
}

/// Parse a FEN character into a Piece.
fn fen_char_to_piece(c: char) -> Option<Piece> {
    let color = if c.is_uppercase() {
        Color::White
    } else {
        Color::Black
    };

    let piece_type = match c.to_ascii_lowercase() {
        'p' => PieceType::Pawn,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => return None,
    };

    Some(Piece::new(piece_type, color))
}

/// Parse a FEN (Forsyth-Edwards Notation) string into a Board and active color.
///
/// FEN format: [piece placement] [active color] [castling] [en passant] [halfmove] [fullmove]
///
/// Returns an error string if the FEN is invalid.
pub fn fen_to_board(fen: &str) -> Result<FenParseResult, String> {
    let parts: Vec<&str> = fen.trim().split_whitespace().collect();

    if parts.len() != 6 {
        return Err(format!(
            "FEN must have 6 fields separated by spaces, got {}",
            parts.len()
        ));
    }

    let piece_placement = parts[0];
    let active_color_str = parts[1];
    let castling_str = parts[2];
    let en_passant_str = parts[3];
    let fullmove_str = parts[5];

    // 1. Parse active color
    let active_color = match active_color_str {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err(format!("Invalid active color: '{active_color_str}'")),
    };

    // 2. Parse fullmove number to compute board move_count
    let fullmove: usize = fullmove_str
        .parse()
        .map_err(|_| format!("Invalid fullmove number: '{fullmove_str}'"))?;

    if fullmove == 0 {
        return Err("Fullmove number must be at least 1".to_string());
    }

    let board_move_count = match active_color {
        Color::White => (fullmove - 1) * 2,
        Color::Black => (fullmove - 1) * 2 + 1,
    };

    // 3. Parse piece placement
    let ranks: Vec<&str> = piece_placement.split('/').collect();
    if ranks.len() != 8 {
        return Err(format!(
            "FEN piece placement must have 8 ranks, got {}",
            ranks.len()
        ));
    }

    let mut squares = vec![vec![None; 8]; 8];

    for (row_index, rank_str) in ranks.iter().enumerate() {
        let mut file_index = 0;

        for c in rank_str.chars() {
            if file_index > 7 {
                return Err(format!("Rank {} has too many squares", 8 - row_index));
            }

            if let Some(digit) = c.to_digit(10) {
                file_index += digit as usize;
            } else if let Some(piece) = fen_char_to_piece(c) {
                squares[row_index][file_index] = Some(piece);
                file_index += 1;
            } else {
                return Err(format!("Invalid FEN character: '{c}'"));
            }
        }

        if file_index != 8 {
            return Err(format!(
                "Rank {} has {} squares instead of 8",
                8 - row_index,
                file_index
            ));
        }
    }

    // 4. Apply castling rights - pieces that have lost castling rights get move_count = 1
    // First, mark all kings and rooks as having moved (move_count = 1)
    // Then, restore move_count = 0 for those with castling rights
    for row in &mut squares {
        for square in row.iter_mut() {
            if let Some(piece) = square {
                if piece.piece_type == PieceType::King || piece.piece_type == PieceType::Rook {
                    piece.move_count = 1;
                }
            }
        }
    }

    if castling_str != "-" {
        for c in castling_str.chars() {
            match c {
                'K' => {
                    // White kingside: white king at e1 (row 7, col 4), white rook at h1 (row 7, col 7)
                    if let Some(piece) = &mut squares[7][4] {
                        if piece.piece_type == PieceType::King && piece.color == Color::White {
                            piece.move_count = 0;
                        }
                    }
                    if let Some(piece) = &mut squares[7][7] {
                        if piece.piece_type == PieceType::Rook && piece.color == Color::White {
                            piece.move_count = 0;
                        }
                    }
                }
                'Q' => {
                    // White queenside: white king at e1, white rook at a1 (row 7, col 0)
                    if let Some(piece) = &mut squares[7][4] {
                        if piece.piece_type == PieceType::King && piece.color == Color::White {
                            piece.move_count = 0;
                        }
                    }
                    if let Some(piece) = &mut squares[7][0] {
                        if piece.piece_type == PieceType::Rook && piece.color == Color::White {
                            piece.move_count = 0;
                        }
                    }
                }
                'k' => {
                    // Black kingside: black king at e8 (row 0, col 4), black rook at h8 (row 0, col 7)
                    if let Some(piece) = &mut squares[0][4] {
                        if piece.piece_type == PieceType::King && piece.color == Color::Black {
                            piece.move_count = 0;
                        }
                    }
                    if let Some(piece) = &mut squares[0][7] {
                        if piece.piece_type == PieceType::Rook && piece.color == Color::Black {
                            piece.move_count = 0;
                        }
                    }
                }
                'q' => {
                    // Black queenside: black king at e8, black rook at a8 (row 0, col 0)
                    if let Some(piece) = &mut squares[0][4] {
                        if piece.piece_type == PieceType::King && piece.color == Color::Black {
                            piece.move_count = 0;
                        }
                    }
                    if let Some(piece) = &mut squares[0][0] {
                        if piece.piece_type == PieceType::Rook && piece.color == Color::Black {
                            piece.move_count = 0;
                        }
                    }
                }
                _ => return Err(format!("Invalid castling character: '{c}'")),
            }
        }
    }

    // 5. Set pawn move_count based on position
    // Pawns on their starting rank have move_count = 0, others have move_count = 1
    for (row_index, row) in squares.iter_mut().enumerate() {
        for square in row.iter_mut() {
            if let Some(piece) = square {
                if piece.piece_type == PieceType::Pawn {
                    let is_on_starting_rank = match piece.color {
                        Color::White => row_index == 6, // Rank 2
                        Color::Black => row_index == 1, // Rank 7
                    };
                    piece.move_count = if is_on_starting_rank { 0 } else { 1 };
                }
            }
        }
    }

    // 6. Apply en passant target
    if en_passant_str != "-" {
        let ep_chars: Vec<char> = en_passant_str.chars().collect();
        if ep_chars.len() != 2 {
            return Err(format!("Invalid en passant target: '{en_passant_str}'"));
        }

        let ep_file = (ep_chars[0] as usize).wrapping_sub('a' as usize);
        let ep_rank: usize = (ep_chars[1] as usize).wrapping_sub('0' as usize);

        if ep_file > 7 || !(3..=6).contains(&ep_rank) {
            return Err(format!("Invalid en passant target: '{en_passant_str}'"));
        }

        // The en passant target square is behind the pawn that just moved
        // If target is rank 3, white just moved a pawn from rank 2 to rank 4, so pawn is at rank 4 (row 4)
        // If target is rank 6, black just moved a pawn from rank 7 to rank 5, so pawn is at rank 5 (row 3)
        let pawn_row = match ep_rank {
            3 => 4, // White pawn at rank 4 = row index 4
            6 => 3, // Black pawn at rank 5 = row index 3
            _ => return Err(format!("Invalid en passant target rank: {ep_rank}")),
        };

        if let Some(piece) = &mut squares[pawn_row][ep_file] {
            if piece.piece_type == PieceType::Pawn {
                piece.move_count = 1;
                piece.last_game_move = Some(board_move_count);
            }
        }
    }

    Ok(FenParseResult {
        board: Board {
            squares,
            move_count: board_move_count,
        },
        active_color,
    })
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
            && white_king.move_count == 0
        {
            // Check kingside rook (h1 = row 7, col 7)
            if let Some(kingside_rook) = &squares[7][7] {
                if kingside_rook.piece_type == PieceType::Rook
                    && kingside_rook.color == Color::White
                    && kingside_rook.move_count == 0
                {
                    castling.push('K');
                }
            }

            // Check queenside rook (a1 = row 7, col 0)
            if let Some(queenside_rook) = &squares[7][0] {
                if queenside_rook.piece_type == PieceType::Rook
                    && queenside_rook.color == Color::White
                    && queenside_rook.move_count == 0
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
            && black_king.move_count == 0
        {
            // Check kingside rook (h8 = row 0, col 7)
            if let Some(kingside_rook) = &squares[0][7] {
                if kingside_rook.piece_type == PieceType::Rook
                    && kingside_rook.color == Color::Black
                    && kingside_rook.move_count == 0
                {
                    castling.push('k');
                }
            }

            // Check queenside rook (a8 = row 0, col 0)
            if let Some(queenside_rook) = &squares[0][0] {
                if queenside_rook.piece_type == PieceType::Rook
                    && queenside_rook.color == Color::Black
                    && queenside_rook.move_count == 0
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
                && piece.move_count == 1
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
                && piece.move_count == 1
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
            moves: Vec::new(),
            engine_result: None,
        };

        let fen = game_state_to_fen(&game_state);
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_fen_to_board_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let result = fen_to_board(fen).unwrap();

        assert_eq!(result.active_color, Color::White);
        assert_eq!(result.board.move_count, 0);

        // Verify king and rooks have move_count = 0 (castling available)
        let white_king = result.board.squares[7][4].as_ref().unwrap();
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.move_count, 0);

        let white_rook_a = result.board.squares[7][0].as_ref().unwrap();
        assert_eq!(white_rook_a.piece_type, PieceType::Rook);
        assert_eq!(white_rook_a.move_count, 0);

        // Verify pawns on starting rank have move_count = 0
        let white_pawn = result.board.squares[6][0].as_ref().unwrap();
        assert_eq!(white_pawn.piece_type, PieceType::Pawn);
        assert_eq!(white_pawn.move_count, 0);
    }

    #[test]
    fn test_fen_to_board_roundtrip() {
        let original_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let result = fen_to_board(original_fen).unwrap();

        let game_state = GameStateAtPointInTime {
            state: State::NotStarted,
            current_turn: result.active_color,
            in_check: None,
            board: result.board,
            captured_pieces: crate::types::game::CapturedPieces {
                white: Vec::new(),
                black: Vec::new(),
                white_points: 0,
                black_points: 0,
            },
            moves: Vec::new(),
            engine_result: None,
        };

        let generated_fen = game_state_to_fen(&game_state);
        assert_eq!(generated_fen, original_fen);
    }

    #[test]
    fn test_fen_to_board_black_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let result = fen_to_board(fen).unwrap();

        assert_eq!(result.active_color, Color::Black);
        assert_eq!(result.board.move_count, 1);

        // Verify e4 pawn has en passant info
        let e4_pawn = result.board.squares[4][4].as_ref().unwrap();
        assert_eq!(e4_pawn.piece_type, PieceType::Pawn);
        assert_eq!(e4_pawn.move_count, 1);
        assert_eq!(e4_pawn.last_game_move, Some(1));
    }

    #[test]
    fn test_fen_to_board_no_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1";
        let result = fen_to_board(fen).unwrap();

        // All kings and rooks should have move_count = 1 (no castling)
        let white_king = result.board.squares[7][4].as_ref().unwrap();
        assert_eq!(white_king.move_count, 1);

        let white_rook_a = result.board.squares[7][0].as_ref().unwrap();
        assert_eq!(white_rook_a.move_count, 1);
    }

    #[test]
    fn test_fen_to_board_partial_castling() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Kq - 0 1";
        let result = fen_to_board(fen).unwrap();

        // White king can castle kingside only
        let white_king = result.board.squares[7][4].as_ref().unwrap();
        assert_eq!(white_king.move_count, 0);

        let white_rook_h = result.board.squares[7][7].as_ref().unwrap();
        assert_eq!(white_rook_h.move_count, 0);

        // White queenside rook has moved
        let white_rook_a = result.board.squares[7][0].as_ref().unwrap();
        assert_eq!(white_rook_a.move_count, 1);

        // Black king can castle queenside only
        let black_king = result.board.squares[0][4].as_ref().unwrap();
        assert_eq!(black_king.move_count, 0);

        let black_rook_a = result.board.squares[0][0].as_ref().unwrap();
        assert_eq!(black_rook_a.move_count, 0);

        let black_rook_h = result.board.squares[0][7].as_ref().unwrap();
        assert_eq!(black_rook_h.move_count, 1);
    }

    #[test]
    fn test_fen_to_board_middlegame() {
        let fen = "r1bqkb1r/pppppppp/2n2n2/8/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let result = fen_to_board(fen).unwrap();

        assert_eq!(result.active_color, Color::White);
        assert_eq!(result.board.move_count, 4);

        // Verify knights are placed correctly
        let black_knight_c6 = result.board.squares[2][2].as_ref().unwrap();
        assert_eq!(black_knight_c6.piece_type, PieceType::Knight);
        assert_eq!(black_knight_c6.color, Color::Black);

        let white_knight_f3 = result.board.squares[5][5].as_ref().unwrap();
        assert_eq!(white_knight_f3.piece_type, PieceType::Knight);
        assert_eq!(white_knight_f3.color, Color::White);
    }

    #[test]
    fn test_fen_to_board_invalid_fen() {
        assert!(fen_to_board("invalid").is_err());
        assert!(fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").is_err());
        assert!(fen_to_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1").is_err());
        assert!(fen_to_board("rnbqkbnr/pppppppp/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err());
    }
}
