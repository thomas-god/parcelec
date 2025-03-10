use std::collections::HashMap;

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use tokio::sync::{
    mpsc::{self, Receiver, Sender, channel},
    oneshot, watch,
};
use tokio_util::sync::CancellationToken;

use crate::{
    forecast::ForecastLevel,
    game::{GameId, delivery_period::DeliveryPeriodId},
    plants::PlantOutput,
    player::{PlayerId, connection::PlayerMessage, repository::ConnectionRepositoryMessage},
};

use super::{
    PlantId, PowerPlant, PowerPlantPublicRepr, Stack, StackService,
    technologies::{
        battery::Battery, consumers::Consumers, gas_plant::GasPlant, nuclear::NuclearPlant,
        renewable::RenewablePlant,
    },
};

#[derive(Debug, Deserialize)]
pub struct ProgramPlant {
    pub plant_id: PlantId,
    pub setpoint: isize,
}

#[derive(Debug)]
pub enum StackMessage {
    OpenStack(DeliveryPeriodId),
    CloseStack {
        period_id: DeliveryPeriodId,
        tx_back: oneshot::Sender<HashMap<PlantId, PlantOutput>>,
    },
    ProgramSetpoint(ProgramPlant),
    GetSnapshot(oneshot::Sender<HashMap<PlantId, PowerPlantPublicRepr>>),
    GetForecasts(oneshot::Sender<HashMap<PlantId, Option<ForecastLevel>>>),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StackState {
    Open,
    Closed,
}

impl Serialize for StackState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("StackState", 2)?;
        state.serialize_field("type", "StackState")?;
        state.serialize_field(
            "state",
            match self {
                Self::Closed => "Closed",
                Self::Open => "Open",
            },
        )?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct StackContext<PS: Stack> {
    pub service: PS,
    pub state_rx: watch::Receiver<StackState>,
}

/// A stack is the collection of power plants owned by a given player
pub struct StackActor {
    game: GameId,
    state: StackState,
    state_sender: watch::Sender<StackState>,
    delivery_period: DeliveryPeriodId,
    player: PlayerId,
    stack: HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>>,
    tx: Sender<StackMessage>,
    rx: Receiver<StackMessage>,
    players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
    past_outputs: HashMap<DeliveryPeriodId, HashMap<PlantId, PlantOutput>>,
    cancellation_token: CancellationToken,
}

impl StackActor {
    pub fn new(
        game: GameId,
        player: PlayerId,
        initial_state: StackState,
        delivery_period: DeliveryPeriodId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
        cancellation_token: CancellationToken,
    ) -> StackActor {
        let (state_tx, _) = watch::channel(initial_state);
        let (tx, rx) = channel::<StackMessage>(16);

        StackActor {
            game,
            state: initial_state,
            state_sender: state_tx,
            delivery_period,
            player,
            stack: default_stack(),
            players_connections,
            past_outputs: HashMap::new(),
            tx,
            rx,
            cancellation_token,
        }
    }

    pub fn start(
        game: &GameId,
        player: &PlayerId,
        players_connections: mpsc::Sender<ConnectionRepositoryMessage>,
        cancellation_token: CancellationToken,
    ) -> StackContext<StackService> {
        let mut stack = StackActor::new(
            game.clone(),
            player.clone(),
            StackState::Closed,
            DeliveryPeriodId::default(),
            players_connections,
            cancellation_token,
        );
        let context = stack.get_context();

        tokio::spawn(async move {
            stack.run().await;
        });
        context
    }

    pub fn get_context(&self) -> StackContext<StackService> {
        StackContext {
            service: StackService::new(self.tx.clone()),
            state_rx: self.state_sender.subscribe(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(message) = self.rx.recv() => {
                    self.process_message(message).await;
                }
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("Stack actor for player {:?} terminated", self.player);
                    break;
                }
            }
        }
    }

