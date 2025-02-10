use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    future::join_all,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

use crate::{
    market::{order_book::OrderRequest, MarketMessage, PlayerConnection, PlayerMessage},
    plants::stack::StackMessage,
};

#[derive(Deserialize, Debug)]
enum WebSocketIncomingMessage {
    ConnectionReady,
    OrderRequest(OrderRequest),
}

pub struct PlayerConnectionActor {}

impl PlayerConnectionActor {
    pub async fn start(
        mut ws: WebSocket,
        player_id: String,
        market: Sender<MarketMessage>,
        stack: Sender<StackMessage>,
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

        join_all([
            market.send(MarketMessage::NewPlayerConnection(PlayerConnection {
                id: connection_id.clone(),
                player_id: player_id.clone(),
                tx: tx.clone(),
            })),
        ])
        .await;

        let (sink, stream) = ws.split();
        let sink_handle = tokio::spawn(process_internal_messages(sink, rx));
        let stream_handle = tokio::spawn(process_ws_messages(
            stream,
            market.clone(),
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

async fn process_ws_messages(
    mut stream: SplitStream<WebSocket>,
    market_tx: Sender<MarketMessage>,
    player_id: String,
) {
    while let Some(Ok(Message::Text(msg))) = stream.next().await {
        match serde_json::from_str::<WebSocketIncomingMessage>(msg.as_str()) {
            Ok(WebSocketIncomingMessage::OrderRequest(mut request)) => {
                request.owner = player_id.clone();
                let _ = market_tx.send(MarketMessage::OrderRequest(request)).await;
            }
            Ok(WebSocketIncomingMessage::ConnectionReady) => { /* Only for WS initialisation */ }
            Err(err) => println!("{err:?}"),
        }
    }
}

async fn process_internal_messages(
    mut sink: SplitSink<WebSocket, Message>,
    mut rx: Receiver<PlayerMessage>,
) {
    while let Some(msg) = rx.recv().await {
        let Ok(msg) = serde_json::to_string(&msg) else {
            println!("Unable to serialize message: {msg:?}");
            return;
        };
        if sink.send(Message::text(msg)).await.is_err() {
            return;
        }
    }
}
