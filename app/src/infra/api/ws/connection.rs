use std::{collections::HashMap, fmt::Debug};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        mpsc::{self, Receiver, Sender, channel},
        oneshot, watch,
    },
};
use uuid::Uuid;

use crate::{
    game::{
        GameContext, GameId, GameMessage, GameState, GetPreviousScoresResult,
        delivery_period::DeliveryPeriodId,
        infra::stack_config::GameStackPerPlayerPlayerConfig,
        scores::{PlayerDetailedScore, PlayerScore},
    },
    infra::api::state::ApiState,
    market::{Direction, Market, MarketContext, order_book::OrderRequest as MarketOrderRequest},
    plants::{
        GetSnapshotError, Stack,
        infra::{ProgramPlant, StackContext},
    },
    player::{
        PlayerMessage,
        infra::{ConnectionRepositoryMessage, PlayerConnection},
    },
    utils::units::{Energy, EnergyCost, Power},
};

use super::{PlayerId, PlayerName};

#[derive(Deserialize, Debug)]
pub struct OrderRequest {
    pub direction: Direction,
    pub price: EnergyCost,
    pub volume: Energy,
}

#[derive(Deserialize, Debug)]
pub struct PlayerStackConfigRequest {
    pub gas_capacity: Power,
    pub nuclear_capcity: Power,
    pub battery_capacity: Energy,
    pub renewable_capacity: Power,
}

impl From<PlayerStackConfigRequest> for GameStackPerPlayerPlayerConfig {
    fn from(value: PlayerStackConfigRequest) -> Self {
        Self {
            gas_capacity: value.gas_capacity,
            nuclear_capacity: value.nuclear_capcity,
            battery_capacity: value.battery_capacity,
            renewable_capacity: value.renewable_capacity,
        }
    }
}

#[derive(Deserialize, Debug)]
enum WebSocketIncomingMessage {
    ConnectionReady,
    PlayerIsReady,
    RegisterPlayerStackConfig(PlayerStackConfigRequest),
    OrderRequest(OrderRequest),
    DeleteOrder { order_id: String },
    ProgramPlant(ProgramPlant),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
struct PlayerScores {
    scores: HashMap<DeliveryPeriodId, PlayerScore>,
    detailed_scores: HashMap<DeliveryPeriodId, PlayerDetailedScore>,
}

#[derive(Debug, Clone)]
pub struct PlayerConnectionContext<MS: Market, PS: Stack> {
    pub game_id: GameId,
    pub player_id: PlayerId,
    pub player_name: PlayerName,
    pub connections_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    pub game: GameContext,
    pub market: MarketContext<MS>,
    pub stack: Option<StackContext<PS>>,
}

#[derive(thiserror::Error, Debug)]
pub enum PlayerConnectionError {
    #[error("Haven't received Message::Text for connection readines")]
    ClientNotReady,
    #[error("First message is not a ConnectionReady")]
    FirstMessageReceivedNotConnectionReady,
    #[error(transparent)]
    ConnectionError(#[from] axum::Error),
    #[error(transparent)]
    SerializaionError(#[from] serde_json::Error),
    #[error("No snapshot")]
    NoSnapshotError(#[from] GetSnapshotError),
    #[error("Internal connection error")]
    InternalConnectionError,
}

pub async fn start_player_connection<MS: Market, PS: Stack>(
    mut ws: WebSocket,
    context: PlayerConnectionContext<MS, PS>,
    state: ApiState,
) -> Result<(), PlayerConnectionError> {
    let connection_id = Uuid::new_v4().to_string();
    let (tx, rx) = channel::<PlayerMessage>(16);

    let Some(Ok(Message::Text(msg))) = ws.recv().await else {
        tracing::error!("Haven't received Message::Text for connection readines, closing WS");
        let _ = ws.close().await;
        return Err(PlayerConnectionError::ClientNotReady);
    };
    match serde_json::from_str::<WebSocketIncomingMessage>(&msg) {
        Ok(WebSocketIncomingMessage::ConnectionReady) => {}
        _ => {
            tracing::error!("First message is not a ConnectionReady, closing WS");
            let _ = ws.close().await;
            return Err(PlayerConnectionError::FirstMessageReceivedNotConnectionReady);
        }
    }

    register_connection(tx, &connection_id, &context).await;

    send_player_name(&mut ws, &context).await?;
    send_game_duration(&mut ws, &context).await?;
    send_game_stack_config(&mut ws, &context).await?;
    send_initial_stack_snapshot(&mut ws, &context).await?;
    send_initial_trades_and_obs(&mut ws, &context).await?;
    send_stack_forecasts(&mut ws, &context).await?;
    send_stack_history(&mut ws, &context).await?;
    send_previous_scores(&mut ws, &context).await?;
    send_readiness_satus(&mut ws, &context).await?;

    let (sink, stream) = ws.split();
    let sink_handle = tokio::spawn(process_internal_messages(sink, rx, context.game.state_rx));
    let stream_handle = tokio::spawn(process_ws_messages(
        stream,
        context.game.tx.clone(),
        context.market.service.clone(),
        context.stack.map(|s| s.service.clone()),
        context.game_id.clone(),
        context.player_id.clone(),
        state,
    ));
    tokio::select! {
        _ = sink_handle => {},
        _ = stream_handle => {}
    }
    tracing::info!("Connection {connection_id:?} finised.");
    Ok(())
}

async fn send_initial_trades_and_obs<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    let (trades, obs) = context
        .market
        .service
        .get_market_snapshot(context.player_id.clone())
        .await;
    ws.send(serde_json::to_string(&PlayerMessage::TradeList { trades })?.into())
        .await?;
    ws.send(
        serde_json::to_string(&PlayerMessage::OrderBookSnapshot {
            bids: obs.bids,
            offers: obs.offers,
        })?
        .into(),
    )
    .await?;

    Ok(())
}

async fn send_player_name<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    ws.send(
        serde_json::to_string(&PlayerMessage::YourName {
            name: context.player_name.clone(),
        })?
        .into(),
    )
    .await?;
    Ok(())
}

async fn send_game_duration<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    ws.send(
        serde_json::to_string(&PlayerMessage::GameDuration {
            last_period: context.game.last_delivery_period,
        })?
        .into(),
    )
    .await?;
    Ok(())
}

async fn send_game_stack_config<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    ws.send(
        serde_json::to_string(&PlayerMessage::StackConfig {
            config: (&context.game.stack).into(),
        })?
        .into(),
    )
    .await?;
    Ok(())
}

