use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use chess::types::board::BoardSetup;
use lambda_http::http::StatusCode;
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
    seconds_per_player: Option<usize>,
) -> Result<ApiGatewayProxyResponse, Error> {
    if username.trim().is_empty() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec!["Must provide a username".into()]),
            None::<()>,
        );
    }

    let new_game = match game_id {
        Some(game_id) => {
            if (get_game(dynamo_db_client, game_table, game_id).await?).is_some() {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![format!(
                        "Game with ID `{game_id}` already exists. Please join the game instead."
                    )
                    .into()]),
                    None::<()>,
                );
            }

            create_game(
                Some(game_id),
                username,
                board_setup,
                color_preference,
                seconds_per_player,
                connection_id,
            )
        }
        None => create_game(
            None,
            username,
            board_setup,
            color_preference,
            seconds_per_player,
            connection_id,
        ),
    };

    save_game(dynamo_db_client, game_table, &new_game).await?;

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

    build_response(
        StatusCode::OK,
        Some(connection_id.to_string()),
        None,
        Some(new_game),
    )
}
