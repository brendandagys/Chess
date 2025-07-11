use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use chess::types::api::ApiMessage;
use chess::types::dynamo_db::GameRecord;
use chess::types::game::State;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::helpers::game::{
    assign_player_to_existing_or_remaining_slot, get_game, notify_other_player_about_game_update,
    save_game,
};
use chess::helpers::user::{create_user_game, get_user_game, save_user_record};
use chess::utils::api::build_response;

fn check_if_both_players_just_joined(game_record: &mut GameRecord) {
    if game_record
        .white_connection_id
        .as_deref()
        .unwrap_or("<disconnected>")
        != "<disconnected>"
        && game_record
            .black_connection_id
            .as_deref()
            .unwrap_or("<disconnected>")
            != "<disconnected>"
    {
        if let Some(game_time) = &mut game_record.game_state.game_time {
            game_time.both_players_last_connected_at = Some(chrono::Utc::now().to_rfc3339());
        }

        let current_state = game_record.game_state.current_state_mut();

        if current_state.state == State::NotStarted {
            current_state.state = State::InProgress;
        }
    }
}

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
    if username.trim().is_empty() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec!["Must provide a username".into()]),
            None::<()>,
        );
    }

    let game = match get_game(dynamo_db_client, game_table, game_id).await? {
        Some(mut existing_game) => {
            tracing::info!(
                "Found existing game (ID: {}) for user ({username}) to try to join",
                existing_game.game_id
            );

            if let Err(err) = assign_player_to_existing_or_remaining_slot(
                &mut existing_game,
                username,
                connection_id,
            ) {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![err.to_string().into()]),
                    None::<()>,
                );
            }

            check_if_both_players_just_joined(&mut existing_game);

            tracing::info!(
                "User ({username}) joined game (ID: {}) as {}",
                existing_game.game_id,
                if existing_game
                    .white_username
                    .as_ref()
                    .is_some_and(|u| u == username)
                {
                    "white"
                } else {
                    "black"
                }
            );

            save_game(dynamo_db_client, game_table, &existing_game).await?;

            existing_game
        }
        None => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.to_string()),
                Some(vec![format!(
                    "Game with ID `{game_id}` does not exist. Please create a new game instead."
                )
                .into()]),
                None::<()>,
            );
        }
    };

    // Retrieve or create a new user-game record and assign user's connection ID to it
    match get_user_game(dynamo_db_client, user_table, username, &game.game_id).await? {
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

    notify_other_player_about_game_update(
        sdk_config,
        request_context,
        connection_id,
        &game,
        Some(vec![ApiMessage {
            message: format!("{username} has joined the game"),
            message_type: chess::types::api::ApiMessageType::Success,
        }]),
    )
    .await?;

    tracing::info!("PLAYER {username} JOINED GAME (ID: {})", game.game_id);

    build_response(
        StatusCode::OK,
        Some(connection_id.to_string()),
        None,
        Some(game),
    )
}
