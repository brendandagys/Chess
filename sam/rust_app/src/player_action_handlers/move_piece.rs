use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::{
    helpers::game::{
        can_player_make_a_move, get_engine_result_if_turn, get_game,
        get_next_move_from_engine_search_result, get_player_details_from_connection_id,
        handle_if_game_is_finished, make_move, notify_player_about_game_update, save_game,
        update_game_time_after_engine_move, validate_move, PlayerDetails,
    },
    types::game::{PlayerMove, SearchStatistics},
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

            if game.engine_difficulty.is_some() {
                if let Some(search_result) =
                    get_engine_result_if_turn(sdk_config, request_context, &mut game, connection_id)
                        .await?
                {
                    let engine_move = get_next_move_from_engine_search_result(&search_result);

                    update_game_time_after_engine_move(&mut game.game_state, search_result.time_ms);

                    if let Err(e) = make_move(&mut game.game_state, &engine_move) {
                        return build_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Some(connection_id.to_string()),
                            Some(vec![e.into()]),
                            Some(game),
                        );
                    }

                    game.game_state.current_state_mut().engine_result = Some(SearchStatistics {
                        depth: search_result.depth,
                        nodes: search_result.nodes,
                        qnodes: search_result.qnodes,
                        time_ms: search_result.time_ms,
                        from_book: search_result.from_book,
                    });
                }
            }

            save_game(dynamo_db_client, game_table, &game).await?;

            handle_if_game_is_finished(
                dynamo_db_client,
                user_table,
                &username,
                opponent_username.as_deref(),
                &game.game_state,
            )
            .await?;

            notify_player_about_game_update(
                sdk_config,
                request_context,
                connection_id,
                &game,
                None,
                false,
            )
            .await?;

            tracing::info!("PLAYER {username} MADE A MOVE (GAME ID: {game_id}): {player_move:?}. Game state: {game:?}");

            build_response(
                StatusCode::OK,
                Some(connection_id.to_string()),
                None,
                Some(game),
            )
        }
    }
}
