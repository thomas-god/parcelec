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
    forecast::ForecastLevel,
    game::{
        GameContext, GameId, GameMessage, GameState, GetPreviousScoresResult,
        delivery_period::DeliveryPeriodId,
        scores::{PlayerScore, RankTier},
    },
    market::{
        Direction, Market, MarketContext, MarketForecast, MarketState, OrderRepr,
        order_book::{OrderRequest as MarketOrderRequest, TradeLeg},
    },
    plants::{
        GetSnapshotError, PlantId, PowerPlantPublicRepr, Stack,
        actor::{ProgramPlant, StackContext, StackState},
    },
};

use super::{PlayerId, PlayerName, repository::ConnectionRepositoryMessage};

#[derive(Clone)]
pub struct PlayerConnection {
    pub id: String,
    pub player_id: PlayerId,
    pub tx: Sender<PlayerMessage>,
}

impl Debug for PlayerConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerConnection")
            .field("id", &self.player_id)
            .finish()
    }
}

#[derive(Deserialize, Debug)]
pub struct OrderRequest {
    pub direction: Direction,
    pub price: isize,
    pub volume: usize,
}

#[derive(Deserialize, Debug)]
enum WebSocketIncomingMessage {
    ConnectionReady,
    PlayerIsReady,
    OrderRequest(OrderRequest),
    DeleteOrder { order_id: String },
    ProgramPlant(ProgramPlant),
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PlayerResultView {
    pub player: PlayerName,
    pub rank: usize,
    pub score: isize,
    pub tier: Option<RankTier>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type")]
pub enum PlayerMessage {
    NewTrade(TradeLeg),
    OrderBookSnapshot {
        bids: Vec<OrderRepr>,
        offers: Vec<OrderRepr>,
    },
    TradeList {
        trades: Vec<TradeLeg>,
    },
    NewMarketForecast(MarketForecast),
    MarketForecasts {
        forecasts: Vec<MarketForecast>,
    },
    StackSnapshot {
        plants: HashMap<PlantId, PowerPlantPublicRepr>,
    },
    StackForecasts {
        forecasts: HashMap<PlantId, Option<ForecastLevel>>,
    },
    DeliveryPeriodResults {
        delivery_period: DeliveryPeriodId,
        score: PlayerScore,
    },
    GameResults {
        rankings: Vec<PlayerResultView>,
    },
    ReadinessStatus {
        readiness: HashMap<PlayerName, bool>,
    },
    YourName {
        name: PlayerName,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
struct PlayerScores {
    scores: HashMap<DeliveryPeriodId, PlayerScore>,
}

#[derive(Debug, Clone)]
pub struct PlayerConnectionContext<MS: Market, PS: Stack> {
    pub game_id: GameId,
    pub player_id: PlayerId,
    pub player_name: PlayerName,
    pub connections_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    pub game: GameContext,
    pub market: MarketContext<MS>,
    pub stack: StackContext<PS>,
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
    send_initial_stack_snapshot(&mut ws, &context).await?;
    send_initial_trades_and_obs(&mut ws, &context).await?;
    send_stack_forecasts(&mut ws, &context).await?;
    send_previous_scores(&mut ws, &context).await?;
    send_readiness_satus(&mut ws, &context).await?;

    let (sink, stream) = ws.split();
    let sink_handle = tokio::spawn(process_internal_messages(
        sink,
        rx,
        context.game.state_rx,
        context.market.state_rx,
        context.stack.state_rx,
    ));
    let stream_handle = tokio::spawn(process_ws_messages(
        stream,
        context.game.tx.clone(),
        context.market.service.clone(),
        context.stack.service.clone(),
        context.player_id.clone(),
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
    let (trades, obs, forecasts) = context
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
    ws.send(serde_json::to_string(&PlayerMessage::MarketForecasts { forecasts })?.into())
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

async fn send_initial_stack_snapshot<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    ws.send(
        serde_json::to_string(&PlayerMessage::StackSnapshot {
            plants: context.stack.service.get_snapshot().await?,
        })?
        .into(),
    )
    .await?;
    Ok(())
}

async fn send_stack_forecasts<MS: Market, PS: Stack>(
    ws: &mut WebSocket,
    context: &PlayerConnectionContext<MS, PS>,
) -> Result<(), PlayerConnectionError> {
    ws.send(
        serde_json::to_string(&PlayerMessage::StackForecasts {
            forecasts: context.stack.service.get_forecasts().await,
        })?
        .into(),
    )
    .await?;
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
        GetPreviousScoresResult::PlayerScores { scores } => {
            ws.send(serde_json::to_string(&PlayerScores { scores })?.into())
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
    stack: PS,
    player_id: PlayerId,
) {
    while let Some(Ok(Message::Text(msg))) = stream.next().await {
        match serde_json::from_str::<WebSocketIncomingMessage>(msg.as_str()) {
            Ok(WebSocketIncomingMessage::PlayerIsReady) => {
                let _ = game_tx
                    .send(GameMessage::PlayerIsReady(player_id.clone()))
                    .await;
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
                let _ = stack.program_setpoint(req.plant_id, req.setpoint).await;
            }
            Err(err) => tracing::error!("{err:?}"),
        }
    }
}

async fn process_internal_messages(
    mut sink: SplitSink<WebSocket, Message>,
    mut rx: Receiver<PlayerMessage>,
    mut game_state: watch::Receiver<GameState>,
    mut market_state: watch::Receiver<MarketState>,
    mut stack_state: watch::Receiver<StackState>,
) {
    // Send initial game, market and stack states before processing further messages
    let initial_game_state = serde_json::to_string(&game_state.borrow_and_update().clone());
    if send_msg(&mut sink, initial_game_state).await.is_err() {
        return;
    }
    let initial_market_state = serde_json::to_string(&market_state.borrow_and_update().clone());
    if send_msg(&mut sink, initial_market_state).await.is_err() {
        return;
    }
    let initial_stack_state = serde_json::to_string(&stack_state.borrow_and_update().clone());
    if send_msg(&mut sink, initial_stack_state).await.is_err() {
        return;
    }

    loop {
        let msg = select! {
            Some(msg) = rx.recv() => serde_json::to_string(&msg),
            Ok(()) = game_state.changed() => {
                serde_json::to_string(&game_state.borrow_and_update().clone())
            }
            Ok(()) = market_state.changed() => {
                serde_json::to_string(&market_state.borrow_and_update().clone())
            }
            Ok(()) = stack_state.changed() => {
                serde_json::to_string(&stack_state.borrow_and_update().clone())
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
