use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use chess::types::board::BoardSetup;
use lambda_http::http::StatusCode;
use lambda_http::Body;
use lambda_runtime::Error;

use chess::helpers::game::{create_game, get_game, save_game};
use chess::helpers::user::{create_user_game, save_user_record};
use chess::types::piece::Color;
use chess::utils::api::build_response;

pub async fn create_new_game(
    dynamo_db_client: &Client,
    game_table: &str,
    user_table: &str,
    connection_id: &str,
    username: &str,
    game_id: Option<&str>,
    board_setup: Option<BoardSetup>,
    color_preference: Option<Color>,
) -> Result<ApiGatewayProxyResponse, Error> {
    if username.trim().is_empty() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some("Must provide a username"),
            None::<()>,
        );
    }

    let new_game = match game_id {
        Some(game_id) => {
            if let Some(_) = get_game(&dynamo_db_client, game_table, game_id).await? {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(&format!(
                        "Game with ID `{game_id}` already exists. Please join the game instead."
                    )),
                    None::<()>,
                );
            }

            create_game(
                Some(game_id),
                username,
                board_setup,
                color_preference,
                connection_id,
            )
        }
        None => create_game(None, username, board_setup, color_preference, connection_id),
    };

    save_game(&dynamo_db_client, game_table, &new_game).await?;

    tracing::info!(
        "Created new game (ID: {}) for user ({username})",
        new_game.game_id
    );

    let new_user_game = create_user_game(&new_game.game_id, username, connection_id);
    save_user_record(dynamo_db_client, user_table, &new_user_game).await?;

    tracing::info!(
        "Created new user-game record for {username} (ID: {})",
        new_user_game.sort_key
    );

    tracing::info!(
        "PLAYER {username} CREATED NEW GAME (ID: {})",
        new_game.game_id
    );

    Ok(ApiGatewayProxyResponse {
        status_code: 200, // Doesn't seem to be used by API Gateway
        body: Some(Body::from(serde_json::to_string(&new_game)?)),
        ..Default::default()
    })
}
