use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::Client as BedrockClient;
use chess_engine::engine::SearchResult;
use chess_engine::types::Square;
use lambda_runtime::Error;

use crate::types::game::AnalysisType;
use crate::types::piece::Color;

pub async fn call_bedrock(
    sdk_config: &aws_config::SdkConfig,
    prompt: &str,
) -> Result<String, Error> {
    let client = BedrockClient::new(sdk_config);

    let model_id = std::env::var("BEDROCK_MODEL_ID")
        .unwrap_or_else(|_| "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string());

    let body = serde_json::json!({
        "anthropic_version": "bedrock-2023-05-31",
        "max_tokens": 300,
        "messages": [{"role": "user", "content": prompt}]
    });

    let response = client
        .invoke_model()
        .model_id(model_id)
        .content_type("application/json")
        .accept("application/json")
        .body(Blob::new(serde_json::to_vec(&body)?))
        .send()
        .await?;

    let response_json: serde_json::Value = serde_json::from_slice(response.body().as_ref())?;

    Ok(response_json["content"][0]["text"]
        .as_str()
        .unwrap_or("Unable to generate analysis.")
        .to_string())
}

fn square_to_algebraic(sq: Square) -> String {
    let file = (b'a' + sq.file()) as char;
    let rank = sq.rank() + 1;
    format!("{file}{rank}")
}

fn eval_description(eval_cp: i32, side_to_move: Color) -> String {
    // Normalize to white's perspective
    let white_cp = match side_to_move {
        Color::White => eval_cp,
        Color::Black => -eval_cp,
    };

    let pawns = white_cp.abs() as f32 / 100.0;

    // MATE_THRESHOLD in chess_engine/src/constants.rs is 9000 (private)
    if white_cp.abs() > 9000 {
        let mover = if white_cp > 0 { "White" } else { "Black" };
        format!("{mover} has a forced checkmate")
    } else if white_cp.abs() > 500 {
        let mover = if white_cp > 0 { "White" } else { "Black" };
        format!("{mover} has a winning advantage")
    } else if white_cp.abs() < 40 {
        "the position is approximately equal".to_string()
    } else {
        let leader = if white_cp > 0 { "White" } else { "Black" };
        format!("{leader} is ahead by {pawns:.1} pawns")
    }
}

pub fn build_analysis_prompt(
    analysis_type: &AnalysisType,
    fen: &str,
    current_turn: Color,
    search_result: &SearchResult,
    prev_eval: Option<(i32, Color)>,
    move_number: usize,
) -> String {
    let turn_str = match current_turn {
        Color::White => "White",
        Color::Black => "Black",
    };

    let best_move_str = match (search_result.best_move_from, search_result.best_move_to) {
        (Some(from), Some(to)) => {
            let promo = search_result
                .best_move_promote
                .map(|p| format!("={p:?}"))
                .unwrap_or_default();
            format!(
                "{}{}{}",
                square_to_algebraic(from),
                square_to_algebraic(to),
                promo
            )
        }
        _ => "none".to_string(),
    };

    let eval_desc = eval_description(search_result.evaluation, current_turn);

    let context = format!(
        "Chess position — FEN: {fen}\n\
         Move {move_number}. It is {turn_str}'s turn.\n\
         Engine evaluation: {eval_desc}.\n\
         Engine best move: {best_move_str} (depth {}).",
        search_result.depth
    );

    match analysis_type {
        AnalysisType::MoveExplanation => format!(
            "{context}\n\n\
             In 2-3 sentences, explain the key features of this position and why \
             the engine recommends {best_move_str}."
        ),

        AnalysisType::BlunderDetection => {
            let prev_context = if let Some((prev_cp, prev_turn)) = prev_eval {
                let prev_desc = eval_description(prev_cp, prev_turn);
                format!("\nBefore the last move: {prev_desc}.")
            } else {
                String::new()
            };

            format!(
                "{context}{prev_context}\n\n\
                 Was the last move a blunder, mistake, or inaccuracy? \
                 In 2-3 sentences, assess the quality of the last move and explain \
                 any better alternatives."
            )
        }

        AnalysisType::Coach => format!(
            "{context}\n\n\
             In 2-3 sentences, give concrete strategic coaching advice for {turn_str}. \
             Focus on the single most important plan or improvement."
        ),

        AnalysisType::PostGame => format!(
            "{context}\n\n\
             The game has just ended. In 3-4 sentences, provide post-game analysis: \
             describe the key turning point, highlight the decisive mistake, and \
             explain what the losing side could have done differently."
        ),
    }
}
