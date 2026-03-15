use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use lambda_http::http::StatusCode;
use lambda_runtime::Error;
use serde::Serialize;

use chess::{
    helpers::{
        ai::{build_analysis_prompt, call_bedrock},
        board::game_state_to_fen,
        engine::get_engine_from_fen,
        game::{get_game, get_player_details_from_connection_id},
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

    let history = &game.game_state.history;
    let current_state = history.last().expect("Game history should not be empty");
    let current_fen = game_state_to_fen(current_state);
    let current_turn = current_state.current_turn;
    let move_number = history.len();

    // For blunder detection, evaluate the position before the last move
    let prev_eval = if matches!(analysis_type, AnalysisType::BlunderDetection) && move_number >= 2 {
        let prev_state = &history[move_number - 2];
        let prev_fen = game_state_to_fen(prev_state);
        let mut prev_engine =
            get_engine_from_fen(&prev_fen, 1000, game.engine_difficulty.map(|d| d.into()));
        let prev_result =
            prev_engine.think::<fn(u16, i32, &mut chess_engine::position::Position)>(None);
        Some((prev_result.evaluation, prev_state.current_turn))
    } else {
        None
    };

    let mut engine =
        get_engine_from_fen(&current_fen, 2000, game.engine_difficulty.map(|d| d.into()));
    let search_result = engine.think::<fn(u16, i32, &mut chess_engine::position::Position)>(None);

    let prompt = build_analysis_prompt(
        &analysis_type,
        &current_fen,
        current_turn,
        &search_result,
        prev_eval,
        move_number,
    );

    let text = call_bedrock(sdk_config, &prompt).await?;

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
                text,
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
