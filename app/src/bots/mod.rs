use initial_orders::InitialOrdersBot;
use tokio::sync::mpsc::Sender;

use crate::market::MarketMessage;

pub mod initial_orders;

pub async fn start_bots(market_tx: Sender<MarketMessage>) {
    let mut initial_orders_bots = InitialOrdersBot::new(market_tx.clone());

    tokio::spawn(async move { initial_orders_bots.start().await });
}
