use crate::{
    helpers::generic::{base64_to_bytes, bytes_to_base64},
    types::piece::{Color, Piece, PieceType},
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
