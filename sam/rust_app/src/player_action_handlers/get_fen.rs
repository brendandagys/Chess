use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;
use serde::Serialize;

use chess::{
    helpers::{board::game_state_to_fen, game::get_game},
    utils::api::build_response,
};

#[derive(Serialize)]
pub struct FenResult {
    pub fen: Option<String>,
}

pub async fn get_fen(
    dynamo_db_client: &Client,
    connection_id: &str,
    game_table: &str,
    game_id: &str,
    history_index: usize,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        Some(game) => {
            let fen = game
                .game_state
                .history
                .get(history_index)
                .and_then(|state| {
                    if state.board.is_standard_board() {
                        Some(game_state_to_fen(state))
                    } else {
                        None
                    }
                });

            build_response(
                StatusCode::OK,
                Some(connection_id.to_string()),
                None,
                Some(FenResult { fen }),
            )
        }
        None => build_response(
            StatusCode::NOT_FOUND,
            Some(connection_id.to_string()),
            Some(vec![format!("Game with ID `{game_id}` not found").into()]),
            None::<()>,
        ),
    }
}
