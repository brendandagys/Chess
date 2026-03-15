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

fn promotion_char(piece: chess_engine::types::Piece) -> &'static str {
    match piece {
        chess_engine::types::Piece::Knight => "n",
        chess_engine::types::Piece::Bishop => "b",
        chess_engine::types::Piece::Rook => "r",
        chess_engine::types::Piece::Queen => "q",
        _ => "",
    }
}

/// Formats the engine's principal variation as a UCI move sequence (up to 5 moves).
fn format_pv(search_result: &SearchResult) -> String {
    let moves: Vec<String> = search_result
        .principal_variation
        .iter()
        .take(5)
        .map(|m| {
            let promo = m.promote.map(promotion_char).unwrap_or("");
            format!(
                "{}{}{}",
                square_to_algebraic(m.from),
                square_to_algebraic(m.to),
                promo
            )
        })
        .collect();

    if moves.is_empty() {
        String::new()
    } else {
        format!("Engine line: {}.", moves.join(" "))
    }
}

/// Describes the evaluation from White's absolute perspective.
fn eval_description(eval_cp: i32, side_to_move: Color) -> String {
    // Normalize to white's perspective
    let white_cp = match side_to_move {
        Color::White => eval_cp,
        Color::Black => -eval_cp,
    };

    let pawns = white_cp.abs() as f32 / 100.0;

    // MATE_THRESHOLD in chess_engine/src/constants.rs is 9000 (private)
    if white_cp.abs() > 9000 {
        let winner = if white_cp > 0 { "White" } else { "Black" };
        format!("{winner} has a forced checkmate")
    } else if white_cp.abs() > 500 {
        let leader = if white_cp > 0 { "White" } else { "Black" };
        format!("{leader} has a winning advantage ({pawns:.1} pawns)")
    } else if white_cp.abs() < 40 {
        "equal (0.0 pawns)".to_string()
    } else {
        let leader = if white_cp > 0 { "White" } else { "Black" };
        format!("{leader} +{pawns:.1} pawns")
    }
}

fn material_context(white_points: u16, black_points: u16) -> String {
    match white_points.cmp(&black_points) {
        std::cmp::Ordering::Greater => {
            format!("White leads in material +{}.", white_points - black_points)
        }
        std::cmp::Ordering::Less => {
            format!("Black leads in material +{}.", black_points - white_points)
        }
        std::cmp::Ordering::Equal => "Material is equal.".to_string(),
    }
}

/// Classifies a centipawn loss into a quality tier.
fn cpl_category(cpl: i32) -> &'static str {
    match cpl {
        i32::MIN..=20 => "best/good",
        21..=50 => "an inaccuracy",
        51..=100 => "a mistake",
        _ => "a blunder",
    }
}

pub fn build_analysis_prompt(
    analysis_type: &AnalysisType,
    fen: &str,
    current_turn: Color,
    search_result: &SearchResult,
    prev_eval: Option<(i32, Color)>,
    move_number: usize,
    in_check: Option<Color>,
    white_material: u16,
    black_material: u16,
) -> String {
    let turn_str = match current_turn {
        Color::White => "White",
        Color::Black => "Black",
    };

    let best_move_str = match (search_result.best_move_from, search_result.best_move_to) {
        (Some(from), Some(to)) => {
            let promo = search_result
                .best_move_promote
                .map(promotion_char)
                .unwrap_or("");
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
    let pv_str = format_pv(search_result);
    let mat_str = material_context(white_material, black_material);

    let check_note = match in_check {
        Some(Color::White) => " White is in check.",
        Some(Color::Black) => " Black is in check.",
        None => "",
    };

    let book_note = if search_result.from_book {
        " Still in opening theory."
    } else {
        ""
    };

    let pv_note = if pv_str.is_empty() {
        String::new()
    } else {
        format!(" {pv_str}")
    };

    // Shared context block used by all prompt types
    let context = format!(
        "Chess position — FEN: {fen}\n\
         Move {move_number}, {turn_str} to move.{check_note}\n\
         Eval: {eval_desc}. {mat_str}{book_note}\n\
         Engine best move: {best_move_str} (depth {depth}).{pv_note}\n",
        depth = search_result.depth,
    );

    match analysis_type {
        AnalysisType::MoveExplanation => format!(
            "{context}\n\
             In 2-3 sentences, explain {best_move_str}: name the specific threat, tactic, or \
             strategic plan it sets up, and describe what the engine line leads to. \
             If the move is a book move, explain the general opening principles \
             it follows and try to mention the name of the opening/variation. \
             Be concrete — no generic filler."
        ),

        AnalysisType::BlunderDetection => {
            let quality_context = if let Some((prev_cp, prev_turn)) = prev_eval {
                // Centipawn loss: how much the last mover lost vs. their best option.
                // prev_cp is from last-mover's perspective; search_result.evaluation is
                // from the opponent's (current_turn) perspective — so negate it.
                let cpl = (prev_cp + search_result.evaluation).max(0);
                let category = cpl_category(cpl);
                let prev_desc = eval_description(prev_cp, prev_turn);
                format!(
                    "Before the last move: {prev_desc}. \
                     Centipawn loss: {cpl} — classifies as {category}.\n"
                )
            } else {
                String::new()
            };

            format!(
                "{context}{quality_context}\n\
                 In 2 sentences, state precisely whether the last move was a blunder, \
                 mistake, inaccuracy, or good move (use the CPL tier above), then name \
                 the better alternative and the concrete reason it was superior. \
                 No hedging."
            )
        }

        AnalysisType::Coach => format!(
            "{context}\n\
             In 2 sentences, give {turn_str} one concrete, actionable coaching tip. \
             Identify the single most important plan or improvement right now — \
             name specific pieces, squares, or threats. Skip generic advice."
        ),

        AnalysisType::PostGame => format!(
            "{context}\n\
             The game has just ended. In 3 sentences: identify the single most critical \
             turning point by move, name the decisive mistake and the specific better move, \
             and give one concrete takeaway lesson for the losing side."
        ),
    }
}
