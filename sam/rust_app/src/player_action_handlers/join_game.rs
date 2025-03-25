use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_http::Body;
use lambda_runtime::Error;

use chess::helpers::game::{
    assign_player_to_remaining_slot, get_game, notify_other_player_about_game_update, save_game,
};
use chess::helpers::user::{create_user_game, get_user_game, save_user_record};
use chess::utils::api::build_response;

pub async fn join_game(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    user_table: &str,
    connection_id: &str,
    username: &str,
    game_id: &str,
) -> Result<ApiGatewayProxyResponse, Error> {
    let game = match get_game(&dynamo_db_client, game_table, game_id).await? {
        Some(mut existing_game) => {
            tracing::info!(
                "Found existing game (ID: {}) for user ({username}) to try to join",
                existing_game.game_id
            );

            if let Err(err) =
                assign_player_to_remaining_slot(&mut existing_game, username, connection_id)
            {
                return build_response(StatusCode::BAD_REQUEST, Some(&err.to_string()), None::<()>);
            }

            tracing::info!(
                "User ({username}) joined game (ID: {}) as {}",
                existing_game.game_id,
                if username == existing_game.white_username.as_ref().unwrap() {
                    // There should always be 2 players here
                    "white"
                } else {
                    "black"
                }
            );

            save_game(&dynamo_db_client, game_table, &existing_game).await?;

            existing_game
        }
        None => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(&format!(
                    "Game with ID `{game_id}` does not exist. Please create a new game instead."
                )),
                None::<()>,
            );
        }
    };

    // Retrieve or create a new user-game record and assign user's connection ID to it
    match get_user_game(&dynamo_db_client, user_table, username, &game.game_id).await? {
        Some(mut found_user_game) => {
            found_user_game.connection_id = Some(connection_id.to_string());

            tracing::info!(
                "Found existing user-game record for user ({username}) and game (ID: {})",
                found_user_game.sort_key.trim_end_matches("GAME-"),
            );
            save_user_record(dynamo_db_client, user_table, &found_user_game).await?;
        }
        None => {
            let new_user_game = create_user_game(&game.game_id, username, connection_id);
            save_user_record(dynamo_db_client, user_table, &new_user_game).await?;
            tracing::info!(
                "Created new user-game record for user ({username}) (ID: {})",
                new_user_game.sort_key.trim_end_matches("GAME-")
            );
        }
    };

    notify_other_player_about_game_update(sdk_config, request_context, connection_id, &game)
        .await?;

    tracing::info!("PLAYER {username} JOINED GAME (ID: {})", game.game_id);

    Ok(ApiGatewayProxyResponse {
        status_code: 200, // Doesn't seem to be used by API Gateway
        body: Some(Body::from(serde_json::to_string(&game)?)),
        ..Default::default()
    })
}
