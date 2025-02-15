use core::fmt;
use std::{collections::HashMap, time::Duration};

use futures_util::future::join_all;
use tokio::{
    join,
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{
    market::{order_book::Trade, MarketMessage},
    plants::{stack::StackMessage, PlantOutput},
};

use super::GameMessage;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DeliveryPeriodId(isize);
impl fmt::Display for DeliveryPeriodId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<isize> for DeliveryPeriodId {
    fn from(value: isize) -> Self {
        DeliveryPeriodId(value)
    }
}
impl DeliveryPeriodId {
    pub fn previous(&self) -> DeliveryPeriodId {
        DeliveryPeriodId(self.0 - 1)
    }
    pub fn next(&self) -> DeliveryPeriodId {
        DeliveryPeriodId(self.0 + 1)
    }
}
#[derive(Debug)]
pub struct DeliveryPeriodResults {
    pub period_id: DeliveryPeriodId,
}

pub async fn start_delivery_period(
    period_id: DeliveryPeriodId,
    game_tx: mpsc::Sender<GameMessage>,
    market_tx: mpsc::Sender<MarketMessage>,
    stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
    players_ready_rx: oneshot::Receiver<()>,
) {
    // First, open market and stacks
    let market_tx_cloned = market_tx.clone();
    let previous_period = period_id.previous();
    let open_market =
        tokio::spawn(async move { open_market(market_tx_cloned, previous_period).await });

    let stacks_tx_cloned = stacks_tx.clone();
    let previous_period = period_id.previous();
    let open_stacks =
        tokio::spawn(async move { open_stacks(stacks_tx_cloned, previous_period).await });

    let _ = open_market.await;
    let _ = open_stacks.await;

    // Close market and stacks when time has elapsed
    let current_period = period_id;
    let market_tx_cloned = market_tx.clone();
    let stacks_tx_cloned = stacks_tx.clone();
    let timers_handle = tokio::spawn(async move {
        join!(
            close_market_future(market_tx_cloned, current_period),
            close_stacks_future(stacks_tx_cloned, current_period)
        )
    });

    // Close market and stacks if all players are ready
    let market_tx_cloned = market_tx.clone();
    let stacks_tx_cloned = stacks_tx.clone();
    let players_ready_handle = tokio::spawn(async move {
        let _ = players_ready_rx.await;
        println!("All players ready, closing delivery period early");
        join!(
            close_market(market_tx_cloned, period_id),
            close_stacks(stacks_tx_cloned, period_id)
        )
    });

    let (trades, plants_outputs) = tokio::select! {
        Ok(res_timers) = timers_handle => res_timers,
        Ok(res_players) = players_ready_handle => res_players,
    };

    println!("Delivery period ended with trades: {trades:?} and plant outputs: {plants_outputs:?}");
    let _ = game_tx
        .send(GameMessage::DeliveryPeriodResults(DeliveryPeriodResults {
            period_id,
        }))
        .await;
}

async fn close_market_future(
    market_tx: mpsc::Sender<MarketMessage>,
    period_id: DeliveryPeriodId,
) -> Vec<Trade> {
    sleep(Duration::from_secs(240)).await;
    close_market(market_tx, period_id).await
}

async fn close_market(
    market_tx: mpsc::Sender<MarketMessage>,
    period_id: DeliveryPeriodId,
) -> Vec<Trade> {
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
            .map(|stack_tx| stack_tx.send(StackMessage::OpenStack(period_id))),
    )
    .await;
}

async fn close_stacks_future(
    stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
    period_id: DeliveryPeriodId,
) -> HashMap<String, HashMap<String, PlantOutput>> {
    sleep(Duration::from_secs(300)).await;
    close_stacks(stacks_tx, period_id).await
}

