use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::{
    helpers::{
        game::{
            get_game, get_player_details_from_connection_id,
            mark_user_as_disconnected_and_notify_other_player, PlayerDetails,
        },
        user::{get_user_game, save_user_record},
    },
    utils::api::build_response,
};

pub async fn leave_game(
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
            let Some(PlayerDetails { username, .. }) =
                get_player_details_from_connection_id(&game, connection_id)
            else {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec!["You are not a player in this game".into()]),
                    None::<()>,
                );
            };

            let mut user_game = get_user_game(dynamo_db_client, user_table, &username, game_id)
                .await?
                .unwrap_or_else(|| {
                    panic!("User game should exist for player {username} leaving game {game_id}")
                });

            user_game.connection_id = Some("<disconnected>".to_string());
            save_user_record(dynamo_db_client, user_table, &user_game).await?;

            mark_user_as_disconnected_and_notify_other_player(
                sdk_config,
                request_context,
                dynamo_db_client,
                game_table,
                &mut game,
                &username,
            )
            .await?;

            tracing::info!("{username} left game {game_id}");

            build_response(
                StatusCode::OK,
                Some(connection_id.to_string()),
                None,
                None::<()>,
            )
        }
    }
}
