use std::{collections::HashMap, fmt::Debug};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        watch,
    },
};
use uuid::Uuid;

use crate::{
    game::{game_repository::GameId, GameContext, GameMessage, GameState},
    market::{
        order_book::{OrderRequest, TradeLeg},
        MarketContext, MarketMessage, MarketState, OrderRepr,
    },
    plants::{
        stack::{ProgramPlant, StackContext, StackMessage, StackState},
        PowerPlantPublicRepr,
    },
};

use super::repository::ConnectionRepositoryMessage;

#[derive(Clone)]
pub struct PlayerConnection {
    pub id: String,
    pub player_id: String,
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
enum WebSocketIncomingMessage {
    ConnectionReady,
    PlayerIsReady,
    OrderRequest(OrderRequest),
    DeleteOrder { order_id: String },
    ProgramPlant(ProgramPlant),
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
    StackSnapshot {
        plants: HashMap<String, PowerPlantPublicRepr>,
    },
}
pub struct PlayerConnectionContext {
    pub game_id: GameId,
    pub player_id: String,
    pub connections_repository: mpsc::Sender<ConnectionRepositoryMessage>,
    pub game: GameContext,
    pub market: MarketContext,
    pub stack: StackContext,
}

pub async fn start_player_connection(mut ws: WebSocket, context: PlayerConnectionContext) {
    let connection_id = Uuid::new_v4().to_string();
    let (tx, rx) = channel::<PlayerMessage>(16);

    let Some(Ok(Message::Text(msg))) = ws.recv().await else {
        println!("Haven't received Message::Text for connection readines, closing WS");
        let _ = ws.close().await;
        return;
    };
    match serde_json::from_str::<WebSocketIncomingMessage>(&msg) {
        Ok(WebSocketIncomingMessage::ConnectionReady) => {}
        _ => {
            println!("First message is not a ConnectionReady, closing WS");
            let _ = ws.close().await;
            return;
        }
    }

    register_connection(tx, connection_id, &context).await;

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
        context.market.tx.clone(),
        context.stack.tx.clone(),
        context.player_id.clone(),
    ));
    let _ = tokio::try_join!(sink_handle, stream_handle);
}

async fn register_connection(
    tx: Sender<PlayerMessage>,
    connection_id: String,
    context: &PlayerConnectionContext,
) {
    let _ = context
        .connections_repository
        .send(ConnectionRepositoryMessage::RegisterConnection(
            context.game_id.clone(),
            PlayerConnection {
                id: connection_id.clone(),
                player_id: context.player_id.clone(),
                tx: tx.clone(),
            },
        ))
        .await;
    let _ = context
        .market
        .tx
        .send(MarketMessage::NewPlayerConnection(
            context.player_id.clone(),
        ))
        .await;
    let _ = context
        .stack
        .tx
        .send(StackMessage::RegisterPlayerConnection(PlayerConnection {
            id: connection_id,
            player_id: context.player_id.clone(),
            tx,
        }))
        .await;
}

async fn process_ws_messages(
    mut stream: SplitStream<WebSocket>,
    game_tx: Sender<GameMessage>,
    market_tx: Sender<MarketMessage>,
    stack_tx: Sender<StackMessage>,
    player_id: String,
) {
    while let Some(Ok(Message::Text(msg))) = stream.next().await {
        match serde_json::from_str::<WebSocketIncomingMessage>(msg.as_str()) {
            Ok(WebSocketIncomingMessage::PlayerIsReady) => {
                let _ = game_tx
                    .send(GameMessage::PlayerIsReady(player_id.clone()))
                    .await;
            }
            Ok(WebSocketIncomingMessage::OrderRequest(mut request)) => {
                request.owner = player_id.clone();
                let _ = market_tx.send(MarketMessage::OrderRequest(request)).await;
            }
            Ok(WebSocketIncomingMessage::DeleteOrder { order_id }) => {
                let _ = market_tx
                    .send(MarketMessage::OrderDeletionRequest { order_id })
                    .await;
            }
            Ok(WebSocketIncomingMessage::ConnectionReady) => { /* Only for WS initialisation */ }
            Ok(WebSocketIncomingMessage::ProgramPlant(req)) => {
                let _ = stack_tx.send(StackMessage::ProgramSetpoint(req)).await;
            }
            Err(err) => println!("{err:?}"),
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
            println!("Unable to serialize message: {msg:?}, error: {err:?}");
            Ok(())
        }
    }
}
