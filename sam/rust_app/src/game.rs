mod helpers;
mod player_action_handlers;
mod types;
mod utils;

use player_action_handlers::{
    create_new_game::create_new_game, get_game_state::get_game_state, join_game::join_game,
    move_piece::move_piece, offer_draw::offer_draw, resign::resign,
};
use types::game::{GameRequest, PlayerAction};

use aws_config::BehaviorVersion;
use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequest};
use aws_sdk_dynamodb::Client;
use lambda_http::LambdaEvent;
use lambda_runtime::{run, service_fn, Error};

async fn function_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    sdk_config: &aws_config::SdkConfig,
    dynamo_db_client: &Client,
) -> Result<ApiGatewayProxyResponse, Error> {
    let game_table = std::env::var("GAME_TABLE").unwrap();
    let user_table = std::env::var("USER_TABLE").unwrap(); // TODO: Update winner after game

    let request_context = event.payload.request_context;

    let connection_id = request_context
        .connection_id
        .as_ref()
        .ok_or_else(|| Error::from("Missing connection ID"))?;

    let request_body = event
        .payload
        .body
        .as_ref()
        .ok_or_else(|| Error::from("Missing request body"))?;

    let request_data: GameRequest = serde_json::from_str(request_body).map_err(|e| {
        Error::from(format!(
            "Failed to parse request body into a valid player action: {e}",
        ))
    })?;

    match request_data.data {
        PlayerAction::CreateGame {
            username,
            game_id,
            color_preference,
        } => {
            create_new_game(
                dynamo_db_client,
                &game_table,
                &user_table,
                &connection_id,
                &username,
                game_id.as_deref(),
                color_preference,
            )
            .await
        }
        PlayerAction::JoinGame { username, game_id } => {
            join_game(
                sdk_config,
                &request_context,
                dynamo_db_client,
                &game_table,
                &user_table,
                &connection_id,
                &username,
                &game_id,
            )
            .await
        }
        PlayerAction::GetGameState { game_id } => {
            return get_game_state(dynamo_db_client, &game_table, &game_id).await;
        }
        PlayerAction::MovePiece {
            game_id,
            player_move,
        } => {
            move_piece(
                sdk_config,
                &request_context,
                dynamo_db_client,
                &game_table,
                &connection_id,
                &game_id,
                player_move,
            )
            .await
        }
        PlayerAction::Resign { game_id } => resign(&game_id),
        PlayerAction::OfferDraw { game_id } => offer_draw(&game_id),
    }
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
