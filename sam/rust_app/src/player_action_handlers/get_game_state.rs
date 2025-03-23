use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use lambda_http::{http::StatusCode, Body};
use lambda_runtime::Error;

use chess::{helpers::game::get_game, utils::api::build_response};

pub async fn get_game_state(
    dynamo_db_client: &Client,
    game_table: &str,
    game_id: &str,
) -> Result<ApiGatewayProxyResponse, Error> {
    match get_game(dynamo_db_client, game_table, game_id).await? {
        Some(game) => {
            tracing::info!("Retrieved game state (ID: {})", game_id);

            Ok(ApiGatewayProxyResponse {
                status_code: 200, // Doesn't seem to be used by API Gateway
                body: Some(Body::from(serde_json::to_string(&game)?)),
                ..Default::default()
            })
        }
        None => build_response(
            Some(StatusCode::BAD_REQUEST),
            Some(&format!("Game with ID `{game_id}` not found")),
            None::<()>,
        ),
    }
}
