use core::fmt;
use std::{collections::HashMap, time::Duration};

use futures_util::future::join_all;
use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{
    market::{order_book::Trade, MarketMessage},
    plants::{stack::StackMessage, PlantOutput},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeliveryPeriodId(usize);
impl fmt::Display for DeliveryPeriodId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<usize> for DeliveryPeriodId {
    fn from(value: usize) -> Self {
        DeliveryPeriodId(value)
    }
}
impl DeliveryPeriodId {
    pub fn next(&self) -> DeliveryPeriodId {
        DeliveryPeriodId(self.0 + 1)
    }
}

pub struct DeliveryPeriod {
    market_tx: mpsc::Sender<MarketMessage>,
    stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
}

impl DeliveryPeriod {
    pub fn new(
        market_tx: mpsc::Sender<MarketMessage>,
        stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
    ) -> DeliveryPeriod {
        DeliveryPeriod {
            market_tx,
            stacks_tx,
        }
    }

    pub async fn start(&mut self) {
        // First, open market and stacks
        let delivery_period_id = DeliveryPeriodId::from(0);
        let market_tx = self.market_tx.clone();
        let period_id = delivery_period_id.clone();
        let open_market = tokio::spawn(async move { open_market(market_tx, period_id).await });

        let stacks_tx = self.stacks_tx.clone();
        let period_id = delivery_period_id.clone();
        let open_stacks = tokio::spawn(async move { open_stacks(stacks_tx, period_id).await });

        let _ = open_market.await;
        let _ = open_stacks.await;

        // Close market and stacks when time has elapsed
        let market_tx = self.market_tx.clone();
        let period_id = delivery_period_id.clone();
        let close_market_handle =
            tokio::spawn(async move { close_market(market_tx, period_id).await });

        let stacks_tx = self.stacks_tx.clone();
        let period_id = delivery_period_id.clone();
        let close_stacks_handle =
            tokio::spawn(async move { close_stacks(stacks_tx, period_id).await });

        // Compute post-delivery results
        let trades = close_market_handle.await;
        let plants_outputs = close_stacks_handle.await;

        println!(
            "Delivery period ended with trades: {trades:?} and plant outputs: {plants_outputs:?}"
        );
    }
}

async fn close_market(
    market_tx: mpsc::Sender<MarketMessage>,
    period_id: DeliveryPeriodId,
) -> Vec<Trade> {
    sleep(Duration::from_secs(240)).await;

    let (tx_back, rx) = oneshot::channel();
    let _ = market_tx
        .send(MarketMessage::CloseMarket { tx_back, period_id })
        .await;

    rx.await.unwrap()
}

async fn open_market(market_tx: mpsc::Sender<MarketMessage>, period_id: DeliveryPeriodId) {
    let _ = market_tx.send(MarketMessage::OpenMarket(period_id)).await;
}

async fn open_stacks(
    stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
    period_id: DeliveryPeriodId,
) {
    join_all(
        stacks_tx
            .values()
            .map(|stack_tx| stack_tx.send(StackMessage::OpenStack(period_id.clone()))),
    )
    .await;
}

async fn close_stacks(
    stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
    period_id: DeliveryPeriodId,
) -> HashMap<String, HashMap<String, PlantOutput>> {
    sleep(Duration::from_secs(300)).await;

    join_all(
        stacks_tx.iter().map(|(player_id, stack_tx)| {
            close_stack(player_id, period_id.clone(), stack_tx.clone())
        }),
    )
    .await
    .into_iter()
    .collect()
}

async fn close_stack(
    player_id: &str,
    period_id: DeliveryPeriodId,
    stack: mpsc::Sender<StackMessage>,
) -> (String, HashMap<String, PlantOutput>) {
    let (tx_back, rx) = oneshot::channel();

    let _ = stack
        .send(StackMessage::CloseStack { tx_back, period_id })
        .await;

    let plant_outputs = rx.await.unwrap();

    (player_id.to_string(), plant_outputs)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tokio::sync::mpsc;

    use crate::{
        game::delivery_period::DeliveryPeriod, market::MarketMessage, plants::stack::StackMessage,
    };

    #[tokio::test(start_paused = true)]
    async fn test_delivery_period_lifecycle() {
        let (market_tx, mut market_rx) = mpsc::channel::<MarketMessage>(16);
        let (stack_tx, mut stack_rx) = mpsc::channel::<StackMessage>(16);
        let stacks_tx = HashMap::from([("toto".to_string(), stack_tx)]);

        // let delivery_period_id = DeliveryPeriodId::from(0);
        let mut delivery_period = DeliveryPeriod::new(market_tx, stacks_tx);

        tokio::spawn(async move {
            delivery_period.start().await;
        });

        // Open the market and the stacks
        let Some(MarketMessage::OpenMarket(_)) = market_rx.recv().await else {
            unreachable!("Should have opened the market");
        };
        let Some(StackMessage::OpenStack(_)) = stack_rx.recv().await else {
            unreachable!("Should have opened the stack");
        };

        // Close the market
        let Some(MarketMessage::CloseMarket { tx_back, .. }) = market_rx.recv().await else {
            unreachable!("Should have closed the market");
        };
        let _ = tx_back.send(Vec::new());

        // Close the stacks
        let Some(StackMessage::CloseStack { tx_back, .. }) = stack_rx.recv().await else {
            unreachable!("Should have closed the stacks");
        };
        let _ = tx_back.send(HashMap::new());
    }
}
