use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::helpers::{
    board::{decode_piece, encode_piece},
    generic::{base64_to_bytes, bytes_to_base64},
};

use super::{
    board::{Board, BoardSetup, Position},
    piece::{Color, Piece},
};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    NotStarted,
    InProgress,
    Finished(GameEnding),
}

#[derive(Clone, Debug)]
pub struct CapturedPieces {
    pub white: Vec<Piece>,
    pub white_points: u16,
    pub black: Vec<Piece>,
    pub black_points: u16,
}

impl Serialize for CapturedPieces {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let white_bytes: Vec<u8> = self.white.iter().map(|piece| encode_piece(piece)).collect();
        let black_bytes: Vec<u8> = self.black.iter().map(|piece| encode_piece(piece)).collect();

        let mut state = serializer.serialize_struct("CapturedPieces", 4)?;

        state.serialize_field("white", &bytes_to_base64(&white_bytes))?;
        state.serialize_field("whitePoints", &self.white_points)?;

        state.serialize_field("black", &bytes_to_base64(&black_bytes))?;
        state.serialize_field("blackPoints", &self.black_points)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for CapturedPieces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct CapturedPiecesVisitor;

        impl<'de> Visitor<'de> for CapturedPiecesVisitor {
            type Value = CapturedPieces;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CapturedPieces")
            }

            fn visit_map<V>(self, mut map: V) -> Result<CapturedPieces, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut white: Option<String> = None;
                let mut white_points: Option<u16> = None;
                let mut black: Option<String> = None;
                let mut black_points: Option<u16> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "white" => {
                            white = Some(map.next_value()?);
                        }
                        "whitePoints" => {
                            white_points = Some(map.next_value()?);
                        }
                        "black" => {
                            black = Some(map.next_value()?);
                        }
                        "blackPoints" => {
                            black_points = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let white_bytes = white.ok_or_else(|| de::Error::missing_field("white"))?;
                let black_bytes = black.ok_or_else(|| de::Error::missing_field("black"))?;

                let white_points =
                    white_points.ok_or_else(|| de::Error::missing_field("whitePoints"))?;

                let black_points =
                    black_points.ok_or_else(|| de::Error::missing_field("blackPoints"))?;

                let white_bytes = base64_to_bytes(&white_bytes)
                    .map_err(|_| de::Error::custom("Invalid base64 in white field"))?;

                let black_bytes = base64_to_bytes(&black_bytes)
                    .map_err(|_| de::Error::custom("Invalid base64 in black field"))?;

                let white = white_bytes
                    .into_iter()
                    .map(|b| {
                        decode_piece(b as usize)
                            .ok_or_else(|| de::Error::custom("Invalid piece byte in white field"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let black = black_bytes
                    .into_iter()
                    .map(|b| {
                        decode_piece(b as usize)
                            .ok_or_else(|| de::Error::custom("Invalid piece byte in black field"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(CapturedPieces {
                    white,
                    white_points,
                    black,
                    black_points,
                })
            }
        }

        deserializer.deserialize_struct(
            "CapturedPieces",
            &["white", "whitePoints", "black", "blackPoints"],
            CapturedPiecesVisitor,
        )
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorPreference {
    White,
    Black,
    Random,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTime {
    pub both_players_last_connected_at: Option<String>,
    pub last_move_at: Option<String>,
    pub white_seconds_left: usize,
    pub black_seconds_left: usize,
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
    pub game_time: Option<GameTime>,
    pub history: Vec<GameStateAtPointInTime>,
}

impl GameState {
    pub fn new(
        game_id: String,
        board_setup: &BoardSetup,
        seconds_per_player: Option<usize>,
    ) -> Self {
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
            game_time: seconds_per_player.map(|seconds| GameTime {
                both_players_last_connected_at: None,
                last_move_at: None,
                white_seconds_left: seconds,
                black_seconds_left: seconds,
            }),
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
        color_preference: ColorPreference,
        seconds_per_player: Option<usize>,
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
    LoseViaOutOfTime {
        game_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Resign {
        game_id: String,
    },
    #[serde(rename_all = "camelCase")]
    OfferDraw {
        game_id: String,
    },
}
