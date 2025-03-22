use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use lambda_http::Body;
use lambda_runtime::Error;

use crate::helpers::game::get_game;

pub async fn get_game_state(
    dynamo_db_client: &Client,
    game_table: &str,
    game_id: &str,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        None => {
            return Err(Error::from(format!("Game with ID {game_id} not found",)));
        }
        Some(game) => {
            tracing::info!("Retrieved game state (ID: {})", game_id);

            Ok(ApiGatewayProxyResponse {
                status_code: 200, // Doesn't seem to be used by API Gateway
                body: Some(Body::from(serde_json::to_string(&game)?)),
                ..Default::default()
            })
        }
    }
}
