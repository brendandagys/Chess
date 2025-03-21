mod helpers;
mod types;
mod utils;

use aws_config::BehaviorVersion;
use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequest};
use aws_sdk_dynamodb::Client;
use helpers::game::{
    can_player_make_move, check_game_state, get_game, make_move,
    notify_other_player_about_game_update, save_game,
};
use lambda_http::LambdaEvent;
use lambda_runtime::{run, service_fn, Error};
use types::game::GameActionPayload;

async fn function_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    sdk_config: &aws_config::SdkConfig,
    dynamo_db_client: &Client,
) -> Result<ApiGatewayProxyResponse, Error> {
    let request_context = event.payload.request_context;

    let game_table = std::env::var("GAME_TABLE").unwrap();
    let user_table = std::env::var("USER_TABLE").unwrap(); // TODO: Update winner after game

    // Get the connection ID from the WebSocket context
    let connection_id = request_context
        .connection_id
        .as_ref()
        .ok_or_else(|| Error::from("Missing connection ID"))?;

    let request_body = event
        .payload
        .body
        .as_ref()
        .ok_or_else(|| Error::from("Missing request body"))?;

    let payload: GameActionPayload = serde_json::from_str(request_body).map_err(|e| {
        Error::from(format!(
            "Failed to parse request body into a valid game action payload: {e}",
        ))
    })?;

    let username = payload.username;
    let game_id = payload.game_id;
    let player_move = payload.player_move;

    // Convert the player move string (from square, to square) to a new struct that represents a from and to for a chess move. Check that from square and to squares are valid notation.

    let mut game = get_game(&dynamo_db_client, &game_table, &game_id).await?;

    can_player_make_move(&game, &username, &connection_id)?;
    check_game_state(&game)?;
    make_move(&mut game, &username, &player_move)?;

    save_game(&dynamo_db_client, &game_table, &game).await?;

    notify_other_player_about_game_update(&sdk_config, &request_context, &game, &username).await?;

    tracing::info!("Player {username} made a move in game {game_id}: {player_move}");

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some(serde_json::to_string(&game)?.into()),
        ..Default::default()
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_db_client: Client = Client::new(&sdk_config);

    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        // Include the name of the module in every log line
        .with_target(true)
        // CloudWatch will add the ingestion time
        .without_time()
        .init();

    run(service_fn(
        |event: LambdaEvent<ApiGatewayWebsocketProxyRequest>| async {
            function_handler(event, &sdk_config, &dynamo_db_client).await
        },
    ))
    .await?;

    Ok(())
}
