use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::{
    helpers::game::{
        get_game, get_player_details_from_connection_id, is_game_over,
        notify_player_about_game_update, save_game, PlayerDetails,
    },
    utils::api::build_response,
};

pub async fn decline_draw(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    connection_id: &str,
    game_table: &str,
    game_id: &str,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        None => build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec![format!("Game with ID `{game_id}` not found").into()]),
            None::<()>,
        ),
        Some(mut game) => {
            let Some(PlayerDetails {
                color: player_color,
                username,
                ..
            }) = get_player_details_from_connection_id(&game, connection_id)
            else {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec!["You are not a player in this game".into()]),
                    None::<()>,
                );
            };

            if is_game_over(&game) {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec!["Game is already over".into()]),
                    None::<()>,
                );
            }

            match game.draw_offered_by {
                Some(color) if color != player_color => {}
                _ => {
                    return build_response(
                        StatusCode::BAD_REQUEST,
                        Some(connection_id.to_string()),
                        Some(vec!["No draw offer to decline".into()]),
                        None::<()>,
                    );
                }
            }

            game.draw_offered_by = None;

            save_game(dynamo_db_client, game_table, &game).await?;

            notify_player_about_game_update(
                sdk_config,
                request_context,
                connection_id,
                &game,
                None,
                false,
            )
            .await?;

            tracing::info!("Player {username} declined a draw in game {game_id}");

            build_response(
                StatusCode::OK,
                Some(connection_id.to_string()),
                None,
                Some(game),
            )
        }
    }
}
