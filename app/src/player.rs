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
    task::JoinSet,
};
use uuid::Uuid;

use crate::{
    game::{GameMessage, GameState},
    market::{
        order_book::{OrderRequest, TradeLeg},
        MarketMessage, MarketState, OrderRepr,
    },
    plants::{
        stack::{ProgramPlant, StackMessage, StackState},
        PowerPlantPublicRepr,
    },
};

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
pub struct PlayerConnectionActor {}

impl PlayerConnectionActor {
    pub async fn start(
        mut ws: WebSocket,
        player_id: String,
        game_tx: mpsc::Sender<GameMessage>,
        game_state: watch::Receiver<GameState>,
        market: Sender<MarketMessage>,
        market_state: watch::Receiver<MarketState>,
        stack: Sender<StackMessage>,
        stack_state: watch::Receiver<StackState>,
    ) {
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

        register_connection(&player_id, &market, &stack, &connection_id, tx).await;

        let (sink, stream) = ws.split();
        let sink_handle = tokio::spawn(process_internal_messages(
            sink,
            rx,
            game_state,
            market_state,
            stack_state,
        ));
        let stream_handle = tokio::spawn(process_ws_messages(
            stream,
            game_tx.clone(),
            market.clone(),
            stack.clone(),
            player_id.clone(),
        ));
        let _ = tokio::try_join!(sink_handle, stream_handle);

        // One side of the WS is closed and/or cannot be processed, disconnect player connection
        // from market and return
        println!("Disconnecting {player_id:?} from market");
        let _ = market
            .send(MarketMessage::PlayerDisconnection { connection_id })
            .await;
    }
}

async fn register_connection(
    player_id: &str,
    market: &Sender<MarketMessage>,
    stack: &Sender<StackMessage>,
    connection_id: &str,
    tx: Sender<PlayerMessage>,
) {
    let mut set = JoinSet::new();
    let p_id = player_id.to_owned();
    let m_tx = market.clone();
    let c_id = connection_id.to_owned();
    let tx_cloned = tx.clone();
    set.spawn(async move {
        connect_to_market(p_id, m_tx, c_id, tx_cloned).await;
    });

    let p_id = player_id.to_owned();
    let s_tx = stack.clone();
    let c_id = connection_id.to_owned();
    let tx_cloned = tx.clone();
    set.spawn(async move {
        connect_to_stack(p_id, s_tx, c_id, tx_cloned).await;
    });
    set.join_all().await;
}

async fn connect_to_stack(
    player_id: String,
    stack: Sender<StackMessage>,
    connection_id: String,
    tx: Sender<PlayerMessage>,
) -> Option<()> {
    stack
        .send(StackMessage::RegisterPlayerConnection(PlayerConnection {
            id: connection_id,
            player_id,
            tx,
        }))
        .await
        .ok()
}

async fn connect_to_market(
    player_id: String,
    market: Sender<MarketMessage>,
    connection_id: String,
    tx: Sender<PlayerMessage>,
) -> Option<()> {
    market
        .send(MarketMessage::NewPlayerConnection(PlayerConnection {
            id: connection_id,
            player_id,
            tx,
        }))
        .await
        .ok()
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