async fn send_initial_stack_snapshot<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    let plants = match &context.stack {
        Some(stack) => Some(stack.service.get_snapshot().await?),
        None => None,
    };
    ws.send(serde_json::to_string(&PlayerMessage::StackSnapshot { plants })?.into())
        .await?;
    Ok(())
}

async fn send_stack_forecasts<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    if let Some(stack) = &context.stack {
        ws.send(
            serde_json::to_string(&PlayerMessage::StackForecasts {
                forecasts: stack.service.get_forecasts().await,
            })?
            .into(),
        )
        .await?;
    }
    Ok(())
}

async fn send_stack_history<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    if let Some(stack) = &context.stack {
        ws.send(
            serde_json::to_string(&PlayerMessage::StackHistory {
                history: stack.service.get_history().await,
            })?
            .into(),
        )
        .await?;
    }
    Ok(())
}

async fn send_previous_scores<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    let (tx_back, rx) = oneshot::channel();
    context
        .game
        .tx
        .send(GameMessage::GetScores {
            player_id: context.player_id.clone(),
            tx_back,
        })
        .await
        .map_err(|_| PlayerConnectionError::InternalConnectionError)?;
    let scores = rx
        .await
        .map_err(|_| PlayerConnectionError::InternalConnectionError)?;
    match scores {
        GetPreviousScoresResult::PlayerScores {
            scores,
            detailed_scores,
        } => {
            ws.send(
                serde_json::to_string(&PlayerScores {
                    scores,
                    detailed_scores,
                })?
                .into(),
            )
            .await?;
        }
        GetPreviousScoresResult::PlayersRanking { scores } => {
            ws.send(
                serde_json::to_string(&PlayerMessage::GameResults { rankings: scores })?.into(),
            )
            .await?;
        }
    }
    Ok(())
}
async fn send_readiness_satus<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    let (tx_back, rx) = oneshot::channel();
    context
        .game
        .tx
        .send(GameMessage::GetReadiness { tx_back })
        .await
        .map_err(|_| PlayerConnectionError::InternalConnectionError)?;
    let readiness = rx
        .await
        .map_err(|_| PlayerConnectionError::InternalConnectionError)?;
    ws.send(serde_json::to_string(&PlayerMessage::ReadinessStatus { readiness })?.into())
        .await?;
    Ok(())
}

