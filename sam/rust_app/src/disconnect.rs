use aws_config::BehaviorVersion;
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client;
use lambda_http::aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_http::http::StatusCode;
use lambda_http::LambdaEvent;
use lambda_runtime::{run, service_fn, Error};

use chess::helpers::game::{get_game, mark_user_as_disconnected_and_notify_other_player};
use chess::helpers::user::{get_user_games_from_connection_id, save_user_record};
use chess::utils::api::build_response;

async fn function_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    sdk_config: &aws_config::SdkConfig,
    dynamo_db_client: &Client,
) -> Result<ApiGatewayProxyResponse, Error> {
    let request_context = event.payload.request_context;

    let user_table = std::env::var("USER_TABLE").unwrap();
    let user_table_gsi = std::env::var("USER_TABLE_GSI").unwrap();
    let game_table = std::env::var("GAME_TABLE").unwrap();

    let Some(connection_id) = request_context.connection_id.as_ref() else {
        return build_response(
            Some(StatusCode::BAD_REQUEST),
            Some("Missing connection ID"),
            None::<()>,
        );
    };

    let mut user_games = get_user_games_from_connection_id(
        dynamo_db_client,
        &user_table,
        &user_table_gsi,
        &connection_id,
    )
    .await?;

    for user_game in user_games.iter_mut() {
        // Get the game ID from the user-game record sort key (e.g., "GAME-1234")
        let game_id = user_game.sort_key.trim_start_matches("GAME-");

        tracing::info!(
            "Found user game record (game ID: {game_id}) for connection ID {connection_id}"
        );

        let username = &user_game.username;

        // Disassociate this connection from the user-game record
        user_game.connection_id = Some("<disconnected>".to_string());
        save_user_record(dynamo_db_client, &user_table, &user_game).await?;

        // Fetch the game record with the user-game's game ID in the sort key
        let Some(mut game) = get_game(dynamo_db_client, &game_table, game_id).await? else {
            tracing::warn!("Game with ID {game_id} not found for associated user-game record");
            continue;
        };

        // Remove the respective connection ID from the game record.
        // Notify the other player if they are connected.
        mark_user_as_disconnected_and_notify_other_player(
            sdk_config,
            &request_context,
            dynamo_db_client,
            &game_table,
            &mut game,
            username,
        )
        .await?;

        tracing::info!("PLAYER {username} DISCONNECTED FROM GAME (ID: {game_id})");
    }

    Ok(ApiGatewayProxyResponse {
        status_code: 200,
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
