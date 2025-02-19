use core::fmt;
use std::{collections::HashMap, time::Duration};

use futures_util::future::join_all;
use tokio::{
    join,
    sync::{mpsc, oneshot},
    time::sleep,
};

use crate::{
    game::scores::compute_players_scores,
    market::{order_book::Trade, Market},
    plants::{PlantId, PlantOutput, Stack},
    player::PlayerId,
};

use super::{scores::PlayerScore, GameMessage};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default)]
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

pub struct DeliveryPeriodTimers {
    pub market: Duration,
    pub stacks: Duration,
}

#[derive(Debug)]
pub struct DeliveryPeriodResults {
    pub period_id: DeliveryPeriodId,
    pub players_scores: HashMap<PlayerId, PlayerScore>,
}

pub async fn start_delivery_period<StkS, MS>(
    period_id: DeliveryPeriodId,
    game_tx: mpsc::Sender<GameMessage>,
    market_service: MS,
    stack_services: HashMap<PlayerId, StkS>,
    players_ready_rx: oneshot::Receiver<()>,
    timers: Option<DeliveryPeriodTimers>,
) where
    StkS: Stack,
    MS: Market,
{
    // First, open market and stacks
    let market_service_cloned = market_service.clone();
    let previous_period = period_id.previous();
    let open_market =
        tokio::spawn(async move { open_market(market_service_cloned, previous_period).await });

    let stack_services_cloned = stack_services.clone();
    let previous_period = period_id.previous();
    let open_stacks =
        tokio::spawn(async move { open_stacks(stack_services_cloned, previous_period).await });

    let _ = open_market.await;
    let _ = open_stacks.await;

    let mut set = tokio::task::JoinSet::new();

    if let Some(timers) = timers {
        // Close market and stacks when time has elapsed
        let current_period = period_id;
        let market_service_cloned = market_service.clone();
        let stack_services_cloned = stack_services.clone();
        set.spawn(async move {
            join!(
                close_market_future(market_service_cloned, current_period, timers.market),
                close_stacks_future(stack_services_cloned, current_period, timers.stacks)
            )
        });
    }

    // Close market and stacks if all players are ready
    set.spawn(async move {
        let _ = players_ready_rx.await;
        println!("All players ready, closing delivery period early");
        join!(
            market_service.close_market(period_id),
            close_stacks(stack_services, period_id)
        )
    });

    let (trades, plants_outputs) = loop {
        match set.join_next().await {
            Some(Ok(result)) => {
                // Got successful result, abort remaining tasks and break
                set.abort_all();
                break result;
            }
            Some(Err(e)) => {
                println!("Task failed: {e}, trying next task");
                // Continue to next task if there is one
                continue;
            }
            None => {
                // No more tasks to try
                panic!("All tasks failed or JoinSet was empty");
            }
        }
    };

    let scores = compute_players_scores(trades, plants_outputs);
    println!("Delivery period ended: {scores:?}");
    let _ = game_tx
        .send(GameMessage::DeliveryPeriodResults(DeliveryPeriodResults {
            period_id,
            players_scores: scores,
        }))
        .await;
}

async fn close_market_future<MS>(
    market: MS,
    period_id: DeliveryPeriodId,
    duration: Duration,
) -> Vec<Trade>
where
    MS: Market,
{
    sleep(duration).await;
    market.close_market(period_id).await
}

async fn open_market<MS>(market: MS, period_id: DeliveryPeriodId)
where
    MS: Market,
{
    let _ = market.open_market(period_id).await;
}

async fn open_stacks<StkS>(stacks: HashMap<PlayerId, StkS>, period_id: DeliveryPeriodId)
where
    StkS: Stack,
{
    join_all(stacks.values().map(|stack| stack.open_stack(period_id))).await;
}

async fn close_stacks_future<StkS>(
    stacks: HashMap<PlayerId, StkS>,
    period_id: DeliveryPeriodId,
    duration: Duration,
) -> HashMap<PlayerId, HashMap<PlantId, PlantOutput>>
where
    StkS: Stack,
{
    sleep(duration).await;
    close_stacks(stacks, period_id).await
}

async fn close_stacks<StkS>(
    stacks: HashMap<PlayerId, StkS>,
    period_id: DeliveryPeriodId,
) -> HashMap<PlayerId, HashMap<PlantId, PlantOutput>>
where
    StkS: Stack,
{
    join_all(
        stacks
            .iter()
            .map(|(player_id, stack)| close_stack(player_id, period_id, stack)),
    )
    .await
    .into_iter()
    .collect()
}

