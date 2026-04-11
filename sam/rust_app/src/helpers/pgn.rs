use crate::types::board::{Board, File, Position, Rank};
use crate::types::dynamo_db::GameRecord;
use crate::types::game::{GameEnding, GameState, GameStateAtPointInTime, PlayerMove, State};
use crate::types::piece::{Color, PieceType};

/// Convert a PieceType to SAN letter. Pawns have no letter.
fn piece_to_san_char(piece_type: &PieceType) -> &'static str {
    match piece_type {
        PieceType::King => "K",
        PieceType::Queen => "Q",
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        PieceType::Pawn => "",
    }
}

/// Convert a promotion char from UCI (q/r/b/n) to SAN suffix (=Q, =R, =B, =N).
fn promotion_suffix(ch: char) -> &'static str {
    match ch {
        'q' => "=Q",
        'r' => "=R",
        'b' => "=B",
        'n' => "=N",
        _ => "",
    }
}

/// Parse a UCI algebraic position like "e4" to a Position.
fn algebraic_to_position(s: &str) -> Position {
    let bytes = s.as_bytes();
    let file = (bytes[0] - b'a') as usize + 1;
    let rank = (bytes[1] - b'0') as usize;
    Position {
        rank: Rank(rank),
        file: File(file),
    }
}

fn position_to_algebraic(pos: &Position) -> String {
    let file_char = (b'a' + (pos.file.0 - 1) as u8) as char;
    format!("{}{}", file_char, pos.rank.0)
}

/// Convert a single UCI move to Standard Algebraic Notation.
///
/// `board_before` is the board state before the move was applied.
/// `state_after` is the full game state after the move was applied (used for check/checkmate).
pub fn uci_to_san(
    board_before: &Board,
    uci: &str,
    color: &Color,
    state_after: &GameStateAtPointInTime,
) -> String {
    let from = algebraic_to_position(&uci[0..2]);
    let to = algebraic_to_position(&uci[2..4]);
    let promotion_char = uci.chars().nth(4);

    let piece = match board_before.get_piece_at_position(&from) {
        Some(p) => p,
        None => return uci.to_string(), // Fallback to UCI if something is unexpected
    };

    let mut san = String::new();

    // Castling: king moving 2+ files
    if piece.piece_type == PieceType::King {
        let file_diff = (to.file.0 as isize - from.file.0 as isize).unsigned_abs();

        if file_diff > 1 {
            if to.file.0 > from.file.0 {
                san.push_str("O-O");
            } else {
                san.push_str("O-O-O");
            }
            append_check_suffix(&mut san, state_after);
            return san;
        }
    }

    let is_capture = board_before.get_piece_at_position(&to).is_some()
        || (piece.piece_type == PieceType::Pawn && from.file != to.file);

    if piece.piece_type == PieceType::Pawn {
        if is_capture {
            let file_char = (b'a' + (from.file.0 - 1) as u8) as char;
            san.push(file_char);
        }
    } else {
        san.push_str(piece_to_san_char(&piece.piece_type));

        // Disambiguation: find other pieces of the same type and color that can
        // also reach the destination square with a legal move.
        let disambiguation =
            compute_disambiguation(board_before, &piece.piece_type, color, &from, &to);
        san.push_str(&disambiguation);
    }

    if is_capture {
        san.push('x');
    }

    san.push_str(&position_to_algebraic(&to));

    if let Some(promo) = promotion_char {
        san.push_str(promotion_suffix(promo));
    }

    append_check_suffix(&mut san, state_after);

    san
}

/// Determine the disambiguation string needed for a non-pawn move.
fn compute_disambiguation(
    board: &Board,
    piece_type: &PieceType,
    color: &Color,
    from: &Position,
    to: &Position,
) -> String {
    let same_type_pieces: Vec<Position> = board
        .get_all_pieces(Some(color))
        .into_iter()
        .filter(|(p, pos)| p.piece_type == *piece_type && *pos != *from)
        .filter(|(p, pos)| {
            // Check if this piece can reach the destination
            let can_reach = p
                .possible_moves(board, pos, false)
                .iter()
                .any(|m| *m == *to);
            if !can_reach {
                return false;
            }
            // Also verify the move is legal (doesn't leave own king in check)
            let mut hypothetical = board.clone();
            hypothetical.apply_move(
                &PlayerMove {
                    from: pos.clone(),
                    to: to.clone(),
                },
                false,
            );

            !hypothetical.is_king_in_check(color)
        })
        .map(|(_, pos)| pos)
        .collect();

    if same_type_pieces.is_empty() {
        return String::new();
    }

    let same_file = same_type_pieces.iter().any(|p| p.file == from.file);
    let same_rank = same_type_pieces.iter().any(|p| p.rank == from.rank);

    let from_file_char = (b'a' + (from.file.0 - 1) as u8) as char;

    if !same_file {
        // File alone is sufficient
        from_file_char.to_string()
    } else if !same_rank {
        // Rank alone is sufficient
        from.rank.0.to_string()
    } else {
        // Need both file and rank
        format!("{}{}", from_file_char, from.rank.0)
    }
}

