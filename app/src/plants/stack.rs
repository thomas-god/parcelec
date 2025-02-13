use std::collections::HashMap;

use futures_util::future::join_all;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot, watch,
};
use uuid::Uuid;

use crate::{
    plants::PlantOutput,
    player::{PlayerConnection, PlayerMessage},
};

use super::{
    battery::Battery, consumers::Consumers, gas_plant::GasPlant, renewable::RenewablePlant,
    PowerPlant, PowerPlantPublicRepr,
};

#[derive(Debug, Deserialize)]
pub struct ProgramPlant {
    pub plant_id: String,
    pub setpoint: isize,
}

#[derive(Debug)]
pub enum StackMessage {
    OpenStack,
    CloseStack {
        tx_back: oneshot::Sender<HashMap<String, PlantOutput>>,
    },
    ProgramSetpoint(ProgramPlant),
    RegisterPlayerConnection(PlayerConnection),
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

/// A stack is the collection of power plants owned by a given player
pub struct StackActor {
    state: StackState,
    state_sender: watch::Sender<StackState>,
    player_id: String,
    plants: HashMap<String, Box<dyn PowerPlant + Send + Sync>>,
    tx: Sender<StackMessage>,
    rx: Receiver<StackMessage>,
    player_connections: Vec<PlayerConnection>,
}

impl StackActor {
    pub fn new(player_id: String) -> StackActor {
        let (state_tx, _) = watch::channel(StackState::Open);
        let (tx, rx) = channel::<StackMessage>(16);

        StackActor {
            state: StackState::Open,
            state_sender: state_tx,
            player_id,
            plants: default_plants(),
            player_connections: Vec::new(),
            tx,
            rx,
        }
    }

    pub fn get_tx(&self) -> Sender<StackMessage> {
        self.tx.clone()
    }

    pub fn get_state_rx(&self) -> watch::Receiver<StackState> {
        self.state_sender.subscribe()
    }

    pub async fn start(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match (&self.state, message) {
                (_, StackMessage::RegisterPlayerConnection(connection)) => {
                    self.handle_player_connection(connection).await;
                }
                (
                    StackState::Open,
                    StackMessage::ProgramSetpoint(ProgramPlant { plant_id, setpoint }),
                ) => {
                    self.program_plant_setpoint(plant_id, setpoint).await;
                }
                (StackState::Closed, StackMessage::OpenStack) => {
                    self.state = StackState::Open;
                    let _ = self.state_sender.send(StackState::Open);
                }
                (StackState::Open, StackMessage::CloseStack { tx_back }) => {
                    self.state = StackState::Closed;
                    let plant_outputs = self
                        .plants
                        .iter_mut()
                        .map(|(plant_id, plant)| (plant_id.clone(), plant.dispatch()))
                        .collect();
                    let _ = tx_back.send(plant_outputs);
                    let _ = self.state_sender.send(StackState::Closed);
                }
                (state, msg) => {
                    println!("Msg {msg:?} unsupported in state: {state:?}")
                }
            }
        }
    }

    async fn handle_player_connection(&mut self, conn: PlayerConnection) {
        if conn.player_id != self.player_id {
            return;
        }
        let conn_id = conn.id.clone();
        self.player_connections.push(conn);
        self.send_stack_snapshot(conn_id).await;
    }

    async fn send_stack_snapshot(&self, conn_id: String) {
        let Some(conn) = self.player_connections.iter().find(|c| c.id == conn_id) else {
            return;
        };

        let stack_snapshot = self.stack_snapshot();

        let _ = conn
            .tx
            .send(PlayerMessage::StackSnapshot {
                plants: stack_snapshot,
            })
            .await;
    }

    async fn send_stack_snapshot_to_all(&self) {
        let snapshot = self.stack_snapshot();

        join_all(self.player_connections.iter().map(|conn| {
            conn.tx.send(PlayerMessage::StackSnapshot {
                plants: snapshot.clone(),
            })
        }))
        .await;
    }

