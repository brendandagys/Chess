use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use lambda_http::Body;
use lambda_runtime::Error;

pub fn offer_draw(game_id: &str) -> Result<ApiGatewayProxyResponse, Error> {
    Ok(ApiGatewayProxyResponse {
        status_code: 200, // Doesn't seem to be used by API Gateway
        body: Some(Body::from(serde_json::to_string("`offer_draw()`")?)),
        ..Default::default()
    })
}