    #[tracing::instrument(name = "StackActor::process_message", skip(self))]
    async fn process_message(&mut self, message: StackMessage) {
        use StackMessage::*;
        use StackState::*;
        match (&self.state, message) {
            (_, GetSnapshot(tx_back)) => {
                let _ = tx_back.send(self.stack_snapshot());
            }
            (_, GetForecasts(tx_back)) => {
                let _ = tx_back.send(self.stack_forecasts());
            }
            (Open, ProgramSetpoint(ProgramPlant { plant_id, setpoint })) => {
                self.program_plant_setpoint(plant_id, setpoint).await;
            }
            (
                Closed,
                ProgramSetpoint(ProgramPlant {
                    plant_id,
                    setpoint: _,
                }),
            ) => {
                tracing::warn!("Trying to program plant {plant_id:?} but stack is closed.");
            }
            (Closed, OpenStack(period_id)) => {
                if period_id == self.delivery_period {
                    self.state = StackState::Open;
                    self.delivery_period = self.delivery_period.next();
                    let _ = self.state_sender.send(StackState::Open);
                }
            }
            (Open, OpenStack(period)) => {
                tracing::warn!(
                    "Trying to open stack for delivery period {period:?}, but stack is already open."
                )
            }
            (Open, CloseStack { tx_back, period_id }) => {
                if period_id == self.delivery_period {
                    self.close_stack(period_id, tx_back).await;
                }
            }
            (Closed, CloseStack { period_id, tx_back }) => {
                if let Some(outputs) = self.past_outputs.get(&period_id) {
                    let _ = tx_back.send(outputs.clone());
                }
            }
        }
    }

    async fn send_stack_snapshot(&self) {
        let stack_snapshot = self.stack_snapshot();

        if let Err(err) = self
            .players_connections
            .send(ConnectionRepositoryMessage::SendToPlayer(
                self.game.clone(),
                self.player.clone(),
                PlayerMessage::StackSnapshot {
                    plants: stack_snapshot,
                },
            ))
            .await
        {
            tracing::error!(
                game_id = ?self.game,
                player_id = ?self.player,
                "Failed to send stack snapshot to player: {:?}", err
            );
        }
    }

    async fn send_stack_forecasts(&self) {
        let forecasts = self.stack_forecasts();

        if let Err(err) = self
            .players_connections
            .send(ConnectionRepositoryMessage::SendToPlayer(
                self.game.clone(),
                self.player.clone(),
                PlayerMessage::StackForecasts { forecasts },
            ))
            .await
        {
            tracing::error!(
                game_id = ?self.game,
                player_id = ?self.player,
                "Failed to send stack forecasts to player: {:?}", err
            );
        }
    }

    fn stack_snapshot(&self) -> HashMap<PlantId, PowerPlantPublicRepr> {
        self.stack
            .iter()
            .map(|(plant_id, plant)| (plant_id.to_owned(), plant.current_state()))
            .collect()
    }

    fn stack_forecasts(&self) -> HashMap<PlantId, Option<ForecastLevel>> {
        self.stack
            .iter()
            .map(|(plant_id, plant)| (plant_id.to_owned(), plant.get_forecast()))
            .collect()
    }

    async fn close_stack(
        &mut self,
        period_id: DeliveryPeriodId,
        tx_back: oneshot::Sender<HashMap<PlantId, PlantOutput>>,
    ) {
        // Update state
        self.state = StackState::Closed;

        // Collect outputs from plants
        let plant_outputs: HashMap<PlantId, PlantOutput> = self
            .stack
            .iter_mut()
            .map(|(plant_id, plant)| (plant_id.clone(), plant.dispatch()))
            .collect();

        // Store outputs for future reference
        self.past_outputs.insert(period_id, plant_outputs.clone());

        // Send outputs back to caller
        if let Err(err) = tx_back.send(plant_outputs) {
            tracing::error!(
                game_id = ?self.game,
                player_id = ?self.player,
                period_id = ?period_id,
                "Failed to send plant outputs back to requester: {:?}", err
            );
        }

        // Update state and notify
        if let Err(err) = self.state_sender.send(StackState::Closed) {
            tracing::error!(
                game_id = ?self.game,
                player_id = ?self.player,
                "Failed to broadcast stack state change: {:?}", err
            );
        }

        // Notify player about updated stack state
        self.send_stack_snapshot().await;
        self.send_stack_forecasts().await;
    }