    fn stack_snapshot(&self) -> HashMap<String, PowerPlantPublicRepr> {
        self.plants
            .iter()
            .map(|(id, p)| (id.to_owned(), p.current_state()))
            .collect()
    }

    async fn program_plant_setpoint(&mut self, plant_id: String, setpoint: isize) {
        if let Some(plant) = self.plants.get_mut(&plant_id) {
            let PlantOutput { cost, .. } = plant.program_setpoint(setpoint);
            println!("Programmed setpoint {setpoint} for plant {plant_id} (cost: {cost}");
            self.send_stack_snapshot_to_all().await;
        };
    }
}

fn default_plants() -> HashMap<String, Box<dyn PowerPlant + Send + Sync>> {
    let mut map: HashMap<String, Box<dyn PowerPlant + Send + Sync>> = HashMap::new();
    map.insert(
        Uuid::new_v4().to_string(),
        Box::new(Battery::new(1_000, 500)),
    );
    map.insert(
        Uuid::new_v4().to_string(),
        Box::new(GasPlant::new(85, 1000)),
    );
    map.insert(
        Uuid::new_v4().to_string(),
        Box::new(RenewablePlant::new(500)),
    );
    map.insert(
        Uuid::new_v4().to_string(),
        Box::new(Consumers::new(1500, 56)),
    );
    map
}

#[cfg(test)]
mod tests_stack {
    use std::{collections::HashMap, time::Duration};

