use initial_orders::InitialOrdersBot;
use tokio::sync::{mpsc::Sender, oneshot};

use crate::game::GameMessage;

pub mod initial_orders;

pub async fn start_bots(game: Sender<GameMessage>) {
    let (tx_back, rx) = oneshot::channel();
    let _ = game.send(GameMessage::GetMarketTx { tx_back }).await;
    let Ok(market) = rx.await else {
        println!("Unable to get market tx for starting bots");
        return;
    };

    let mut initial_orders_bots = InitialOrdersBot::new(market);

    tokio::spawn(async move { initial_orders_bots.start().await });
}