    async fn program_plant_setpoint(&mut self, plant_id: PlantId, setpoint: isize) {
        if let Some(plant) = self.stack.get_mut(&plant_id) {
            let PlantOutput { cost, .. } = plant.program_setpoint(setpoint);
            tracing::info!("Programmed setpoint {setpoint} for plant {plant_id} (cost: {cost}");
            self.send_stack_snapshot().await;
        };
    }
}

fn default_stack() -> HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>> {
    let mut map: HashMap<PlantId, Box<dyn PowerPlant + Send + Sync>> = HashMap::new();
    map.insert(PlantId::default(), Box::new(Battery::new(300, 0)));
    map.insert(PlantId::default(), Box::new(GasPlant::new(80, 500)));
    map.insert(
        PlantId::default(),
        Box::new(RenewablePlant::new_with_looping(
            300,
            vec![0, 150, 300, 100],
        )),
    );
    map.insert(
        PlantId::default(),
        Box::new(Consumers::new_with_looping(
            1500,
            56,
            vec![-900, -1200, -600, -1800],
        )),
    );
    map.insert(PlantId::default(), Box::new(NuclearPlant::new(1000, 35)));
    map
}

#[cfg(test)]
mod tests_stack {
    use std::{collections::HashMap, time::Duration};

    use tokio::sync::{
        mpsc::{self, Sender},
        oneshot, watch,
    };
    use tokio_util::sync::CancellationToken;

    use crate::{
        game::{GameId, delivery_period::DeliveryPeriodId},
        plants::{
            PlantId, PowerPlantPublicRepr,
            actor::{ProgramPlant, StackActor, StackMessage, StackState},
        },
        player::{PlayerId, connection::PlayerMessage, repository::ConnectionRepositoryMessage},
    };

    fn start_stack() -> (
        PlayerId,
        mpsc::Sender<StackMessage>,
        watch::Receiver<StackState>,
        mpsc::Receiver<ConnectionRepositoryMessage>,
    ) {
        let game_id = GameId::default();
        let (conn_tx, conn_rx) = mpsc::channel(16);
        let player_id = PlayerId::default();
        let token = CancellationToken::new();
        let mut stack = StackActor::new(
            game_id,
            player_id.clone(),
            StackState::Open,
            DeliveryPeriodId::from(0),
            conn_tx,
            token,
        );
        let tx = stack.tx.clone();
        let state_rx = stack.state_sender.subscribe();
        tokio::spawn(async move {
            stack.run().await;
        });
        (player_id, tx, state_rx, conn_rx)
    }

    #[tokio::test]
    async fn test_get_snapshot() {
        let (_, tx, _, _) = start_stack();

        let (tx_back, rx) = oneshot::channel();
        let _ = tx.send(StackMessage::GetSnapshot(tx_back)).await;

        let Ok(_) = rx.await else {
            unreachable!();
        };
    }

    async fn get_stack_snashot(
        stack_tx: Sender<StackMessage>,
    ) -> HashMap<PlantId, PowerPlantPublicRepr> {
        let (tx, rx) = oneshot::channel();
        let _ = stack_tx.send(StackMessage::GetSnapshot(tx)).await;

        let Ok(snapshot) = rx.await else {
            unreachable!();
        };
        snapshot
    }

    #[tokio::test]
    async fn test_get_forecasts() {
        let (_, tx, _, _) = start_stack();

        let (tx_back, rx) = oneshot::channel();
        let _ = tx.send(StackMessage::GetForecasts(tx_back)).await;

        let Ok(_) = rx.await else {
            unreachable!();
        };
    }

