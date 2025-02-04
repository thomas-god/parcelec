use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::Deserialize;
use tokio::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use crate::{
    market::{MarketMessage, Player, PlayerMessage},
    order_book::OrderRequest,
};

#[derive(Deserialize, Debug)]
enum WebSocketIncomingMessage {
    OrderRequest(OrderRequest),
}

pub struct PlayerActor {}

impl PlayerActor {
    pub async fn start(stream: TcpStream, market_tx: Sender<MarketMessage>) {
        let (tx, rx) = channel::<PlayerMessage>(16);

        let ws_stream = accept_async(stream).await.expect("Failed to accept");

        let _ = market_tx
            .send(MarketMessage::NewPlayer(Player {
                id: Uuid::new_v4().to_string(),
                tx: tx.clone(),
            }))
            .await;

        let (sink, stream) = ws_stream.split();
        tokio::join!(
            process_internal_messages(sink, rx),
            process_ws_messages(stream, market_tx.clone())
        );
    }
}

async fn process_ws_messages(
    mut stream: SplitStream<WebSocketStream<TcpStream>>,
    market_tx: Sender<MarketMessage>,
) {
    while let Some(Ok(msg)) = stream.next().await {
        if msg.is_text() {
            let Ok(content) = msg.into_text().map(|s| s.to_string()) else {
                return;
            };

            match serde_json::from_str::<WebSocketIncomingMessage>(&content) {
                Ok(WebSocketIncomingMessage::OrderRequest(request)) => {
                    let _ = market_tx.send(MarketMessage::OrderRequest(request)).await;
                }
                Err(err) => println!("{err:?}"),
            }
        }
    }
}

async fn process_internal_messages(
    mut sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut rx: Receiver<PlayerMessage>,
) {
    while let Some(msg) = rx.recv().await {
        let Ok(msg) = serde_json::to_string(&msg) else {
            return;
        };
        let _ = sink.send(Message::text(msg)).await;
    }
}
