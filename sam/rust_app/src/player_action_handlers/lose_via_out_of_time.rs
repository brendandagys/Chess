use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::{
    helpers::game::{
        get_game, get_player_details_from_connection_id, handle_if_game_is_finished,
        notify_other_player_about_game_update, save_game, PlayerDetails,
    },
    types::{
        game::{GameEnding, State},
        piece::Color,
    },
    utils::api::build_response,
};

pub async fn lose_via_out_of_time(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    connection_id: &str,
    game_table: &str,
    user_table: &str,
    game_id: &str,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        None => build_response(
            StatusCode::NOT_FOUND,
            Some(connection_id.to_string()),
            Some(vec![format!("Game with ID `{game_id}` not found").into()]),
            None::<()>,
        ),
        Some(mut game) => {
            let Some(PlayerDetails {
                color: loser_color,
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

            game.game_state.current_state_mut().state =
                State::Finished(GameEnding::OutOfTime(loser_color));

            if let Some(game_time) = game.game_state.game_time.as_mut() {
                if loser_color == Color::White {
                    game_time.white_seconds_left = 0;
                } else {
                    game_time.black_seconds_left = 0;
                }
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

            tracing::info!("{username} lost game {game_id} via no time left");

            build_response(
                StatusCode::OK,
                Some(connection_id.to_string()),
                None,
                Some(game),
            )
        }
    }
}
