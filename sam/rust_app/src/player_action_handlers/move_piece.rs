use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::{http::StatusCode, Body};
use lambda_runtime::Error;

use chess::{
    helpers::game::{
        can_player_make_move, get_game, get_player_details_from_connection_id, make_move,
        notify_other_player_about_game_update, save_game, PlayerDetails,
    },
    types::game::PlayerMove,
    utils::api::build_response,
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
        None => build_response(
            Some(StatusCode::BAD_REQUEST),
            Some(&format!("Game (ID: {game_id}) not found")),
            None::<()>,
        ),
        Some(mut game) => {
            let Some(PlayerDetails { color, username }) =
                get_player_details_from_connection_id(&game, connection_id)
            else {
                return build_response(
                    Some(StatusCode::BAD_REQUEST),
                    Some(&format!("You must be a player in the game to make a move")),
                    None::<()>,
                );
            };

            if let Err(e) = can_player_make_move(&game, &color) {
                return build_response(Some(StatusCode::BAD_REQUEST), Some(e), None::<()>);
            }

            if let Err(e) = make_move(&mut game, connection_id, &player_move) {
                return build_response(Some(StatusCode::BAD_REQUEST), Some(e), None::<()>);
            }

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
