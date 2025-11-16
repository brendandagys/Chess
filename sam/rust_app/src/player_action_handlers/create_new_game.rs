use aws_lambda_events::apigw::{ApiGatewayProxyResponse, ApiGatewayWebsocketProxyRequestContext};
use aws_sdk_dynamodb::Client;
use chess::types::board::BoardSetup;
use chess::types::game::{ColorPreference, EngineDifficulty};
use lambda_http::http::StatusCode;
use lambda_runtime::Error;

use chess::helpers::game::{
    check_if_both_players_just_joined, create_game, get_engine_move_if_turn, get_game, make_move,
    save_game,
};
use chess::helpers::user::{create_user_game, save_user_record};
use chess::utils::api::build_response;

pub async fn create_new_game(
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    dynamo_db_client: &Client,
    game_table: &str,
    user_table: &str,
    connection_id: &str,
    username: &str,
    game_id: Option<&str>,
    board_setup: Option<BoardSetup>,
    color_preference: ColorPreference,
    engine_difficulty: Option<EngineDifficulty>,
    seconds_per_player: Option<usize>,
) -> Result<ApiGatewayProxyResponse, Error> {
    if username.trim().is_empty() {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec!["Must provide a username".into()]),
            None::<()>,
        );
    }

    // Validate that engine difficulty is only set for standard 8x8 boards
    if engine_difficulty.is_some() && !matches!(board_setup, Some(BoardSetup::Standard) | None) {
        return build_response(
            StatusCode::BAD_REQUEST,
            Some(connection_id.to_string()),
            Some(vec![
                "Engine difficulty can only be set for standard 8x8 boards".into(),
            ]),
            None::<()>,
        );
    }

    let mut new_game = match game_id {
        Some(game_id) => {
            if (get_game(dynamo_db_client, game_table, game_id).await?).is_some() {
                return build_response(
                    StatusCode::BAD_REQUEST,
                    Some(connection_id.to_string()),
                    Some(vec![format!(
                        "Game with ID `{game_id}` already exists. Please join the game instead."
                    )
                    .into()]),
                    None::<()>,
                );
            }

            create_game(
                Some(game_id),
                username,
                board_setup,
                color_preference,
                engine_difficulty,
                seconds_per_player,
                connection_id,
            )
        }
        None => create_game(
            None,
            username,
            board_setup,
            color_preference,
            engine_difficulty,
            seconds_per_player,
            connection_id,
        ),
    };

    if new_game.engine_difficulty.is_some() {
        check_if_both_players_just_joined(&mut new_game);

        if let Some(engine_move) =
            get_engine_move_if_turn(sdk_config, request_context, &mut new_game, connection_id)
                .await?
        {
            if let Err(e) = make_move(&mut new_game.game_state, &engine_move) {
                return build_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some(connection_id.to_string()),
                    Some(vec![e.into()]),
                    Some(new_game),
                );
            }
        }
    }

    save_game(dynamo_db_client, game_table, &new_game).await?;

    tracing::info!(
        "Created new game record (ID: {}) for user ({username})",
        new_game.game_id
    );

    let new_user_game = create_user_game(&new_game.game_id, username, connection_id);
    save_user_record(dynamo_db_client, user_table, &new_user_game).await?;

    tracing::info!(
        "Created new user-game record for {username} (ID: {})",
        new_user_game.sort_key
    );

    tracing::info!(
        "PLAYER {username} CREATED NEW GAME (ID: {})",
        new_game.game_id
    );

    build_response(
        StatusCode::OK,
        Some(connection_id.to_string()),
        None,
        Some(new_game),
    )
}
