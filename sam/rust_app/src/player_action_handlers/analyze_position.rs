use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::InvocationType;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;
use tracing::info;

use chess::{
    helpers::{
        board::game_state_to_fen,
        game::{get_game, get_player_details_from_connection_id},
        opening_detection::GamePhase,
        pgn::build_pgn_movetext,
    },
    types::game::AnalysisType,
    utils::api::build_response,
};

pub async fn analyze_position(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    connection_id: &str,
    game_id: &str,
    analysis_type: AnalysisType,
) -> Result<ApiGatewayProxyResponse, Error> {
    let Some(game) = get_game(dynamo_db_client, game_table, game_id).await? else {
        return build_response(
            StatusCode::NOT_FOUND,
            Some(connection_id.to_string()),
            Some(vec![format!("Game (ID: {game_id}) not found").into()]),
            None::<()>,
        );
    };

    if get_player_details_from_connection_id(&game, connection_id).is_none() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec!["You are not a player in this game".into()]),
            None::<()>,
        );
    }

    let current_state = game
        .game_state
        .history
        .last()
        .expect("Game history should not be empty");

    if !current_state.board.is_standard_board() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec![
                "Position analysis is only supported for standard 8x8 boards".into(),
            ]),
            None::<()>,
        );
    }

    info!(
        game_id,
        analysis_type = ?analysis_type,
        connection_id,
        "Starting position analysis"
    );

    let fen = game_state_to_fen(current_state);
    let pgn_moves = build_pgn_movetext(&game.game_state);

    let opening_name = game
        .game_state
        .opening
        .as_ref()
        .map(|o| o.name.as_str())
        .unwrap_or("");

    let game_phase = game
        .game_state
        .opening
        .as_ref()
        .map(|o| match o.phase {
            GamePhase::Opening => "Opening",
            GamePhase::EarlyMiddlegame => "Early Middlegame",
            GamePhase::Middlegame => "Middlegame",
            GamePhase::EarlyEndgame => "Early Endgame",
            GamePhase::Endgame => "Endgame",
        })
        .unwrap_or("Middlegame");

    let goal = match analysis_type {
        AnalysisType::Coach => "Coaching / teaching",
        AnalysisType::Analysis => "Deep analysis",
    };

    let callback_url = format!(
        "https://{}/{}",
        request_context.domain_name.as_deref().unwrap_or(""),
        request_context.stage.as_deref().unwrap_or(""),
    );

    let payload = serde_json::json!({
        "fen": fen,
        "pgn_moves": pgn_moves,
        "opening_name": opening_name,
        "game_phase": game_phase,
        "goal": goal,
        "connection_id": connection_id,
        "callback_url": callback_url,
        "analysis_type": analysis_type,
    });

    let function_name =
        std::env::var("CHESS_AGENT_FUNCTION_NAME").expect("CHESS_AGENT_FUNCTION_NAME must be set");

    let lambda_client = aws_sdk_lambda::Client::new(sdk_config);

    info!(
        %fen,
        %pgn_moves,
        opening_name,
        game_phase,
        goal,
        %callback_url,
        "Invoking chess agent asynchronously"
    );

    lambda_client
        .invoke()
        .function_name(&function_name)
        .invocation_type(InvocationType::Event)
        .payload(Blob::new(serde_json::to_vec(&payload)?))
        .send()
        .await?;

    info!("Chess agent Lambda invoked asynchronously (fire-and-forget)");

    build_response(
        StatusCode::OK,
        Some(connection_id.to_string()),
        None,
        None::<()>,
    )
}