    #[tokio::test]
    async fn test_programm_a_plant_setpoint() {
        let (_, tx, _, mut conn_rx) = start_stack();
        let plants = get_stack_snashot(tx.clone()).await;

        // Program a plant's setpoint
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = tx
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 100,
            }))
            .await;

        // Should receive a stack snapshot back
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::StackSnapshot { plants: _ },
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
    }

    #[tokio::test]
    async fn test_no_dispatch_when_stack_closed() {
        let (_, tx, _, mut conn_rx) = start_stack();
        let plants = get_stack_snashot(tx.clone()).await;

        // Close the stack
        let (tx_back, _) = oneshot::channel();
        let _ = tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        // Consume the stack snapshot and forecasts messages sent on stack closing
        let _ = conn_rx.recv().await;
        let _ = conn_rx.recv().await;

        // Try to send a dispatch command
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = tx
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 100,
            }))
            .await;

        // Should not receive a new stack snapshot
        tokio::select! {
        _ = conn_rx.recv() => {
            unreachable!("Should not have received a message");
        }
        _ = tokio::time::sleep(Duration::from_micros(1)) => {}
        };
    }
    #[tokio::test]
    async fn test_receive_plant_outputs_when_closing_stack() {
        let (_, tx, _, _) = start_stack();

        let plants = get_stack_snashot(tx.clone()).await;
        let plants_balance = plants.values().fold(0, |acc, plant| {
            acc + match plant {
                PowerPlantPublicRepr::Battery(batt) => batt.output.setpoint,
                PowerPlantPublicRepr::Consumers(cons) => cons.output.setpoint,
                PowerPlantPublicRepr::GasPlant(plant) => plant.output.setpoint,
                PowerPlantPublicRepr::RenewablePlant(plant) => plant.output.setpoint,
                PowerPlantPublicRepr::Nuclear(plant) => plant.output.setpoint,
            }
        });

        // Close the stack
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        let plant_outputs = rx_back
            .await
            .expect("Should have received a map of plant outputs");
        assert!(!plant_outputs.is_empty());
        assert_eq!(
            plants_balance,
            plants.values().fold(0, |acc, plant| {
                acc + match plant {
                    PowerPlantPublicRepr::Battery(batt) => batt.output.setpoint,
                    PowerPlantPublicRepr::Consumers(cons) => cons.output.setpoint,
                    PowerPlantPublicRepr::GasPlant(plant) => plant.output.setpoint,
                    PowerPlantPublicRepr::RenewablePlant(plant) => plant.output.setpoint,
                    PowerPlantPublicRepr::Nuclear(plant) => plant.output.setpoint,
                }
            })
        );
    }

    #[tokio::test]
    async fn test_stack_state_watch() {
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let player_id = PlayerId::default();
        let token = CancellationToken::new();
        let mut stack = StackActor::new(
            game_id,
            player_id.clone(),
            StackState::Open,
            DeliveryPeriodId::from(0),
            conn_tx,
            token,
        );
        let stack_tx = stack.tx.clone();
        let mut state_rx = stack.state_sender.subscribe();
        tokio::spawn(async move {
            stack.run().await;
        });

        assert_eq!(*state_rx.borrow(), StackState::Open);

        // Close the Stack
        let (tx_back, _) = oneshot::channel();
        let _ = stack_tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), StackState::Closed);

        // Reopen the Stack
        let _ = stack_tx
            .send(StackMessage::OpenStack(DeliveryPeriodId::from(0)))
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), StackState::Open);
    }

    #[tokio::test]
    async fn test_try_closing_stack_wrong_period_id_does_not_close_it() {
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let token = CancellationToken::new();
        let mut stack = StackActor::new(
            game_id,
            PlayerId::default(),
            StackState::Open,
            DeliveryPeriodId::from(1),
            conn_tx,
            token,
        );
        let stack_tx = stack.tx.clone();
        let mut state_rx = stack.state_sender.subscribe();
        tokio::spawn(async move {
            stack.run().await;
        });

        assert_eq!(*state_rx.borrow_and_update(), StackState::Open);

        // Try closing the stack with period ID greater than the actual one
        let (tx_back, _) = oneshot::channel();
        let _ = stack_tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(2),
            })
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), StackState::Open);

        // Try closing the stack with period ID smaller than the actual one
        let (tx_back, _) = oneshot::channel();
        let _ = stack_tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), StackState::Open);
    }

    #[tokio::test]
    async fn test_opening_stack_wrong_period_id_does_not_open_it() {
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let token = CancellationToken::new();
        let mut stack = StackActor::new(
            game_id,
            PlayerId::default(),
            StackState::Closed,
            DeliveryPeriodId::from(1),
            conn_tx,
            token,
        );
        let stack_tx = stack.tx.clone();
        let mut state_rx = stack.state_sender.subscribe();
        tokio::spawn(async move {
            stack.run().await;
        });

        assert_eq!(*state_rx.borrow_and_update(), StackState::Closed);

        // Try openning the stack with period ID greater than the actual one
        let _ = stack_tx
            .send(StackMessage::OpenStack(DeliveryPeriodId::from(2)))
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), StackState::Closed);

        // Try closing the stack with period ID smaller than the actual one
        let _ = stack_tx
            .send(StackMessage::OpenStack(DeliveryPeriodId::from(0)))
            .await;
        tokio::time::sleep(Duration::from_micros(1)).await;
        assert_eq!(*state_rx.borrow_and_update(), StackState::Closed);
    }

    #[tokio::test]
    async fn test_open_stack_then_close_next_period() {
        let game_id = GameId::default();
        let (conn_tx, _) = mpsc::channel(16);
        let token = CancellationToken::new();
        let mut stack = StackActor::new(
            game_id,
            PlayerId::default(),
            StackState::Closed,
            DeliveryPeriodId::from(1),
            conn_tx,
            token,
        );
        let stack_tx = stack.tx.clone();
        let mut state_rx = stack.state_sender.subscribe();
        tokio::spawn(async move {
            stack.run().await;
        });

        // Open the stack
        let _ = stack_tx
            .send(StackMessage::OpenStack(DeliveryPeriodId::from(1)))
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), StackState::Open);

        // Close the stack with next period id
        let (tx_back, _) = oneshot::channel();
        let _ = stack_tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(2),
            })
            .await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), StackState::Closed);
    }

    #[tokio::test]
    async fn test_closing_twice_should_return_the_same_plants_outputs() {
        let (_, tx, _, _) = start_stack();

        // Close the stack
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        let plant_outputs = rx_back
            .await
            .expect("Should have received a map of plant outputs");
        assert!(!plant_outputs.is_empty());

        // Close the stack again
        let (tx_back, rx_back) = oneshot::channel();
        let _ = tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        let same_plant_outputs = rx_back
            .await
            .expect("Should have received a map of plant outputs");
        assert_eq!(same_plant_outputs, plant_outputs);
    }

    #[tokio::test]
    async fn test_closing_the_stack_should_send_an_updated_snapshot_and_forecasts() {
        let (_, tx, _, mut conn_rx) = start_stack();
        let plants = get_stack_snashot(tx.clone()).await;

        // Program a plant's setpoint
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = tx
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 100,
            }))
            .await;

        // Should receive a stack snapshot back
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::StackSnapshot { plants: _ },
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received a snapshot of the player's stack");
        };

        // Close the stack
        let (tx_back, _) = oneshot::channel();
        let _ = tx
            .send(StackMessage::CloseStack {
                tx_back,
                period_id: DeliveryPeriodId::from(0),
            })
            .await;

        // Should receive a stack snapshot back
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::StackSnapshot { plants: _ },
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received a snapshot of the player's stack");
        };

        // Should receive a stack forecasts back
        let Some(ConnectionRepositoryMessage::SendToPlayer(
            _,
            _,
            PlayerMessage::StackForecasts { forecasts: _ },
        )) = conn_rx.recv().await
        else {
            unreachable!("Should have received a forecast of the player's stack");
        };
    }

    #[tokio::test]
    async fn test_terminate_actor() {
        let (connections, _) = mpsc::channel(128);
        let token = CancellationToken::new();
        let mut market = StackActor::new(
            GameId::default(),
            PlayerId::default(),
            StackState::Open,
            DeliveryPeriodId::from(0),
            connections,
            token.clone(),
        );
        let handle = tokio::spawn(async move {
            market.run().await;
        });

        token.cancel();
        match tokio::time::timeout(Duration::from_millis(10), handle).await {
            Err(_) => unreachable!("Should have ended stack actor"),
            Ok(_) => {}
        }
    }
}
