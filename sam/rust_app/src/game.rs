use aws_config::BehaviorVersion;
use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequest};
use aws_sdk_dynamodb::Client;
use chess::{types::api::GameRequest, utils::api::build_response};
use lambda_http::{http::StatusCode, LambdaEvent};
use lambda_runtime::{run, service_fn, Error};

mod player_action_handlers;

use chess::types::game::PlayerAction;
use player_action_handlers::{
    create_new_game::create_new_game, get_game_state::get_game_state, join_game::join_game,
    move_piece::move_piece, offer_draw::offer_draw,
};

async fn function_handler(
    event: LambdaEvent<ApiGatewayWebsocketProxyRequest>,
    sdk_config: &aws_config::SdkConfig,
    dynamo_db_client: &Client,
) -> Result<ApiGatewayProxyResponse, Error> {
    let game_table = std::env::var("GAME_TABLE").unwrap();
    let user_table = std::env::var("USER_TABLE").unwrap();

    let request_context = event.payload.request_context;

    let Some(connection_id) = request_context.connection_id.as_ref() else {
        return build_response(
            StatusCode::BAD_REQUEST,
            None,
            Some(vec!["Missing connection ID".into()]),
            None::<()>,
        );
    };

    let Some(request_body) = event.payload.body.as_ref() else {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.clone()),
            Some(vec!["Missing request body".into()]),
            None::<()>,
        );
    };

    let request_data = match serde_json::from_str::<GameRequest>(request_body) {
        Ok(data) => data,
        Err(e) => {
            return build_response(
                StatusCode::BAD_REQUEST,
                Some(connection_id.clone()),
                Some(vec![format!(
                    "Failed to parse request body into a valid player action: {e}"
                )
                .into()]),
                None::<()>,
            );
        }
    };

    match request_data.data {
        PlayerAction::CreateGame {
            username,
            game_id,
            board_setup,
            color_preference,
            seconds_per_player,
        } => {
            create_new_game(
                dynamo_db_client,
                &game_table,
                &user_table,
                connection_id,
                username.trim(),
                game_id.as_deref().map(|s| s.trim()),
                board_setup,
                color_preference,
                seconds_per_player,
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
                connection_id,
                username.trim(),
                game_id.trim(),
            )
            .await
        }
        PlayerAction::LeaveGame { game_id } => {
            player_action_handlers::leave_game::leave_game(
                sdk_config,
                &request_context,
                dynamo_db_client,
                connection_id,
                &game_table,
                &user_table,
                &game_id,
            )
            .await
        }
        PlayerAction::GetGameState { game_id } => {
            get_game_state(dynamo_db_client, connection_id, &game_table, game_id.trim()).await
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
                &user_table,
                connection_id,
                &game_id,
                player_move,
            )
            .await
        }
        PlayerAction::Heartbeat => build_response(
            StatusCode::OK,
            Some(connection_id.clone()),
            None,
            None::<()>,
        ),
        PlayerAction::LoseViaOutOfTime { game_id } => {
            player_action_handlers::lose_via_out_of_time::lose_via_out_of_time(
                sdk_config,
                &request_context,
                dynamo_db_client,
                connection_id,
                &game_table,
                &user_table,
                &game_id,
            )
            .await
        }
        PlayerAction::Resign { game_id } => {
            player_action_handlers::resign::resign(
                sdk_config,
                &request_context,
                dynamo_db_client,
                connection_id,
                &game_table,
                &user_table,
                &game_id,
            )
            .await
        }
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