fn append_check_suffix(san: &mut String, state_after: &GameStateAtPointInTime) {
    match state_after.state {
        State::Finished(GameEnding::Checkmate(_)) => san.push('#'),
        _ if state_after.in_check.is_some() => san.push('+'),
        _ => {}
    }
}

/// Convert the game result to PGN result token.
fn game_result_to_pgn(game_state: &GameState) -> &'static str {
    match game_state.current_state().state {
        State::Finished(GameEnding::Checkmate(losing_color))
        | State::Finished(GameEnding::Resignation(losing_color))
        | State::Finished(GameEnding::OutOfTime(losing_color)) => match losing_color {
            Color::White => "0-1",
            Color::Black => "1-0",
        },
        State::Finished(GameEnding::Stalemate)
        | State::Finished(GameEnding::DrawByThreefoldRepetition)
        | State::Finished(GameEnding::DrawByFiftyMoveRule)
        | State::Finished(GameEnding::DrawByInsufficientMaterial)
        | State::Finished(GameEnding::DrawByMutualAgreement) => "1/2-1/2",
        _ => "*",
    }
}

/// Build the SAN move list on-the-fly from UCI moves and board history.
/// Used as a fallback for games that were created before `san_list` was stored.
fn compute_san_list_from_history(game_state: &GameState) -> Vec<String> {
    let mut san_moves = Vec::with_capacity(game_state.move_list.len());

    for (i, uci) in game_state.move_list.iter().enumerate() {
        let state_before = &game_state.history[i];
        let state_after = &game_state.history[i + 1];
        let color = &state_before.current_turn;

        san_moves.push(uci_to_san(&state_before.board, uci, color, state_after));
    }

    san_moves
}

