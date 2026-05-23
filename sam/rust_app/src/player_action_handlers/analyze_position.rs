use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use aws_sdk_lambda::primitives::Blob;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use chess::{
    helpers::{
        board::game_state_to_fen,
        game::{get_game, get_player_details_from_connection_id},
        opening_detection::GamePhase,
        pgn::build_pgn_movetext,
    },
    types::{api::ApiResponse, game::AnalysisType},
    utils::{api::build_response, api_gateway::post_to_connection},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiAnalysisResult {
    analysis_type: AnalysisType,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AgentResponse {
    // status_code: u16,
    // headers: std::collections::HashMap<String, String>,
    body: String,
}

#[derive(Debug, Deserialize)]
struct AgentBody {
    response: String,
}

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

    let payload = serde_json::json!({
        "fen": fen,
        "pgn_moves": pgn_moves,
        "opening_name": opening_name,
        "game_phase": game_phase,
        "goal": goal,
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
        "Invoking chess agent"
    );
    let invoke_output = lambda_client
        .invoke()
        .function_name(&function_name)
        .payload(Blob::new(serde_json::to_vec(&payload)?))
        .send()
        .await?;

    info!(
        status_code = ?invoke_output.status_code(),
        "Chess agent Lambda invoked successfully"
    );

    let response_payload = invoke_output
        .payload()
        .ok_or("Chess agent returned no payload")?;

    info!(
        payload_bytes = response_payload.as_ref().len(),
        "Retrieved response payload from Lambda"
    );

    if let Some(error) = invoke_output.function_error() {
        warn!(
            function_name,
            error, "Chess agent returned a function error"
        );
    }

    let agent_response: AgentResponse = serde_json::from_slice(response_payload.as_ref())?;
    info!("Agent response: {:?}", agent_response);

    info!(
        body_length = agent_response.body.len(),
        "Parsing response body JSON"
    );
    let agent_body: AgentBody = serde_json::from_str(&agent_response.body)?;

    info!("Agent body: {:?}", agent_body);

    info!(
        response_length = agent_body.response.len(),
        response = %agent_body.response,
        "Chess agent responded successfully"
    );

    post_to_connection(
        sdk_config,
        request_context,
        connection_id,
        &ApiResponse {
            status_code: 200,
            connection_id: Some(connection_id.to_string()),
            messages: vec![],
            data: Some(AiAnalysisResult {
                analysis_type,
                text: agent_body.response,
            }),
        },
    )
    .await?;

    build_response(
        StatusCode::OK,
        Some(connection_id.to_string()),
        None,
        None::<()>,
    )
}
