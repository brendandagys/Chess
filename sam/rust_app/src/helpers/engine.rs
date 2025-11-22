use crate::helpers::board::game_state_to_fen;
use crate::helpers::game::{make_move, notify_player_about_game_update};
use crate::helpers::opening_book::get_opening_book_path;
use crate::types::board::{File, Position, Rank};
use crate::types::dynamo_db::GameRecord;
use crate::types::game::{GameState, PlayerMove, SearchStatistics};
use crate::types::piece::Color;

use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequestContext;
use chess_engine::engine::{Engine, SearchResult};
use lambda_runtime::Error;

fn get_engine_color(game: &mut GameRecord) -> Color {
    match (&game.white_username, &game.black_username) {
        (None, Some(_)) => Color::White,
        (Some(_), None) => Color::Black,
        _ => unreachable!("Engine games should have exactly one player"),
    }
}

pub async fn use_engine(
    game: &mut GameRecord,
    sdk_config: &aws_config::SdkConfig,
    request_context: &ApiGatewayWebsocketProxyRequestContext,
    connection_id: &str,
) -> Result<(), Error> {
    if game.engine_difficulty.is_none() {
        return Ok(());
    }

    if !is_engine_turn(game) {
        return Ok(());
    }

    notify_player_about_game_update(sdk_config, request_context, connection_id, game, None, true)
        .await?;

    let mut engine = get_engine(game);
    let search_result = engine.think::<fn(u16, i32, &mut chess_engine::position::Position)>(None);

    if let Some(engine_move) = get_engine_move_from_search_result(&search_result) {
        make_engine_move_from_search_result(&mut engine, &search_result);
        make_move(&mut game.game_state, &engine_move);
    }

    handle_engine_think_time(&mut game.game_state, search_result.time_ms);

    game.game_state.current_state_mut().engine_result = Some(SearchStatistics {
        depth: search_result.depth,
        nodes: search_result.nodes,
        qnodes: search_result.qnodes,
        time_ms: search_result.time_ms,
        from_book: search_result.from_book,
    });

    game.game_state.current_state_mut().moves = engine.position.get_legal_moves();

    Ok(())
}

fn make_engine_move_from_search_result(engine: &mut Engine, search_result: &SearchResult) {
    let from_square = search_result
        .best_move_from
        .expect("Engine must provide a move");
    let to_square = search_result
        .best_move_to
        .expect("Engine must provide a move");

    engine
        .position
        .make_move(from_square, to_square, search_result.best_move_promote);
}

pub fn get_engine_move_from_search_result(search_result: &SearchResult) -> Option<PlayerMove> {
    let from_square = search_result.best_move_from?;
    let to_square = search_result.best_move_to?;

    Some(PlayerMove {
        from: Position {
            file: File((from_square.file() + 1) as usize),
            rank: Rank((from_square.rank() + 1) as usize),
        },
        to: Position {
            file: File((to_square.file() + 1) as usize),
            rank: Rank((to_square.rank() + 1) as usize),
        },
    })
}

pub fn handle_engine_think_time(game_state: &mut GameState, search_duration: u64) {
    let engine_color = game_state.current_state().current_turn;

    if let Some(game_time) = &mut game_state.game_time {
        let search_seconds = ((search_duration + 999) / 1000) as usize; // Round up
        let time_to_decrement = search_seconds.max(1);

        match engine_color {
            Color::White => {
                game_time.white_seconds_left = game_time
                    .white_seconds_left
                    .saturating_sub(time_to_decrement);
            }
            Color::Black => {
                game_time.black_seconds_left = game_time
                    .black_seconds_left
                    .saturating_sub(time_to_decrement);
            }
        }
    }
}

pub fn get_engine(game_record: &GameRecord) -> Engine {
    let opening_book_path = get_opening_book_path();
    let fen = game_state_to_fen(game_record.game_state.history.last().unwrap());

    let mut engine = Engine::new(
        None,
        None,
        None,
        None,
        Some(3000),
        None,
        None,
        opening_book_path,
        game_record.engine_difficulty.map(|d| d.into()),
    );

    engine.position = chess_engine::position::Position::from_fen(&fen)
        .unwrap_or_else(|_| panic!("Failed to load FEN: {fen}"));

    engine
}

fn is_engine_turn(game: &mut GameRecord) -> bool {
    let current_turn = game.game_state.current_state().current_turn;
    current_turn == get_engine_color(game)
}
