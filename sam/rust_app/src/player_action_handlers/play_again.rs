use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::{http::StatusCode, Body};
use lambda_runtime::Error;

use chess::helpers::game::{
    assign_player_to_existing_or_remaining_slot, check_if_both_players_just_joined, create_game,
    get_game, is_game_over, save_game,
};
use chess::helpers::user::{create_user_game, save_user_record};
use chess::types::api::{ApiMessage, ApiMessageType, ApiResponse};
use chess::utils::api::build_response;
use chess::utils::api_gateway::post_to_connection;

pub async fn play_again(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    user_table: &str,
    connection_id: &str,
    game_id: &str,
) -> Result<ApiGatewayProxyResponse, Error> {
    let old_game = match get_game(dynamo_db_client, game_table, game_id).await? {
        Some(game) => game,
        None => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.to_string()),
                Some(vec![format!("Game `{game_id}` not found").into()]),
                None::<()>,
            );
        }
    };

    if !is_game_over(&old_game) {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec!["Game is not finished".into()]),
            None::<()>,
        );
    }

    if old_game.engine_difficulty.is_some() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec!["Use create-game for engine rematches".into()]),
            None::<()>,
        );
    }

    // Determine which player is requesting and which is the opponent
    let (requester_username, opponent_username, opponent_connection_id) =
        if old_game.white_connection_id.as_deref() == Some(connection_id) {
            (
                old_game.white_username.as_deref(),
                old_game.black_username.as_deref(),
                old_game.black_connection_id.as_deref(),
            )
        } else if old_game.black_connection_id.as_deref() == Some(connection_id) {
            (
                old_game.black_username.as_deref(),
                old_game.white_username.as_deref(),
                old_game.white_connection_id.as_deref(),
            )
        } else {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.to_string()),
                Some(vec!["You are not a player in this game".into()]),
                None::<()>,
            );
        };

    let requester_username = match requester_username {
        Some(u) => u.to_string(),
        None => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.to_string()),
                Some(vec!["Missing username".into()]),
                None::<()>,
            );
        }
    };

    let opponent_username = match opponent_username {
        Some(u) => u.to_string(),
        None => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.to_string()),
                Some(vec!["Opponent username not found".into()]),
                None::<()>,
            );
        }
    };

    let opponent_connection_id = match opponent_connection_id {
        Some(id) if id != "<disconnected>" => id.to_string(),
        _ => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.to_string()),
                Some(vec!["Opponent is not connected".into()]),
                None::<()>,
            );
        }
    };

    // Create new game with same settings, requester as first player
    let mut new_game = create_game(
        None,
        &requester_username,
        Some(old_game.board_setup),
        Some(old_game.color_preference),
        None, // No engine for human rematch
        old_game.seconds_per_player,
        connection_id,
    );

    // Auto-join opponent
    assign_player_to_existing_or_remaining_slot(
        &mut new_game,
        &opponent_username,
        &opponent_connection_id,
    )?;

    // Both players present — start the game
    check_if_both_players_just_joined(&mut new_game);

    save_game(dynamo_db_client, game_table, &new_game).await?;

    tracing::info!(
        "Created rematch game (ID: {}) from old game (ID: {game_id})",
        new_game.game_id
    );

    // Create user-game records for both players
    let requester_user_game =
        create_user_game(&new_game.game_id, &requester_username, connection_id);
    save_user_record(dynamo_db_client, user_table, &requester_user_game).await?;

    let opponent_user_game = create_user_game(
        &new_game.game_id,
        &opponent_username,
        &opponent_connection_id,
    );
    save_user_record(dynamo_db_client, user_table, &opponent_user_game).await?;

    // Notify opponent with the new game, including which old game it replaces
    let _ = post_to_connection(
        sdk_config,
        request_context,
        &opponent_connection_id,
        &ApiResponse {
            status_code: 200,
            connection_id: Some(opponent_connection_id.clone()),
            messages: vec![ApiMessage {
                message: format!("{requester_username} wants a rematch!"),
                message_type: ApiMessageType::Success,
            }],
            data: Some(&new_game),
            replaces_game_id: Some(game_id.to_string()),
        },
    )
    .await?;

    tracing::info!(
        "PLAYER {requester_username} STARTED REMATCH (old: {game_id}, new: {})",
        new_game.game_id
    );

    // Return new game to requester — build response manually to include replaces_game_id
    let body = serde_json::to_string(&ApiResponse {
        status_code: 200,
        connection_id: Some(connection_id.to_string()),
        messages: Vec::new(),
        data: Some(new_game),
        replaces_game_id: Some(game_id.to_string()),
    })?;

    let mut response = ApiGatewayProxyResponse::default();
    response.status_code = 200;
    response.body = Some(Body::from(body));

    Ok(response)
}
