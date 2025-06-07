use serde::{Deserialize, Serialize};

use super::{
    board::{Board, BoardSetup, Position},
    piece::{Color, Piece},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    NotStarted,
    InProgress,
    Finished(GameEnding),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturedPieces {
    pub white: Vec<Piece>,
    pub white_points: u16,
    pub black: Vec<Piece>,
    pub black_points: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateAtPointInTime {
    pub state: State,
    pub current_turn: Color,
    pub in_check: Option<Color>,
    pub board: Board,
    pub captured_pieces: CapturedPieces,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub game_id: String,
    pub history: Vec<GameStateAtPointInTime>,
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
            history: vec![GameStateAtPointInTime {
                state: State::NotStarted,
                current_turn: Color::White,
                in_check: None,
                board: board.clone(),
                captured_pieces: captured_pieces.clone(),
            }],
        }
    }

    pub fn current_state(&self) -> &GameStateAtPointInTime {
        self.history
            .last()
            .expect("Game history should not be empty")
    }

    pub fn current_state_mut(&mut self) -> &mut GameStateAtPointInTime {
        self.history
            .last_mut()
            .expect("Game history should not be empty")
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
    JoinGame {
        username: String,
        game_id: String,
    },
    #[serde(rename_all = "camelCase")]
    GetGameState {
        game_id: String,
    },
    #[serde(rename_all = "camelCase")]
    MovePiece {
        game_id: String,
        player_move: PlayerMove,
    },
    Heartbeat,
    #[serde(rename_all = "camelCase")]
    Resign {
        game_id: String,
    },
    #[serde(rename_all = "camelCase")]
    OfferDraw {
        game_id: String,
    },
}
