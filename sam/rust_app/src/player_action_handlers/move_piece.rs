use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::{
    helpers::{
        game::{
            can_player_make_a_move, get_game, get_player_details_from_connection_id, make_move,
            notify_other_player_about_game_update, save_game, validate_move, PlayerDetails,
        },
        user::{get_user_game, save_user_record},
    },
    types::game::{GameEnding, GameState, PlayerMove, State},
    utils::api::build_response,
};

async fn handle_if_game_is_finished(
    dynamo_db_client: &Client,
    user_table: &str,
    username: &str,
    game_state: &GameState,
) -> Result<(), Error> {
    if let State::Finished(GameEnding::Checkmate(checkmated_color)) =
        game_state.current_state().state
    {
        let mut user_game =
            get_user_game(dynamo_db_client, user_table, &username, &game_state.game_id)
                .await?
                .expect(&format!(
                    "User ({username}) does not have a user-game record for game `{}`",
                    game_state.game_id
                ));

        user_game.winner = Some(checkmated_color.opponent_color().to_string());
        save_user_record(dynamo_db_client, user_table, &user_game).await?;
    }

    Ok(())
}

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
            }) = get_player_details_from_connection_id(&game, connection_id)
            else {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![format!("You are not a player in this game").into()]),
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

            save_game(&dynamo_db_client, game_table, &game).await?;

            handle_if_game_is_finished(dynamo_db_client, user_table, &username, &game.game_state)
                .await?;

            notify_other_player_about_game_update(
                &sdk_config,
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
