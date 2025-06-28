use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::{
    helpers::game::{
        can_player_make_a_move, get_game, get_player_details_from_connection_id,
        handle_if_game_is_finished, make_move, notify_other_player_about_game_update, save_game,
        validate_move, PlayerDetails,
    },
    types::game::PlayerMove,
    utils::api::build_response,
};

pub async fn move_piece(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    user_table: &str,
    connection_id: &str,
    game_id: &str,
    player_move: PlayerMove,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        None => build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec![format!("Game (ID: {game_id}) not found").into()]),
            None::<()>,
        ),
        Some(mut game) => {
            let Some(PlayerDetails {
                color: player_color,
                username,
                opponent_username,
            }) = get_player_details_from_connection_id(&game, connection_id)
            else {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec!["You are not a player in this game".into()]),
                    None::<()>,
                );
            };

            if let Err(e) = can_player_make_a_move(&game, &player_color) {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![e.into()]),
                    Some(game),
                );
            }

            if let Err(e) = validate_move(
                &game.game_state.current_state().board,
                &player_move,
                &player_color,
            ) {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![e.into()]),
                    Some(game),
                );
            }

            if let Err(e) = make_move(&mut game.game_state, &player_move) {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![e.into()]),
                    Some(game),
                );
            }

            save_game(dynamo_db_client, game_table, &game).await?;

            handle_if_game_is_finished(
                dynamo_db_client,
                user_table,
                &username,
                opponent_username.as_deref().unwrap_or_else(|| {
                    panic!("Opponent username should be set for game ID: {game_id}")
                }),
                &game.game_state,
            )
            .await?;

            notify_other_player_about_game_update(
                sdk_config,
                request_context,
                connection_id,
                &game,
                None,
            )
            .await?;

            tracing::info!(
        "PLAYER {username} MADE A MOVE (GAME ID: {game_id}): {player_move:?}. Game state: {game:?}"
    );

            build_response(
                StatusCode::OK,
                Some(connection_id.to_string()),
                None,
                Some(game),
            )
        }
    }
}
