mod helpers;
mod types;
mod utils;

use helpers::game::{get_game, save_game};
use helpers::user::{get_user_game_from_connection_id, save_user_record};
use utils::api_gateway::post_to_connection;

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
    let request_context = event.payload.request_context;

    let user_table = std::env::var("USER_TABLE").unwrap();
    let user_table_gsi = std::env::var("USER_TABLE_GSI").unwrap();
    let game_table = std::env::var("GAME_TABLE").unwrap();

    let connection_id = request_context
        .connection_id
        .as_ref()
        .ok_or_else(|| Error::from("Missing connection ID"))?;

    let mut user_game = get_user_game_from_connection_id(
        dynamo_db_client,
        &user_table,
        &user_table_gsi,
        &connection_id,
    )
    .await?;

    // Get the game ID from the user-game record sort key
    let game_id = user_game.sort_key.trim_start_matches("GAME-");
    let username = &user_game.username;

    // Disassociate this connection from the user's game and the game itself
    user_game.connection_id = Some("<disconnected>".to_string());
    save_user_record(dynamo_db_client, &user_table, &user_game).await?;

    // Fetch the game using the user-game record's game ID from the sort key
    let mut game = get_game(dynamo_db_client, &game_table, game_id).await?;

    // Remove the respective connection ID from the game record.
    // Notify the other player about the disconnect, if they are connected.
    match game.white_username == Some(username.clone()) {
        true => {
            game.white_connection_id = None;
            save_game(dynamo_db_client, &game_table, &game).await?;

            if let Some(black_connection_id) = &game.black_connection_id {
                post_to_connection(sdk_config, &request_context, &black_connection_id, &game)
                    .await?;
                tracing::info!("Notified black player of disconnection for game (ID: {game_id})",);
            }
        }
        false => {
            game.black_connection_id = None;
            save_game(dynamo_db_client, &game_table, &game).await?;

            if let Some(white_connection_id) = &game.white_connection_id {
                post_to_connection(sdk_config, &request_context, &white_connection_id, &game)
                    .await?;
                tracing::info!("Notified white player of disconnection for game (ID: {game_id})",);
            }
        }
    }

    tracing::info!("USER {username} DISCONNECTED FROM GAME (ID: {game_id})");

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
        body: Some((format!("{username} disconnected from game (ID: {})", game_id)).into()),
        ..Default::default()
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamo_db_client: Client = Client::new(&sdk_config);

    tracing_subscriber::fmt()
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
