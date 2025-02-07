use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

use crate::market::{order_book::OrderRequest, MarketMessage, Player, PlayerMessage};

#[derive(Deserialize, Debug)]
enum WebSocketIncomingMessage {
    OrderRequest(OrderRequest),
}

pub struct PlayerActor {}

impl PlayerActor {
    pub async fn start(ws: WebSocket, market_tx: Sender<MarketMessage>) {
        let (tx, rx) = channel::<PlayerMessage>(16);

        let player_id = Uuid::new_v4().to_string();
        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: player_id.clone(),
                tx: tx.clone(),
            }))
            .await;

        let (sink, stream) = ws.split();
        tokio::join!(
            process_internal_messages(sink, rx),
            process_ws_messages(stream, market_tx.clone(), player_id)
        );
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
        let _ = sink.send(Message::text(msg)).await;
    }
}