/// Generate a PGN string for a game record.
/// Returns `None` if the board is not a standard 8×8 board.
pub fn game_to_pgn(game: &GameRecord) -> Option<String> {
    if !game
        .game_state
        .history
        .first()
        .map_or(false, |s| s.board.is_standard_board())
    {
        return None;
    }

    let game_state = &game.game_state;
    let result = game_result_to_pgn(game_state);

    // Parse the ISO 8601 date → PGN date format (YYYY.MM.DD)
    let pgn_date = if game.created.len() >= 10 {
        game.created[..10].replace('-', ".")
    } else {
        "????.??.??".to_string()
    };

    let white = game.white_username.as_deref().unwrap_or("?");
    let black = game.black_username.as_deref().unwrap_or("?");

    // Build PGN headers (Seven Tag Roster)
    let mut pgn = String::new();
    pgn.push_str(&format!("[Event \"Live Chess\"]\n"));
    pgn.push_str(&format!("[Site \"chess.brendandagys.com\"]\n"));
    pgn.push_str(&format!("[Date \"{pgn_date}\"]\n"));
    pgn.push_str(&format!("[Round \"-\"]\n"));
    pgn.push_str(&format!("[White \"{white}\"]\n"));
    pgn.push_str(&format!("[Black \"{black}\"]\n"));
    pgn.push_str(&format!("[Result \"{result}\"]\n"));

    // Optional tags
    if let Some(seconds) = game.seconds_per_player {
        pgn.push_str(&format!("[TimeControl \"{seconds}s\"]\n"));
    }

    if let Some(opening) = &game_state.opening {
        if !opening.eco.is_empty() {
            pgn.push_str(&format!("[ECO \"{}\"]\n", opening.eco));
        }
        pgn.push_str(&format!("[Opening \"{}\"]\n", opening.name));
    }

    pgn.push('\n');

    // Use stored SAN list if available, otherwise compute from history
    let san_list = if game_state.san_list.is_empty() {
        compute_san_list_from_history(game_state)
    } else {
        game_state.san_list.clone()
    };

    // Build movetext with line wrapping at ~80 chars
    let mut line = String::new();
    for (i, san) in san_list.iter().enumerate() {
        let move_num = i / 2 + 1;
        let token = if i % 2 == 0 {
            format!("{}. {}", move_num, san)
        } else {
            san.to_string()
        };

        if line.is_empty() {
            line.push_str(&token);
        } else if line.len() + 1 + token.len() > 80 {
            pgn.push_str(&line);
            pgn.push('\n');
            line = token;
        } else {
            line.push(' ');
            line.push_str(&token);
        }
    }

    if !line.is_empty() {
        pgn.push_str(&line);
        pgn.push(' ');
    }
    pgn.push_str(result);
    pgn.push('\n');

    Some(pgn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::game::CapturedPieces;
    use crate::types::piece::Piece;

    fn standard_board() -> Board {
        Board::new(&crate::types::board::BoardSetup::Standard)
    }

    fn empty_state(board: Board, turn: Color) -> GameStateAtPointInTime {
        GameStateAtPointInTime {
            state: State::InProgress,
            current_turn: turn,
            in_check: None,
            board,
            captured_pieces: CapturedPieces {
                white: vec![],
                black: vec![],
                white_points: 0,
                black_points: 0,
            },
            moves: vec![],
            engine_result: None,
        }
    }

    #[test]
    fn test_pawn_push() {
        let board = standard_board();
        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "e2e4", &Color::White, &after);
        assert_eq!(san, "e4");
    }

    #[test]
    fn test_knight_move() {
        let board = standard_board();
        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "g1f3", &Color::White, &after);
        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_pawn_capture() {
        // Set up a board where e4 pawn can capture d5 pawn
        let mut board = standard_board();
        // Place white pawn on e4
        board.set_piece_at_position(
            &Position {
                rank: Rank(2),
                file: File(5),
            },
            None,
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(4),
                file: File(5),
            },
            Some(Piece::new(PieceType::Pawn, Color::White)),
        );
        // Place black pawn on d5
        board.set_piece_at_position(
            &Position {
                rank: Rank(7),
                file: File(4),
            },
            None,
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(5),
                file: File(4),
            },
            Some(Piece::new(PieceType::Pawn, Color::Black)),
        );

        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "e4d5", &Color::White, &after);
        assert_eq!(san, "exd5");
    }

    #[test]
    fn test_kingside_castling() {
        // Set up a board where White can castle kingside
        let mut board = standard_board();
        // Clear f1, g1
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(6),
            },
            None,
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(7),
            },
            None,
        );

        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "e1g1", &Color::White, &after);
        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_queenside_castling() {
        let mut board = standard_board();
        // Clear b1, c1, d1
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(2),
            },
            None,
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(3),
            },
            None,
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(4),
            },
            None,
        );

        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "e1c1", &Color::White, &after);
        assert_eq!(san, "O-O-O");
    }

    #[test]
    fn test_check_suffix() {
        let board = standard_board();
        let mut after = empty_state(standard_board(), Color::Black);
        after.in_check = Some(Color::Black);
        let san = uci_to_san(&board, "g1f3", &Color::White, &after);
        assert_eq!(san, "Nf3+");
    }

    #[test]
    fn test_checkmate_suffix() {
        let board = standard_board();
        let mut after = empty_state(standard_board(), Color::Black);
        after.state = State::Finished(GameEnding::Checkmate(Color::Black));
        let san = uci_to_san(&board, "g1f3", &Color::White, &after);
        assert_eq!(san, "Nf3#");
    }

    #[test]
    fn test_promotion() {
        // White pawn on e7, promote to queen on e8
        let mut board = Board::new(&crate::types::board::BoardSetup::Standard);
        // Clear everything, place a white pawn on e7
        for rank in 1..=8 {
            for file in 1..=8 {
                board.set_piece_at_position(
                    &Position {
                        rank: Rank(rank),
                        file: File(file),
                    },
                    None,
                );
            }
        }
        board.set_piece_at_position(
            &Position {
                rank: Rank(7),
                file: File(5),
            },
            Some(Piece::new(PieceType::Pawn, Color::White)),
        );
        // Need kings for the state
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(5),
            },
            Some(Piece::new(PieceType::King, Color::White)),
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(8),
                file: File(1),
            },
            Some(Piece::new(PieceType::King, Color::Black)),
        );

        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "e7e8q", &Color::White, &after);
        assert_eq!(san, "e8=Q");
    }

    #[test]
    fn test_rook_disambiguation_by_file() {
        // Two white rooks on a1 and h1, moving to e1
        let mut board = Board::new(&crate::types::board::BoardSetup::Standard);
        for rank in 1..=8 {
            for file in 1..=8 {
                board.set_piece_at_position(
                    &Position {
                        rank: Rank(rank),
                        file: File(file),
                    },
                    None,
                );
            }
        }
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(1),
            },
            Some(Piece::new(PieceType::Rook, Color::White)),
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(8),
            },
            Some(Piece::new(PieceType::Rook, Color::White)),
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(5),
            },
            Some(Piece::new(PieceType::King, Color::White)),
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(8),
                file: File(5),
            },
            Some(Piece::new(PieceType::King, Color::Black)),
        );

        // Move king out of the way first
        board.set_piece_at_position(
            &Position {
                rank: Rank(1),
                file: File(5),
            },
            None,
        );
        board.set_piece_at_position(
            &Position {
                rank: Rank(2),
                file: File(5),
            },
            Some(Piece::new(PieceType::King, Color::White)),
        );

        let after = empty_state(standard_board(), Color::Black);
        let san = uci_to_san(&board, "a1e1", &Color::White, &after);
        assert_eq!(san, "Rae1");
    }
}
