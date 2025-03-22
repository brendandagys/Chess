use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::Body;
use lambda_runtime::Error;

use crate::{
    helpers::game::{
        can_player_make_move, check_game_state, get_game, get_username_for_connection_id,
        make_move, notify_other_player_about_game_update, save_game,
    },
    types::game::PlayerMove,
};

pub async fn move_piece(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    connection_id: &str,
    game_id: &str,
    player_move: PlayerMove,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        None => {
            return Err(Error::from(format!("Game with ID {game_id} not found",)));
        }
        Some(mut game) => {
            let username = get_username_for_connection_id(&game, connection_id)
                .ok_or_else(|| Error::from("Player not found in game"))?;

            can_player_make_move(&game, connection_id)?;
            check_game_state(&game)?;
            make_move(&mut game, connection_id, &player_move)?;
            save_game(&dynamo_db_client, game_table, &game).await?;
            notify_other_player_about_game_update(
                &sdk_config,
                request_context,
                connection_id,
                &game,
            )
            .await?;

            tracing::info!(
        "PLAYER {username} MADE A MOVE (GAME ID: {game_id}): {player_move:?}. Game state: {game:?}"
    );

            Ok(ApiGatewayProxyResponse {
                status_code: 200, // Doesn't seem to be used by API Gateway
                body: Some(Body::from(serde_json::to_string(&game)?)),
                ..Default::default()
            })
        }
    }
}