async fn close_stack<StkS>(
    player_id: &PlayerId,
    period_id: DeliveryPeriodId,
    stack: &StkS,
) -> (PlayerId, HashMap<PlantId, PlantOutput>)
where
    StkS: Stack,
{
    let plant_outputs = stack.close_stack(period_id).await.unwrap_or(HashMap::new());

    (player_id.clone(), plant_outputs)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, time::Duration};

    use futures::future;
    use mockall::{predicate::eq, Sequence};
    use tokio::sync::{mpsc, oneshot};

    use crate::{
        game::{
            delivery_period::{start_delivery_period, DeliveryPeriodId, DeliveryPeriodTimers},
            GameMessage,
        },
        market::service::MockMarketService,
        plants::service::MockStackService,
        player::PlayerId,
    };

    #[tokio::test]
    async fn test_delivery_period_lifecycle() {
        let period = DeliveryPeriodId::from(1);
        let (game_tx, mut game_rx) = mpsc::channel(16);
        //Mocked market service
        let mut market_service = MockMarketService::new();
        let mut mocked_market_seq = Sequence::new();
        market_service
            .expect_clone()
            .once()
            .in_sequence(&mut mocked_market_seq)
            .returning(|| {
                // First clone to open the market
                let mut mocked = MockMarketService::new();
                mocked
                    .expect_open_market()
                    .once()
                    .with(eq(DeliveryPeriodId::from(0)))
                    .returning(|_| Box::pin(future::ready(())));
                mocked
            });
        market_service
            .expect_clone()
            .once()
            .in_sequence(&mut mocked_market_seq)
            .returning(|| {
                // Second clone to close the market in the timer branch
                let mut mocked = MockMarketService::new();
                mocked
                    .expect_close_market()
                    .with(eq(DeliveryPeriodId::from(1)))
                    .once()
                    .returning(|_| Box::pin(future::ready(Vec::new())));
                mocked
            });
        market_service.expect_close_market().never();

        // Mocked stack service
        let mut stack_service = MockStackService::new();
        let mut mocked_stack_seq = Sequence::new();
        stack_service
            .expect_clone()
            .once()
            .in_sequence(&mut mocked_stack_seq)
            .returning(|| {
                // First clone to open the stack
                let mut mocked = MockStackService::new();
                mocked
                    .expect_open_stack()
                    .once()
                    .with(eq(DeliveryPeriodId::from(0)))
                    .returning(|_| Box::pin(future::ready(())));
                mocked
            });
        stack_service
            .expect_clone()
            .once()
            .in_sequence(&mut mocked_stack_seq)
            .returning(|| {
                // Second clone to close the stack in the timer branch
                let mut mocked = MockStackService::new();
                mocked
                    .expect_close_stack()
                    .with(eq(DeliveryPeriodId::from(1)))
                    .once()
                    .returning(|_| Box::pin(future::ready(Ok(HashMap::new()))));
                mocked
            });
        stack_service.expect_close_stack().never();
        let stacks_services = HashMap::from([(PlayerId::from("toto"), stack_service)]);

        // Keep _players_ready_tx around to not dropt the channel and trigger early closing
        let (_players_ready_tx, players_ready_rx) = oneshot::channel();
        let timers = Some(DeliveryPeriodTimers {
            market: Duration::from_micros(1),
            stacks: Duration::from_micros(1),
        });

        tokio::spawn(async move {
            start_delivery_period(
                period,
                game_tx,
                market_service,
                stacks_services,
                players_ready_rx,
                timers,
            )
            .await;
        });

        // Should publish its results back to the game actor
        let Some(GameMessage::DeliveryPeriodResults(results)) = game_rx.recv().await else {
            unreachable!("Should have received results for the delivery period")
        };
        assert_eq!(results.period_id, period);
    }

    #[tokio::test]
    async fn test_should_end_early_if_all_players_are_ready() {
        let period = DeliveryPeriodId::from(1);
        let (game_tx, mut game_rx) = mpsc::channel(16);
        // let (market_tx, mut market_rx) = mpsc::channel(16);
        let (players_ready_tx, players_ready_rx) = oneshot::channel();

        //Mocked market service
        let mut market_service = MockMarketService::new();
        let mut mocked_market_seq = Sequence::new();
        market_service
            .expect_clone()
            .once()
            .in_sequence(&mut mocked_market_seq)
            .returning(|| {
                // First clone to open the market
                let mut mocked = MockMarketService::new();
                mocked
                    .expect_open_market()
                    .once()
                    .with(eq(DeliveryPeriodId::from(0)))
                    .returning(|_| Box::pin(future::ready(())));
                mocked
            });
        market_service
            .expect_close_market()
            .with(eq(DeliveryPeriodId::from(1)))
            .once()
            .returning(|_| Box::pin(future::ready(Vec::new())));

        // Mocked stack service
        let mut stack_service = MockStackService::new();
        let mut mocked_stack_seq = Sequence::new();
        stack_service
            .expect_clone()
            .once()
            .in_sequence(&mut mocked_stack_seq)
            .returning(|| {
                // First clone to open the stack
                let mut mocked = MockStackService::new();
                mocked
                    .expect_open_stack()
                    .once()
                    .with(eq(DeliveryPeriodId::from(0)))
                    .returning(|_| Box::pin(future::ready(())));
                mocked
            });
        // No second close as timers = None, and corresponding branch will not exist
        stack_service
            .expect_close_stack()
            .with(eq(DeliveryPeriodId::from(1)))
            .once()
            .returning(|_| Box::pin(future::ready(Ok(HashMap::new()))));
        let stacks_services = HashMap::from([(PlayerId::from("toto"), stack_service)]);

        let timers = None;

        tokio::spawn(async move {
            start_delivery_period(
                period,
                game_tx,
                market_service,
                stacks_services,
                players_ready_rx,
                timers,
            )
            .await;
        });

        // All players are ready, delivery period should end early
        let _ = players_ready_tx.send(());

        // Should publish its results back to the game actor
        let Some(GameMessage::DeliveryPeriodResults(results)) = game_rx.recv().await else {
            unreachable!("Should have received results for the delivery period")
        };
        assert_eq!(results.period_id, period);
    }
}
