use serde::{Deserialize, Serialize};

use super::{
    board::{Board, BoardSetup, Position},
    piece::{Color, Piece},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    NotStarted,
    InProgress,
    Finished(GameEnding),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPieces {
    pub white: Vec<Piece>,
    pub white_points: u16,
    pub black: Vec<Piece>,
    pub black_points: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub game_id: String,
    pub state: State,
    pub current_turn: Color,
    pub in_check: Option<Color>,
    pub board: Board,
    pub move_history: Vec<(Position, Position)>, // TODO: Implement
    pub captured_pieces: CapturedPieces,
}

impl GameState {
    pub fn new(game_id: String, board_setup: &BoardSetup) -> Self {
        let board = Board::new(board_setup);

        let captured_pieces = CapturedPieces {
            white: Vec::new(),
            black: Vec::new(),
            white_points: 0,
            black_points: 0,
        };

        GameState {
            game_id,
            state: State::NotStarted,
            current_turn: Color::White,
            in_check: None,
            board,
            move_history: Vec::new(),
            captured_pieces,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerMove {
    pub from: Position,
    pub to: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlayerAction {
    #[serde(rename_all = "camelCase")]
    CreateGame {
        username: String,
        game_id: Option<String>,
        board_setup: Option<BoardSetup>,
        color_preference: Option<Color>,
    },
    #[serde(rename_all = "camelCase")]
    JoinGame { username: String, game_id: String },
    #[serde(rename_all = "camelCase")]
    GetGameState { game_id: String },
    #[serde(rename_all = "camelCase")]
    MovePiece {
        game_id: String,
        player_move: PlayerMove,
    },
    #[serde(rename_all = "camelCase")]
    Resign { game_id: String },
    #[serde(rename_all = "camelCase")]
    OfferDraw { game_id: String },
}