    use tokio::sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    };
    use uuid::Uuid;

    use crate::{
        plants::{
            stack::{ProgramPlant, StackActor, StackMessage, StackState},
            PowerPlantPublicRepr,
        },
        player::{PlayerConnection, PlayerMessage},
    };

    fn start_stack() -> (String, Sender<StackMessage>) {
        let player_id = Uuid::new_v4().to_string();
        let mut stack = StackActor::new(player_id.clone());
        let stack_tx = stack.get_tx();
        tokio::spawn(async move {
            stack.start().await;
        });
        (player_id, stack_tx)
    }

    #[tokio::test]
    async fn test_register_player_connection() {
        let (player_id, stack_tx) = start_stack();

        let (tx, mut rx) = channel::<PlayerMessage>(16);

        // Register player connection
        let connection_id = Uuid::new_v4().to_string();
        let _ = stack_tx
            .send(StackMessage::RegisterPlayerConnection(PlayerConnection {
                id: connection_id,
                player_id,
                tx: tx.clone(),
            }))
            .await;

        // Should receive a snapshot of the stack
        let Some(PlayerMessage::StackSnapshot { plants: _ }) = rx.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
    }

    async fn register_player_connection(
        player_id: &str,
        stack_tx: Sender<StackMessage>,
    ) -> (
        String,
        Receiver<PlayerMessage>,
        HashMap<String, PowerPlantPublicRepr>,
    ) {
        let (tx, mut rx) = channel::<PlayerMessage>(16);
        let connection_id = Uuid::new_v4().to_string();
        let _ = stack_tx
            .send(StackMessage::RegisterPlayerConnection(PlayerConnection {
                id: connection_id.clone(),
                player_id: player_id.to_string(),
                tx: tx.clone(),
            }))
            .await;

        // Should receive a snapshot of the stack
        let Some(PlayerMessage::StackSnapshot { plants }) = rx.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
        (connection_id, rx, plants)
    }

    #[tokio::test]
    async fn test_programm_a_plant_setpoint() {
        let (player_id, stack_tx) = start_stack();
        let (_, mut player_rx, plants) =
            register_player_connection(&player_id, stack_tx.clone()).await;

        // Program a plant's setpoint
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = stack_tx
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 100,
            }))
            .await;

        // Should receive a stack snapshot back
        let Some(PlayerMessage::StackSnapshot { plants: _ }) = player_rx.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
    }

    #[tokio::test]
    async fn test_programm_a_plant_setpoint_multiple_connections() {
        let (player_id, stack_tx) = start_stack();

        // Register two players
        let (_, mut player_rx_1, plants) =
            register_player_connection(&player_id, stack_tx.clone()).await;
        let (_, mut player_rx_2, _) =
            register_player_connection(&player_id, stack_tx.clone()).await;

        // Program a plant's setpoint
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = stack_tx
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 100,
            }))
            .await;

        // Each conn should receive a stack snapshot back
        let Some(PlayerMessage::StackSnapshot { plants: _ }) = player_rx_1.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
        let Some(PlayerMessage::StackSnapshot { plants: _ }) = player_rx_2.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
    }

    #[tokio::test]
    async fn test_no_dispatch_when_stack_closed() {
        let (player_id, stack) = start_stack();
        let (_, mut player_rx, plants) =
            register_player_connection(&player_id, stack.clone()).await;

        // Close the stack
        let (tx_back, _) = oneshot::channel();
        let _ = stack
            .clone()
            .send(StackMessage::CloseStack { tx_back })
            .await;

        // Try to send a dispatch command
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = stack
            .clone()
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 100,
            }))
            .await;

        // Should not receive a new stack snapshot
        tokio::select! {
        _ = player_rx.recv() => {
            unreachable!("Should not have received a message");
        }
        _ = tokio::time::sleep(Duration::from_micros(1)) => {}
        };
    }
    #[tokio::test]
    async fn test_receive_plant_outputs_when_closing_stack() {
        let (_, stack) = start_stack();

        // Close the stack
        let (tx_back, rx_back) = oneshot::channel();
        let _ = stack
            .clone()
            .send(StackMessage::CloseStack { tx_back })
            .await;

        let plant_outputs = rx_back
            .await
            .expect("Should have received a map of plant outputs");
        assert!(!plant_outputs.is_empty());
    }
    #[tokio::test]
    async fn test_register_connection_when_stack_closed() {
        let (player_id, stack) = start_stack();

        // Close the stack
        let (tx_back, _) = oneshot::channel();
        let _ = stack.send(StackMessage::CloseStack { tx_back }).await;

        // Register a player
        let (tx, mut rx) = channel::<PlayerMessage>(16);
        let connection_id = Uuid::new_v4().to_string();
        let _ = stack
            .send(StackMessage::RegisterPlayerConnection(PlayerConnection {
                id: connection_id.clone(),
                player_id: player_id.to_string(),
                tx: tx.clone(),
            }))
            .await;

        // Should receive a snapshot of the stack, even if the stack is closed
        let Some(PlayerMessage::StackSnapshot { plants }) = rx.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
        assert!(!plants.is_empty());

        // Reopen the stack
        let _ = stack.send(StackMessage::OpenStack).await;

        // Check dispatch is working
        let Some(plant_id) = plants.keys().next() else {
            unreachable!("Stack should contain at least one power plant");
        };
        let _ = stack
            .send(StackMessage::ProgramSetpoint(ProgramPlant {
                plant_id: plant_id.to_owned(),
                setpoint: 0,
            }))
            .await;
        let Some(PlayerMessage::StackSnapshot { plants: _ }) = rx.recv().await else {
            unreachable!("Should have received a snapshot of the player's stack");
        };
    }

    #[tokio::test]
    async fn test_stack_state_watch() {
        let player_id = Uuid::new_v4().to_string();
        let mut stack = StackActor::new(player_id.clone());
        let stack_tx = stack.get_tx();
        let mut state_rx = stack.get_state_rx();
        tokio::spawn(async move {
            stack.start().await;
        });

        assert_eq!(*state_rx.borrow(), StackState::Open);

        // Close the Stack
        let (tx_back, _) = oneshot::channel();
        let _ = stack_tx.send(StackMessage::CloseStack { tx_back }).await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), StackState::Closed);

        // Reopen the Stack
        let _ = stack_tx.send(StackMessage::OpenStack).await;
        assert!(state_rx.changed().await.is_ok());
        assert_eq!(*state_rx.borrow_and_update(), StackState::Open);
    }
}
