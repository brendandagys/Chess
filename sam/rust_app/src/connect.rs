mod helpers;
mod types;
mod utils;

use helpers::game::{
    assign_player_to_remaining_slot, create_game, get_game, notify_players_about_game_update,
    save_game,
};
use helpers::user::{create_user_game, get_user_game, save_user_record};
use types::pieces::Color;

use aws_config::BehaviorVersion;
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use lambda_http::aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_http::LambdaEvent;
use lambda_runtime::{run, service_fn, Error};

async fn function_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    sdk_config: &aws_config::SdkConfig,
    dynamo_db_client: &Client,
) -> Result<ApiGatewayProxyResponse, Error> {
    let game_table = std::env::var("GAME_TABLE").unwrap();
    let user_table = std::env::var("USER_TABLE").unwrap();

    let request_context = event.payload.request_context;
    let query_params = event.payload.query_string_parameters;

    // Mandatory query parameters
    let username = query_params
        .first("username")
        .ok_or_else(|| Error::from("Username is required"))?;

    // Optional query parameters
    let game_id = query_params.first("game_id");
    let color_preference = query_params
        .first("color")
        .and_then(|color| color.parse::<Color>().ok());

    // Get the connection ID from the WebSocket context
    let connection_id = request_context
        .connection_id
        .as_ref()
        .ok_or_else(|| Error::from("Missing connection ID"))?;

    let game = match game_id {
        Some(game_id) => match get_game(&dynamo_db_client, &game_table, game_id).await {
            Ok(found_game) => {
                tracing::info!(
                    "Found existing game for {username} (ID: {})",
                    found_game.game_id
                );
                assign_player_to_remaining_slot(found_game, username, &connection_id)?
            }
            Err(_) => {
                let new_game =
                    create_game(Some(game_id), username, color_preference, &connection_id);
                tracing::info!("Created new game for {username} (ID: {})", new_game.game_id);
                new_game
            }
        },
        None => {
            let new_game = create_game(None, username, color_preference, &connection_id);
            tracing::info!("Created new game for {username} (ID: {})", new_game.game_id);
            new_game
        }
    };

    save_game(&dynamo_db_client, &game_table, &game).await?;

    // Retrieve or create a new user-game record and assign user's connection ID to it
    match get_user_game(&dynamo_db_client, &user_table, username, &game.game_id).await {
        Ok(mut found_user_game) => {
            found_user_game.connection_id = Some(connection_id.clone());

            tracing::info!(
                "Found existing user-game record for {username} (ID: {})",
                found_user_game.sort_key
            );
            save_user_record(dynamo_db_client, &user_table, &found_user_game).await?;
        }
        Err(_) => {
            let new_user_game = create_user_game(&game.game_id, username, &connection_id);
            save_user_record(dynamo_db_client, &user_table, &new_user_game).await?;
            tracing::info!(
                "Created new user-game record for {username} (ID: {})",
                new_user_game.sort_key
            );
        }
    };

    notify_players_about_game_update(sdk_config, &request_context, connection_id, &game).await?;

    tracing::info!("PLAYER {username} CONNECTED TO GAME (ID: {})", game.game_id);

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        ..Default::default()
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_db_client = Client::new(&sdk_config);

    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .with_target(true) // Include the name of the module in every log line
        .with_current_span(false) // Remove duplicated "span" key in from logs
        .without_time() // CloudWatch will add the ingestion time
        .init();

    run(service_fn(
        |event: LambdaEvent<ApiGatewayWebsocketProxyRequest>| async {
            function_handler(event, &sdk_config, &dynamo_db_client).await
        },
    ))
    .await?;

    Ok(())
}
