mod helpers;
mod types;
mod utils;

use helpers::game::{assign_player_to_remaining_slot, create_game, get_game, save_game};
use helpers::user::{create_user_game, get_user_game};

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_http::aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_http::LambdaEvent;
use lambda_runtime::{run, service_fn, Error};
use std::env;
use types::lambda_runtime::CustomResponse;
use types::pieces::Color;

async fn function_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    client: &Client,
) -> Result<CustomResponse, Error> {
    let request_context = event.payload.request_context;
    let query_params = event.payload.query_string_parameters;

    // Mandatory
    let username = query_params
        .first("username")
        .ok_or_else(|| Error::from("Username is required"))?;

    // Optional
    let game_id = query_params.first("game_id");
    let color_preference = query_params
        .first("color")
        .and_then(|color| color.parse::<Color>().ok());

    // Get the connection ID from the WebSocket context
    let connection_id = request_context
        .connection_id
        .ok_or_else(|| Error::from("Missing connection ID"))?;

    // Initialize DynamoDB client
    let game_table = env::var("GAME_TABLE").unwrap();
    let user_table = env::var("USER_TABLE").unwrap();

    let game = match game_id {
        Some(game_id) => match get_game(&client, &game_table, game_id).await {
            Ok(game) => assign_player_to_remaining_slot(game, username, &connection_id)?,
            Err(_) => create_game(Some(game_id), username, color_preference, &connection_id),
        },
        None => create_game(None, username, color_preference, &connection_id),
    };

    save_game(&client, &game_table, &game).await?;

    if let Err(_) = get_user_game(&client, &user_table, username, &game.game_id).await {
        create_user_game(&game.game_id, username);
    }

    Ok(CustomResponse {
        status_code: 200,
        body: format!("{username} joined game (ID: {}) successfully", game.game_id),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client: Client = Client::new(&aws_config::load_defaults(BehaviorVersion::latest()).await);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(
        |event: LambdaEvent<ApiGatewayWebsocketProxyRequest>| async {
            function_handler(event, &client).await
        },
    ))
    .await?;

    Ok(())
}
