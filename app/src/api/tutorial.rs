use std::{collections::HashMap, env, time::Duration as StdDuration};

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tokio::sync::oneshot;

use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

use crate::{
    game::{
        Game, GameId, GameMessage, GameName, RegisterPlayerResponse,
        delivery_period::DeliveryPeriodId,
        scores::{GameRankings, TierLimits},
    },
    market::{MarketActor, bots::start_bots},
    player::PlayerName,
    utils::program_cleanup,
};

use super::ApiState;

pub async fn create_tutorial_game(
    cookies: Cookies,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    let mut state = state.write().await;
    let game_id = GameId::default();
    let player_name = PlayerName::random();
    let game_name = GameName::from(format!("tutorial-{}", player_name));
    let cancellation_token = program_cleanup(StdDuration::from_secs(3600 * 24));
    let market_context = MarketActor::start(
        &game_id,
        state.player_connections_repository.clone(),
        cancellation_token.clone(),
    );
    let last_delivery_period = DeliveryPeriodId::from(4);
    let game_context = Game::start(
        &game_id,
        Some(game_name),
        state.player_connections_repository.clone(),
        market_context.clone(),
        Some(last_delivery_period),
        GameRankings {
            tier_limits: Some(TierLimits {
                gold: 80_000,
                silver: 50_000,
                bronze: 25_000,
            }),
        },
        None,
        cancellation_token,
    );

    state
        .market_services
        .insert(game_id.clone(), market_context.clone());
    state
        .game_services
        .insert(game_id.clone(), game_context.clone());

    // Start the bots
    let cloned_market_context = market_context.clone();
    tokio::spawn(async move {
        start_bots(cloned_market_context).await;
    });

    // Register a player for this game
    let (tx_back, rx) = oneshot::channel();
    let _ = game_context
        .tx
        .send(GameMessage::RegisterPlayer {
            name: player_name.clone(),
            tx_back,
        })
        .await;
    let Ok(RegisterPlayerResponse::Success {
        id: player_id,
        stack,
    }) = rx.await
    else {
        tracing::error!("Unable to register tutorial player");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    match state.stack_services.get_mut(&game_id) {
        Some(game_stacks) => {
            if game_stacks.get(&player_id).is_some() {
                tracing::error!(
                    "A stack already exist for player {player_id:?} in game {game_id:?}"
                );
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            game_stacks.insert(player_id.clone(), stack.clone());
        }
        None => {
            let game_stacks = HashMap::from([(player_id.clone(), stack)]);
            state.stack_services.insert(game_id.clone(), game_stacks);
        }
    }

    // Start the game
    let _ = game_context
        .tx
        .send(GameMessage::PlayerIsReady(player_id.clone()))
        .await;

    // Write cookies back
    let Ok(domain) = env::var("DOMAIN") else {
        tracing::error!("No DOMAIN environnement variable");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let player_id_cookie = Cookie::build(("player_id", player_id.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain.clone())
        .path("/")
        .build();
    cookies.add(player_id_cookie);
    let game_id_cookie = Cookie::build(("game_id", game_id.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain.clone())
        .path("/")
        .build();
    cookies.add(game_id_cookie);
    let name_cookie = Cookie::build(("player_name", player_name.to_string()))
        .max_age(Duration::days(1))
        .same_site(SameSite::Strict)
        .domain(domain)
        .path("/")
        .build();
    cookies.add(name_cookie);
    tracing::info!("Tutorial game created");
    StatusCode::CREATED
}