async fn close_stacks(
    stacks_tx: HashMap<String, mpsc::Sender<StackMessage>>,
    period_id: DeliveryPeriodId,
) -> HashMap<String, HashMap<String, PlantOutput>> {
    join_all(
        stacks_tx
            .iter()
            .map(|(player_id, stack_tx)| close_stack(player_id, period_id, stack_tx.clone())),
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

    use tokio::sync::{mpsc, oneshot};

    use crate::{
        game::{
            delivery_period::{start_delivery_period, DeliveryPeriodId},
            GameMessage,
        },
        market::MarketMessage,
        plants::stack::StackMessage,
    };

    #[tokio::test(start_paused = true)]
    async fn test_delivery_period_lifecycle() {
        let period = DeliveryPeriodId::from(1);
        let (game_tx, mut game_rx) = mpsc::channel(16);
        let (market_tx, mut market_rx) = mpsc::channel(16);
        let (stack_tx, mut stack_rx) = mpsc::channel(16);
        let stacks_tx = HashMap::from([("toto".to_string(), stack_tx)]);
        let (_, players_ready_rx) = oneshot::channel();

        tokio::spawn(async move {
            start_delivery_period(period, game_tx, market_tx, stacks_tx, players_ready_rx).await;
        });

        // Open the market and the stacks
        let Some(MarketMessage::OpenMarket(period_id)) = market_rx.recv().await else {
            unreachable!("Should have opened the market");
        };
        assert_eq!(period_id, period.previous());
        let Some(StackMessage::OpenStack(period_id)) = stack_rx.recv().await else {
            unreachable!("Should have opened the stack");
        };
        assert_eq!(period_id, period.previous());

        // Close the market
        let Some(MarketMessage::CloseMarket { tx_back, period_id }) = market_rx.recv().await else {
            unreachable!("Should have closed the market");
        };
        assert_eq!(period_id, period);
        let _ = tx_back.send(Vec::new());

        // Close the stacks
        let Some(StackMessage::CloseStack { tx_back, period_id }) = stack_rx.recv().await else {
            unreachable!("Should have closed the stacks");
        };
        assert_eq!(period_id, period);
        let _ = tx_back.send(HashMap::new());

        // Should publish its results back to the game actor
        let Some(GameMessage::DeliveryPeriodResults(results)) = game_rx.recv().await else {
            unreachable!("Should have received results for the delivery period")
        };
        assert_eq!(results.period_id, period_id);
    }

    #[tokio::test]
    async fn test_should_end_early_if_all_players_are_ready() {
        let period = DeliveryPeriodId::from(1);
        let (game_tx, mut game_rx) = mpsc::channel(16);
        let (market_tx, mut market_rx) = mpsc::channel(16);
        let (stack_tx, mut stack_rx) = mpsc::channel(16);
        let (players_ready_tx, players_ready_rx) = oneshot::channel();
        let stacks_tx = HashMap::from([("toto".to_string(), stack_tx)]);

        tokio::spawn(async move {
            start_delivery_period(period, game_tx, market_tx, stacks_tx, players_ready_rx).await;
        });

        // Open the market and the stacks
        let Some(MarketMessage::OpenMarket(period_id)) = market_rx.recv().await else {
            unreachable!("Should have opened the market");
        };
        assert_eq!(period_id, period.previous());
        let Some(StackMessage::OpenStack(period_id)) = stack_rx.recv().await else {
            unreachable!("Should have opened the stack");
        };
        assert_eq!(period_id, period.previous());

        // All players are ready, delivery period should end early
        let _ = players_ready_tx.send(());

        // Close the market
        let Some(MarketMessage::CloseMarket { tx_back, period_id }) = market_rx.recv().await else {
            unreachable!("Should have closed the market");
        };
        assert_eq!(period_id, period);
        let _ = tx_back.send(Vec::new());

        // Close the stacks
        let Some(StackMessage::CloseStack { tx_back, period_id }) = stack_rx.recv().await else {
            unreachable!("Should have closed the stacks");
        };
        assert_eq!(period_id, period);
        let _ = tx_back.send(HashMap::new());

        // Should publish its results back to the game actor
        let Some(GameMessage::DeliveryPeriodResults(results)) = game_rx.recv().await else {
            unreachable!("Should have received results for the delivery period")
        };
        assert_eq!(results.period_id, period_id);
    }
}