async fn register_connection<MS: Market, PS: Stack>(
    tx: Sender<PlayerMessage>,
    connection_id: &str,
    context: &PlayerConnectionContext<MS, PS>,
) {
    let _ = context
        .connections_repository
        .send(ConnectionRepositoryMessage::RegisterConnection(
            context.game_id.clone(),
            PlayerConnection {
                id: connection_id.to_owned(),
                player_id: context.player_id.clone(),
                tx: tx.clone(),
            },
        ))
        .await;
}

async fn process_ws_messages<MS: Market, PS: Stack>(
    mut stream: SplitStream<WebSocket>,
    game_tx: Sender<GameMessage>,
    market: MS,
    stack: Option<PS>,
    game_id: GameId,
    player_id: PlayerId,
    state: ApiState,
) {
    while let Some(Ok(Message::Text(msg))) = stream.next().await {
        match serde_json::from_str::<WebSocketIncomingMessage>(msg.as_str()) {
            Ok(WebSocketIncomingMessage::PlayerIsReady) => {
                let _ = game_tx
                    .send(GameMessage::PlayerIsReady(player_id.clone()))
                    .await;
            }
            Ok(WebSocketIncomingMessage::RegisterPlayerStackConfig(request)) => {
                let (tx, rx) = oneshot::channel();
                let _ = game_tx
                    .send(GameMessage::RegisterPlayerStackConfig {
                        player: player_id.clone(),
                        config: request.into(),
                        tx_back: tx,
                    })
                    .await;
                if let Ok(Ok(stack)) = rx.await {
                    let mut state = state.write().await;

                    match state.stack_services.get_mut(&game_id) {
                        Some(stacks) => {
                            let _ = stacks.insert(player_id.clone(), stack);
                        }
                        None => {
                            let _ = state.stack_services.insert(
                                game_id.clone(),
                                HashMap::from([(player_id.clone(), stack)]),
                            );
                        }
                    };
                }
            }
            Ok(WebSocketIncomingMessage::OrderRequest(request)) => {
                let order_request = MarketOrderRequest {
                    direction: request.direction,
                    price: request.price,
                    volume: request.volume,
                    owner: player_id.clone(),
                };
                let _ = market.new_order(order_request).await;
            }
            Ok(WebSocketIncomingMessage::DeleteOrder { order_id }) => {
                let _ = market.delete_order(order_id).await;
            }
            Ok(WebSocketIncomingMessage::ConnectionReady) => { /* Only for WS initialisation */ }
            Ok(WebSocketIncomingMessage::ProgramPlant(req)) => {
                if let Some(ref stack) = stack {
                    let _ = stack.program_setpoint(req.plant_id, req.setpoint).await;
                }
            }
            Err(err) => tracing::error!("{err:?}"),
        }
    }
}

async fn process_internal_messages(
    mut sink: SplitSink<WebSocket, Message>,
    mut rx: Receiver<PlayerMessage>,
    mut game_state: watch::Receiver<GameState>,
) {
    // Send initial game state before processing further messages
    let initial_game_state = serde_json::to_string(&game_state.borrow_and_update().clone());
    if send_msg(&mut sink, initial_game_state).await.is_err() {
        return;
    }

    loop {
        let msg = select! {
            Some(msg) = rx.recv() => serde_json::to_string(&msg),
            Ok(()) = game_state.changed() => {
                serde_json::to_string(&game_state.borrow_and_update().clone())
            }
        };
        if send_msg(&mut sink, msg).await.is_err() {
            return;
        }
    }
}

async fn send_msg(
    sink: &mut SplitSink<WebSocket, Message>,
    msg: Result<String, serde_json::Error>,
) -> Result<(), ()> {
    match &msg {
        Ok(msg) => {
            if sink.send(Message::text(msg)).await.is_err() {
                return Err(());
            }
            Ok(())
        }
        Err(err) => {
            tracing::error!("Unable to serialize message: {msg:?}, error: {err:?}");
            Ok(())
        }
    }
}
